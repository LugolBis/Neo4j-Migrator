//! This module simplify interactions with Neo4j database

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;

/// A structure that represent a Neo4j connection
#[derive(Debug)]
pub struct Neo4j {
    uri: String,
    username: String,
    password: String,
    database: String,
    import_folder: String,
}

impl Neo4j {
    pub fn new(
        uri: &str,
        username: &str,
        password: &str,
        database: &str,
        import_folder: &str,
    ) -> Self {
        Self {
            uri: String::from(uri),
            username: String::from(username),
            password: String::from(password),
            database: String::from(database),
            import_folder: String::from(import_folder),
        }
    }

    #[allow(unused)]
    pub fn get_uri(&self) -> &String {
        &self.uri
    }

    #[allow(unused)]
    pub fn set_uri(&mut self, new_uri: String) {
        self.uri = new_uri
    }

    #[allow(unused)]
    pub fn get_username(&self) -> &String {
        &self.username
    }

    #[allow(unused)]
    pub fn set_username(&mut self, new_username: String) {
        self.username = new_username
    }

    #[allow(unused)]
    pub fn get_password(&self) -> &String {
        &self.password
    }

    #[allow(unused)]
    pub fn set_password(&mut self, new_password: String) {
        self.password = new_password
    }

    pub fn get_database(&self) -> &String {
        &self.database
    }

    #[allow(unused)]
    pub fn set_database(&mut self, new_database: String) {
        self.database = new_database
    }

    pub fn get_import_folder(&self) -> &String {
        &self.import_folder
    }

    #[allow(unused)]
    pub fn set_import_folder(&mut self, new_import_folder: String) {
        self.import_folder = new_import_folder
    }

    #[allow(unused)]
    /// This method take in input only one Cypher query.<br>
    /// To run more queries please use ```Neo4j::execute_script()```
    pub fn execute_query(&self, query: &str) -> Result<String, String> {
        let output = Command::new("cypher-shell")
            .arg("-a")
            .arg(&self.uri)
            .arg("-u")
            .arg(&self.username)
            .arg("-p")
            .arg(&self.password)
            .arg("-d")
            .arg(&self.database)
            .arg(format!("{}", query))
            .output()
            .expect("Error when try to execute the cypher-shell command");
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let result = format!("\nResult of the cypher query :\n{}", stdout);
            return Ok(result);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let result = format!(
                "\nError when try to execute the cypher query : {}\n{}",
                query, stderr
            );
            return Err(result);
        }
    }

    /// Execute the cypher script using the '*-f*' flag of *cypher-shell*<br>
    /// IMPORTANT : The path of the Cypher script need to be the reel path (not the relative path).
    pub fn execute_script(&self, script_path: &str) -> Result<String, String> {
        let output = Command::new("cypher-shell")
            .arg("-a")
            .arg(&self.uri)
            .arg("-u")
            .arg(&self.username)
            .arg("-p")
            .arg(&self.password)
            .arg("-d")
            .arg(&self.database)
            .arg("-f")
            .arg(format!("{}", script_path))
            .output()
            .expect("Error when try to execute the cypher-shell command");
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

    /// This function configure the ***apoc.conf*** file in the ***conf*** directory of your database.<br>
    /// /!\ **WARNING** : This function truncate the content of ***apoc.conf***
    pub fn configure_db_on_linux(&mut self) -> Result<String, String> {
        let config_path = format!(
            "{}/Neo4j/config.cql",
            env::current_dir()
                .map_err(|error| format!("{}", error))?
                .display()
        );
        match self.execute_script(&config_path) {
            Ok(content) => {
                let lines = content.split("\n").collect::<Vec<&str>>();
                for line in lines {
                    if line.contains("server.directories.neo4j_home") {
                        let content = "apoc.trigger.enabled=true\napoc.import.file.enabled=true\napoc.export.file.enabled=true";
                        let file_path = format!(
                            "{}/conf/apoc.conf",
                            line.split(",").collect::<Vec<&str>>()[1]
                                .trim()
                                .replace('"', "")
                        );
                        let mut file = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .truncate(true)
                            .open(&file_path)
                            .map_err(|error| format!("{}", error))?;
                        match file.write_all(&content.as_bytes()) {
                            Ok(_) => println!("\nSuccessfully write in the file {}\n", file_path),
                            Err(error) => {
                                return Err(format!("{}", error));
                            }
                        }
                    }
                    if line.contains("server.directories.import") {
                        self.import_folder = line.split(",").collect::<Vec<&str>>()[1]
                            .trim()
                            .replace('"', "");
                        self.import_folder.push('/');
                        println!("\nThe import folder was set to :\n{}", self.import_folder)
                    }
                }
                Ok(String::from("Successfully configure the database files."))
            }
            Err(res) => Err(format!("{}", res)),
        }
    }
}
