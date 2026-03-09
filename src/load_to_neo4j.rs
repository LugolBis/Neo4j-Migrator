//! This module contains the logic to load the formated data to neo4j

use std::env;
use std::fs::{self, DirEntry, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::neo4j::Neo4j;

/// This method perform the 'neo4j-admin import' from the ```&self.import_folder```<br><br>
/// **WARNING** : This method construct the command 'neo4j-admin import' by detecting <br>
/// the **CSV** files in the folder, you need to assert that there isn't other CSV files <br>
/// than these you need for the import. Moreover assert that the CSV files who are contain the <br>
/// *relationships* have '_ref_' in their name.
pub fn load_with_admin(db_neo4j: &Neo4j) -> Result<String, String> {
    let import_folder = db_neo4j.get_import_folder();
    println!("Import folder path : {}", import_folder);
    if let Err(error) = env::set_current_dir(Path::new(&import_folder)) {
        return Err(format!("{}", error));
    }

    let mut command: Command;
    if cfg!(target_os = "windows") {
        command = Command::new("bin\neo4j-admin.bat");
    } else {
        command = Command::new("../bin/neo4j-admin");
    }
    command.args(["database", "import", "full", db_neo4j.get_database()]);

    let path = Path::new(&import_folder);
    let mut nodes: Vec<String> = Vec::new();
    let mut relationships: Vec<String> = Vec::new();
    match fs::read_dir(path) {
        Ok(entries) => {
            let entries = entries.flatten().collect::<Vec<DirEntry>>();
            for entry in entries {
                let file_name = entry.file_name().into_string().unwrap_or_default();
                match (&file_name.find("_ref_"), &file_name.ends_with(".csv")) {
                    (Some(_), true) => {
                        relationships.push(file_name);
                    }
                    (None, true) => {
                        nodes.push(file_name);
                    }
                    (_, _) => {}
                }
            }

            // WARNING : DON'T TRY TO OPTIMIZE THAT
            // neo4j-admin needs to pass each different Label/RelsType as an argument
            // If you have the two labels "A" and "B" in two differents CSV files you absolutly need to pass the following arguments :
            // ```neo4j-admin ... --nodes=/path/to/A.csv --nodes=/path/to/B.csv```
            for node in nodes {
                command.arg(format!("--nodes={}{}", import_folder, node));
            }

            for relationship in relationships {
                command.arg(format!("--relationships={}{}", import_folder, relationship));
            }

            command.args([
                "--delimiter=;",
                "--array-delimiter=,",
                "--overwrite-destination",
                "--verbose",
            ]);

            let output = command.output();
            match output {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let result = format!("{}", stdout);
                        Ok(result)
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let result = format!("{}", stderr);
                        Err(result)
                    }
                }
                Err(error) => Err(format!(
                    "ERROR when try to execute the command :\n{:?}\n{}",
                    command, error
                )),
            }
        }
        Err(error) => Err(format!(
            "ERROR when try to list the files in the import directory :\n{}",
            error
        )),
    }
}

#[allow(unused)]
/// This function help you to recovery your database after it was down due to inconsistent import.
pub fn recovery_database(db_neo4j: &Neo4j) -> Result<String, String> {
    if let Err(error) = env::set_current_dir(Path::new(&db_neo4j.get_import_folder())) {
        return Err(format!("{}", error));
    }

    let mut command: Command;
    if cfg!(target_os = "windows") {
        command = Command::new("bin\neo4j-admin.bat");
    } else {
        command = Command::new("../bin/neo4j-admin");
    }
    command.args(["database", "import", "full", db_neo4j.get_database()]);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("{}/RECOVERY.csv", db_neo4j.get_import_folder()))
        .map_err(|e| format!("ERROR : load_to_neo4j.rs - recovery_database()\n{}", e))?;

    let _ = file.write_all(":ID;:LABEL".as_bytes());

    command.args([
        "--nodes=RECOVERY.csv",
        "--delimiter=;",
        "--array-delimiter=,",
        "--overwrite-destination",
        "--verbose",
    ]);

    let output = command.output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let result = format!("{}", stdout);
                Ok(result)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let result = format!("{}", stderr);
                Err(result)
            }
        }
        Err(error) => Err(format!(
            "ERROR when try to execute the command :\n{:?}\n{}",
            command, error
        )),
    }
}
