use mylog::logs::init;
use std::{collections::HashMap, path::PathBuf};

const HELP_MESS: &str = r#"
    Usage : neo4j-migrator <ARGS>

    <ARGS> :
        REQUIRED :
        -pg_host=<Host> : <Host> is the PostgreSQL host.
        -pg_port=<Port> : <Port> is the PostgreSQL port.
        -pg_user=<Username> : <Username> is the name of the user used to connect.
        -pg_password=<Password> : <Password> is the password used to connect to the PostgreSQL database.
        -pg_database=<Database> : <Database> is the PostgreSQL database name.
        -neo4j_database=<Database> : <Database> is the Neo4j database name.
        -neo4j_import_folder=<Folder> : <Folder> is the absolute path to the Neo4j database import path.

        OPTIONAL :
        --help, -help, help : Display this message.
        -work_folder=<Folder>: <Folder> is the absolute path to the folder who's gonna be used as storage for the program.
        --ni, -ni, --non-interactive, -non-interactive: Deativate user interaction and runing the program end-to-end, assert that your Neo4j instance is stop before use it. 

    Note : Don't use quotes around values of <ARGS>.
"#;

fn get_arg_value(arg: &&str, index: usize) -> Result<String, String> {
    Ok(arg
        .split("=")
        .collect::<Vec<&str>>()
        .get(1)
        .ok_or_else(|| format!("Invalid argument for 'pg_host' at index {}", index))?
        .trim()
        .to_string())
}

fn parse_args(args: Vec<&str>) -> Result<HashMap<String, String>, String> {
    let mut config = HashMap::new();
    for (index, arg) in args.iter().enumerate() {
        if arg.starts_with("-pg_host=") {
            config.insert("PG_HOST".into(), get_arg_value(arg, index)?);
        } else if arg.starts_with("-pg_port=") {
            config.insert("PG_PORT".into(), get_arg_value(arg, index)?);
        } else if arg.starts_with("-pg_user=") {
            config.insert("PG_USER".into(), get_arg_value(arg, index)?);
        } else if arg.starts_with("-pg_password=") {
            config.insert("PG_PASSWORD".into(), get_arg_value(arg, index)?);
        } else if arg.starts_with("-pg_database=") {
            config.insert("PG_DB".into(), get_arg_value(arg, index)?);
        } else if arg.starts_with("-neo4j_database=") {
            config.insert("NEO4J_DB".into(), get_arg_value(arg, index)?);
        } else if arg.starts_with("-neo4j_import_folder=") {
            config.insert("NEO4J_IMPORT_FOLDER".into(), get_arg_value(arg, index)?);
        } else if arg.to_lowercase().replace("-", "").starts_with("help") {
            config.insert("HELP".into(), "".into());
        } else if ["--ni", "-ni", "--non-interactive", "-non-interactive"].contains(arg) {
            config.insert("NI".into(), "".into());
        } else if arg.starts_with("-work_folder=") {
            config.insert("WORK_FOLDER".into(), get_arg_value(arg, index)?);
        }
    }
    Ok(config)
}

pub async fn main(args: Vec<&str>) -> Result<(), String> {
    use crate::format_to_neo4j::*;
    use crate::load_to_neo4j::*;
    use crate::neo4j::Neo4j;
    use crate::postgresql::PostgreSQL;
    use std::env;
    use std::fs;
    use std::io;

    let conf = parse_args(args)?;
    let current_dir = format!("{}", env::current_dir().unwrap().display());

    if let Some(_) = conf.get("HELP") {
        println!("{}", HELP_MESS);
        return Ok(());
    }

    let work_folder: PathBuf;
    if conf.get("WORK_FOLDER") == None {
        let folder_path = PathBuf::from(&current_dir).join("neo4j_migrator");
        fs::create_dir_all(&folder_path).map_err(|e| {
            format!(
                "Failed to create the folder {} due to : {}",
                folder_path.display(),
                e
            )
        })?;
        work_folder = folder_path;
    } else {
        work_folder = PathBuf::from(conf["WORK_FOLDER"].clone());
    }

    // Configure logs
    init(
        format!("{}", work_folder.display()),
        "100mo".into(),
        "4d".into(),
    )?;

    let db_postgresql = PostgreSQL::from(
        conf.get("PG_HOST")
            .ok_or_else(|| "Failed to get the PostgreSQL Host from CLI args.".to_string())?,
        conf.get("PG_PORT")
            .ok_or_else(|| "Failed to get the PostgreSQL Port from CLI args.".to_string())?,
        conf.get("PG_USER")
            .ok_or_else(|| "Failed to get the PostgreSQL User from CLI args.".to_string())?,
        conf.get("PG_PASSWORD")
            .ok_or_else(|| "Failed to get the PostgreSQL Password from CLI args.".to_string())?,
        conf.get("PG_DB").ok_or_else(|| {
            "Failed to get the PostgreSQL Database name from CLI args.".to_string()
        })?,
    )
    .await
    .map_err(|_| "Failed to initialize PostgreSQL connection.".to_string())?;

    let db_neo4j = Neo4j::new(
        conf.get("NEO4J_DB")
            .ok_or_else(|| "Failed to get the Neo4j Database name from CLI args.".to_string())?,
        conf.get("NEO4J_IMPORT_FOLDER")
            .ok_or_else(|| "Failed to get the Neo4j import folder from CLI args.".to_string())?,
    );

    // PostgreSQL part

    let save_meta_data = work_folder.join("postgresql_meta_data.json");

    let tables_folder = work_folder.clone();
    let save_fk = work_folder.join("neo4j_migrator_FK.csv");

    match db_postgresql.export_meta_data(&save_meta_data).await {
        Ok(_) => {
            println!("Successfuly export meta data !");
            match db_postgresql.export_tables_csv(&tables_folder).await {
                Ok(_) => {
                    println!("Successfuly export tables !");
                }
                Err(_) => return Err("ERROR when try to export tables.".into()),
            }
        }
        Err(_) => return Err("ERROR when try to export meta data.".into()),
    }

    // Neo4J part

    match generate_import_files(
        &db_neo4j,
        &work_folder,
        &save_meta_data,
        &tables_folder,
        &save_fk,
    ) {
        Ok(result) => println!("{}", result),
        Err(result) => println!("{}", result),
    }

    let mut user_input = String::new();
    if let Some(_) = conf.get("NI") {
        user_input = "YES".into();
    } else {
        println!("Please stop your Neo4j database to process the import.\nWhen it is done enter 'YES' below :\n");

        io::stdin()
            .read_line(&mut user_input)
            .expect("Error when try to read the user input.");
    }

    if user_input.trim() == "YES" {
        match load_with_admin(&db_neo4j) {
            Ok(result) => {
                println!("{}", result);
                Ok(())
            }
            Err(result) => Err(result),
        }
    } else {
        println!("Ok, you could done the import later with the method 'load_with_admin()' of the struct 'Neo4j'.");
        Ok(())
    }
}
