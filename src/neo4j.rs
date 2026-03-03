//! This module simplify interactions with Neo4j database

/// A structure that represent a Neo4j connection
pub struct Neo4j {
    database: String,
    import_folder: String,
}

const _CONFIG_QUERY: &str = "CALL dbms.listConfig() YIELD name, value, description WHERE name = 'server.directories.neo4j_home' RETURN name, value, description;";

impl Neo4j {
    pub fn new(
        database: &str,
        import_folder: &str,
    ) -> Self {
        Self {
            database: String::from(database),
            import_folder: String::from(import_folder),
        }
    }

    pub fn get_database(&self) -> &String {
        &self.database
    }

    pub fn get_import_folder(&self) -> &String {
        &self.import_folder
    }

    /// Convert PostgreSQL Type into Neo4j type.<br>
    /// CAUTION : These convertion are suitable for mass export.
    pub fn convert_postgresql_type(postgresql_type: &str) -> Result<String, String> {
        let target_type = postgresql_type.to_uppercase();
        match target_type.as_str() {
            "SMALLINT" | "INT" | "INTEGER" | "BIGINT" => Ok(String::from("LONG")),
            "BIGSERIAL" | "SMALLSERIAL" | "SERIAL" => Ok(String::from("LONG")),
            "REAL" | "DOUBLE" | "DECIMAL" | "PRECISION" | "FLOAT8" | "DOUBLE PRECISION"
            | "NUMERIC" => Ok(String::from("DOUBLE")),
            "VARCHAR" | "TEXT" | "CHAR" | "CHARACTER VARYING" | "CHARACTER" | "BPCHAR" => {
                Ok(String::from("STRING"))
            }
            "BOOLEAN" => Ok(String::from("BOOLEAN")),
            "DATE" | "TIME" | "TIMESTAMP" => Ok(String::from("DATE")),
            "TIMESTAMP WITHOUT TIME ZONE"
            | "TIME WITH TIME ZONE"
            | "TIME WITHOUT TIME ZONE"
            | "TIMESTAMP WITH TIME ZONE" => Ok(String::from("STRING")),
            "JSON" | "XML" | "JSONB" | "INTERVAL" | "UUID" | "MONEY" => Ok(String::from("STRING")),
            "POINT" => Ok(String::from("STRING")),
            "ARRAY" | "TSVECTOR" | "TSQUERY" => Ok(String::from("STRING[]")),
            "BIGINT[]" => Ok(String::from("LONG[]")),
            "BYTEA" | "ENUM" | "BIT" | "BIT VARYING" => Ok(String::from("STRING")),
            "LINE" | "LSEG" | "PATH" | "POLYGON" | "CIRCLE" => Ok(String::from("STRING")),
            "CIDR" | "INET" | "MACADDR" | "MACADDR8" => Ok(String::from("STRING")),
            _ => Err(format!(
                "ERROR : Can't convert THE PostgreSQL type '{}' into Neo4j type.",
                target_type
            )),
        }
    }
}
