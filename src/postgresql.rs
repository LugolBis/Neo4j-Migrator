//! This module simplify interactions with PostgreSQL database

use std::process::Command;

/// A structure that represent a PostgreSQL connection
#[derive(Debug)]
pub struct PostgreSQL {
    host: String,
    port: String,
    username: String,
    password: String,
    database: String,
}

impl PostgreSQL {
    pub fn new(host: &str, port: &str, username: &str, password: &str, database: &str) -> Self {
        Self {
            host: String::from(host),
            port: String::from(port),
            username: String::from(username),
            password: String::from(password),
            database: String::from(database),
        }
    }

    /// This method take in input only one PostgreSQL query.<br>
    /// To run more queries please use ```PostgreSQL.execute_script()```<br>
    /// Use the parameter ***format_csv*** to configure the format of the output.
    pub fn execute_query(&self, query: &str, format_csv: bool) -> Result<String, String> {
        let mut command = Command::new("psql");
        command.args([
            "-h",
            &self.host,
            "-p",
            &self.port,
            "-U",
            &self.username,
            "-d",
            &self.database,
            "-c",
            query,
        ]);
        if format_csv {
            command.arg("--csv");
        }
        command.env("PGPASSWORD", &self.password);

        let output = command
            .output()
            .expect("Échec de l'exécution de la commande psql");
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let result = format!("{}", stdout);
            return Ok(result);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let result = format!("{}", stderr);
            return Err(result);
        }
    }

    /// The path of the script need to be the reel path (not the relative path).
    pub fn execute_script(&self, script_path: &str) -> Result<String, String> {
        let output = Command::new("psql")
            .args([
                "-h",
                &self.host,
                "-p",
                &self.port,
                "-U",
                &self.username,
                "-d",
                &self.database,
                "-f",
                script_path,
                "--csv",
            ])
            .env("PGPASSWORD", &self.password)
            .output()
            .expect("Échec de l'exécution de la commande psql");
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let result = format!("{}", stdout);
            return Ok(result);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let result = format!("{}", stderr);
            return Err(result);
        }
    }

    /// This method allows you to export the result of the SQL function called ```function_name```
    /// and define in the PostgreSQL script ```script_path``` to the file specified in ```save_path```.
    /// You should use it to export the meta data of your PostgreSQL database.
    pub fn export_from_sql(&self,script_path: &str,function_name: &str,save_path: &str) -> Result<String, String> {
        match &self.execute_script(script_path) {
            Ok(_) => {
                println!(
                    "\nExport data from PostgreSQL - Successfully created the function {}\n",
                    function_name
                );
                let query = format!(r"\copy (select {}()) to '{}'", function_name, save_path);
                match self.execute_query(query.as_str(), false) {
                    Ok(res) => {
                        Ok(res)
                    }
                    Err(error) => {
                        Err(error)
                    }
                }
            }
            Err(error) => {
                Err(String::from(error))
            }
        }
    }

    /// This method export in CSV all the tables from the public scheme of the
    /// PostgreSQL database to the folder passed in argument.
    pub fn export_tables_csv(&self, folder_path: &str) -> Result<String, String> {
        let query =
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'";
        match &self.execute_query(query, true) {
            Ok(result) => {
                let tables = result.split("\n").collect::<Vec<&str>>();
                for index in 1..tables.len() - 1 {
                    if let Some(table) = tables.get(index) {
                        let table = *table;
                        if let Err(error) = &self.execute_query(
                            &format!(
                                r"\copy {} to '{}{}.csv' CSV HEADER",
                                table, folder_path, table
                            ),
                            true,
                        ) {
                            return Err(format!(
                                "ERROR : when try to export the data of the table : '{}'\n{}",
                                table, error
                            ));
                        }
                    }
                }
                Ok(String::from(""))
            }
            Err(error) => Err(String::clone(error)),
        }
    }
}
