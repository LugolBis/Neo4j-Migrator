//! This module simplify interactions with PostgreSQL database

use futures::TryStreamExt;
use mylog::{error, info};
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::{Pool, Postgres, Row};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use tokio::time::Duration;

const META_DATA_SCRIPT: &str = include_str!("../PostgreSQL/meta_data.sql");

/// A structure that represent a PostgreSQL connection
pub struct PostgreSQL {
    pool: Pool<Postgres>,
}

impl PostgreSQL {
    pub async fn from(
        host: &str,
        port: &str,
        username: &str,
        password: &str,
        database: &str,
    ) -> Result<PostgreSQL, ()> {
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            username, password, host, port, database
        );

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect(&url)
            .await
            .map_err(|e| error!("{}", e))?;

        Ok(Self { pool })
    }

    pub async fn query(&self, query: &str) -> Result<Vec<PgRow>, ()> {
        Ok(sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| error!("{}\n\tQuery : {}", e, query))?)
    }

    pub async fn copy_to_file(
        conn: &mut sqlx::pool::PoolConnection<Postgres>,
        query: &str,
        file_path: PathBuf,
    ) -> Result<(), ()> {
        // "COPY (SELECT * FROM my_table) TO STDOUT"
        let mut stream = conn
            .copy_out_raw(query)
            .await
            .map_err(|e| error!("{}", e))?;

        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(file_path)
            .map_err(|e| error!("{}", e))?;

        while let Some(chunk) = stream.try_next().await.map_err(|e| error!("{}", e))? {
            let _ = file.write_all(&chunk);
        }

        Ok(())
    }

    /// This method allows you to export the PostgreSQL meta data intop the `save_path` file.
    pub async fn export_meta_data(&self, save_path: &str) -> Result<(), ()> {
        match &self.query(META_DATA_SCRIPT).await {
            Ok(rows) => {
                let mut file = OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(save_path)
                    .map_err(|e| e.to_string())
                    .map_err(|e| error!("{}", e))?;

                if let Some(row) = rows.get(0) {
                    let content: String = row
                        .try_get(0)
                        .map_err(|e| e.to_string())
                        .map_err(|e| error!("{}", e))?;
                    let _ = file.write_all(content.as_bytes());
                }
                Ok(())
            }
            Err(_) => Err(error!("Failed to execute meta data script.")),
        }
    }

    /// This method export in CSV all the tables from the public scheme of the
    /// PostgreSQL database to the folder passed in argument.
    pub async fn export_tables_csv(&self, folder_path: &str) -> Result<(), ()> {
        let query =
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'";
        match &self.query(query).await {
            Ok(rows) => {
                for row in rows {
                    let table: String = row.try_get(0).map_err(|e| error!("{}", e))?;
                    let query = format!(
                        "COPY (SELECT * FROM {}) TO STDOUT (FORMAT CSV, HEADER)",
                        table
                    );
                    let file_path =
                        PathBuf::from(folder_path).join(PathBuf::from(format!("{table}.csv")));

                    let mut conn = self.pool.acquire().await.map_err(|e| error!("{}", e))?;

                    Self::copy_to_file(&mut conn, &query, file_path.clone())
                        .await
                        .map_err(|_| {
                            error!(
                                "Failed to copy the table {} to {}",
                                table,
                                file_path.display()
                            )
                        })?;
                }
                Ok(())
            }
            Err(_) => Err(error!(
                "Failed to query the meta datas to get the table names.".to_string()
            )),
        }
    }
}
