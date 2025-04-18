mod format_to_neo4j;
mod load_to_neo4j;
mod neo4j;
mod postgresql;
mod translate;
mod utils;

fn main() {
    match example() {
        Ok(_) => println!("\n\nSuccessfully migrate the database to Neo4j !"),
        Err(error) => println!("{}", error),
    }
}

fn example() -> Result<(), String> {
    use format_to_neo4j::*;
    use load_to_neo4j::*;
    use neo4j::Neo4j;
    use postgresql::PostgreSQL;
    use std::env;
    use std::fs;
    use std::io;

    let current_dir = format!("{}", env::current_dir().unwrap().display());

    // Your personnal informations about the connections of the databases
    let infos = fs::read_to_string("env.txt").map_err(|error| format!("{}", error))?;
    let infos = infos.split("\n").collect::<Vec<&str>>();

    let db_postgresql = PostgreSQL::new(infos[0], infos[1], infos[2], infos[3], infos[4]);

    let mut db_neo4j = Neo4j::new(infos[5], infos[6], infos[7], infos[8], "");

    // PostgreSQL part

    let script_meta_data = format!("{}/PostgreSQL/meta_data.sql", current_dir);
    let function_meta_data = "export_tables_metadata";
    let save_meta_data = format!("{}/Data/postgresql_meta_data.json", current_dir);

    let tables_folder = format!("{}/Data/", current_dir);
    let save_fk = format!("{}/Neo4j/FK.csv", current_dir);

    match db_postgresql.export_from_sql(&script_meta_data, function_meta_data, &save_meta_data) {
        Ok(_) => {
            println!("Successfuly export meta data !");
            match db_postgresql.export_tables_csv(&tables_folder) {
                Ok(_) => {
                    println!("Successfuly export tables !");
                }
                Err(result) => {
                    return Err(format!("ERROR when try to export tables :\n{}", result))
                }
            }
        }
        Err(result) => return Err(format!("ERROR when try to export meta data :\n{}", result)),
    }

    // Neo4J part

    match db_neo4j.configure_db_on_linux() {
        Ok(result) => {
            println!("{}", result);
            match generate_import_files(&db_neo4j, &save_meta_data, &tables_folder, &save_fk) {
                Ok(result) => println!("{}", result),
                Err(result) => println!("{}", result),
            }
        }
        Err(result) => println!("{}", result),
    }

    println!("Please stop your Neo4j database to process the import.\nWhen it is done enter 'YES' below :\n");
    let mut user_input = String::new();
    io::stdin()
        .read_line(&mut user_input)
        .expect("Error when try to read the user input.");

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