use std::process::Command;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::env;
use std::fs;

use serde_json::Value;
use polars::prelude::*;

use crate::neo4j::*;
use crate::utils::*;

pub fn load_with_admin(db_neo4j: &Neo4j) -> Result<String, String> {
    //! This method perform the 'neo4j-admin import' from the ```&self.import_folder```<br><br>
    //! **WARNING** : This method construct the command 'neo4j-admin import' by detecting <br>
    //! the **CSV** files in the folder, you need to assert that there isn't other CSV files <br>
    //! than these you need for the import. Moreover assert that the CSV files who are contain the <br>
    //! *relationships* have '_REF_' in their name.
    if let Err(error) = env::set_current_dir(Path::new(db_neo4j.get_import_folder())) {
        return Err(format!("{}",error));
    }

    let mut command:Command;
    if cfg!(target_os = "windows") { command = Command::new("bin\neo4j-admin.bat"); }
    else { command = Command::new("../bin/neo4j-admin"); }
    command.args(["database","import","full",db_neo4j.get_database()]);

    let path = Path::new(db_neo4j.get_import_folder());
    let mut nodes: Vec<String> = Vec::new();
    let mut relationships: Vec<String> = Vec::new();
    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry_) = entry {
                    let file = entry_.file_name().into_string().unwrap_or_default();
                    match (&file.find("_REF_"), &file.ends_with(".csv")) {
                        (Some(_), true) => { relationships.push(file); },
                        (None, true) => { nodes.push(file); }
                        (_, _) => {}
                    }
                }
            }
            for node in nodes {
                command.arg(format!("--nodes={}",node));
            }
            for relationship in relationships {
                command.arg(format!("--relationships={}",relationship));
            }
            command.args(["--delimiter=;", "--array-delimiter=,", "--overwrite-destination", "--verbose"]);

            let output = command.output();
            match output {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let result = format!("{}",stdout);
                        return Ok(result);
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let result = format!("{}",stderr);
                        return Err(result);
                    }
                },
                Err(error) => { 
                    return Err(format!("ERROR when try to execute the command :\n{:?}\n{}",command,error))
                }
            }
        },
        Err(error) => { 
            return Err(format!("ERROR when try to list the files in the import directory :\n{}",error))
        }
    }
}

fn create_csv_headers(db_neo4j: &Neo4j,meta_data_path:&str) -> Result<String, String> {
    //! Generate **CSV** files who contains the **HEADERS** needed to generate and organise the
    //! data to be imported to Neo4j.
    if let Err(error) = clean_directory(&db_neo4j.get_import_folder()) {
        return Err(error);
    }

    let content = fs::read_to_string(meta_data_path)
        .map_err(|error| format!("{}",error))?;

    let json_object:Value = serde_json::from_str(&content)
        .map_err(|error| format!("{}",error))?;

    let constraints_path = format!("{}/Neo4j/constraints.cql", env::current_dir()
        .map_err(|error| format!("{}",error))?.display());

    let triggers_path = format!("{}/Neo4j/triggers.cql", env::current_dir()
        .map_err(|error| format!("{}",error))?.display());

    let mut constraints_content = String::new();
    let mut triggers_content = String::new();
    let mut fk_content = String::new();

    match json_object {
        Value::Array(vector) => {
            for table in vector {
                let label = String::from(table["table_name"].as_str()
                    .ok_or_else(|| 
                    format!("Error when try to get the 'table_name' field in {}",table))?).to_uppercase();
                let mut headers = String::from(":ID;");
                let mut foreign_keys: Vec<String> = Vec::new();

                let columns = table["columns"].as_array()
                    .ok_or_else(|| 
                    format!("Error when try to get the 'columns' field in {}",table))?;
                
                for column in columns {
                    let column_name = String::from(column["column_name"].as_str()
                        .ok_or_else(|| format!("Error when try to get the 'column_name' field in {}",column))?);
                    let function_name = format!("{}_{}",label.to_lowercase(),column_name);
                    match &column["foreign_key"] {
                        Value::Null => {
                            let pg_data_type = column["data_type"].as_str()
                                .ok_or_else(|| format!("Error when try to get the 'data_type' field in {}",column))?;
                            let data_type = Neo4j::convert_postgresql_type(pg_data_type)
                                .map_err(|error| format!("{}",error))?;

                            if let Some(value) = column["primary_key"].as_bool() {
                                if value == true {
                                    constraints_content.push_str(&format!("create constraint unique_{} if not exists for (n:{}) require n.{} is unique;\n",
                                    function_name,label,column_name));
                                }
                            }
                            if let Some(value) = column["is_nullable"].as_str() {
                                if value == "NO" {
                                    constraints_content.push_str(&format!("create constraint nonull_{} if not exists for (n:{}) require n.{} is not null;\n",
                                    function_name,label,column_name));
                                }
                            }
                            triggers_content.push_str(&format!(r#"CALL apoc.trigger.add('type_{}',"MATCH (m:{}) WHERE m.{} IS NOT NULL AND NOT valueType(m.{}) = '{}' CALL apoc.util.validate(true, 'ERROR : The type of the field {} need to be a {} .', []) RETURN m",{{phase: 'before'}});
                            "#,function_name,label,column_name,column_name,data_type,column_name,data_type));
                            headers.push_str(&format!("{}:{};",column_name,data_type));
                        },
                        Value::Array(vector) => {
                            let key = String::from(vector[0]["referenced_table"].as_str()
                                .ok_or_else(|| format!("Error when try to get the 'referenced_table' field in {}",vector[0]))?)
                                .to_uppercase();
                            let column_ref_name = String::from(vector[0]["referenced_column"].as_str()
                                .ok_or_else(|| format!("Error when try to get the 'referenced_column' field in {}",vector[0]))?);
                            if !foreign_keys.contains(&key) {
                                foreign_keys.push(String::clone(&key));
                            }
                            fk_content.push_str(&format!("{}_REF_{};{};{}\n",label,key,column_name,column_ref_name));
                        },
                        _ => { return Err(format!("Error when try to match the 'foreign_keys' field in {}", column)); }
                    }
                }
                
                headers.push_str(":LABEL\n");
                let file_path = format!("{}{}.csv",db_neo4j.get_import_folder(),label);
                let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&file_path)
                    .map_err(|error| format!("{}",error))?;
                match file.write_all(&headers.as_bytes()) {
                    Ok(_) => println!("\nSuccessfully write the headers in {}\n",file_path),
                    Err(error) => { return Err(format!("{}",error)); }
                }

                const HEADERS_FK:&str = ":START_ID;:END_ID;:TYPE";
                for fk in foreign_keys {
                    let file_path = format!("{}{}_REF_{}.csv",db_neo4j.get_import_folder(),label,fk);
                    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&file_path)
                        .map_err(|error| format!("{}",error))?;
                    match file.write_all(HEADERS_FK.as_bytes()) {
                        Ok(_) => println!("\nSuccessfully write the fk headers in {}\n",file_path),
                        Err(error) => { return Err(format!("{}",error)); }
                    }
                }
            }
            let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&constraints_path)
                .map_err(|error| format!("{}",error))?;
            match file.write_all(constraints_content.as_bytes()) {
                Ok(_) => {
                    match db_neo4j.execute_script(&constraints_path) {
                        Ok(_) => println!("\nSuccessfully create and run the Cypher script : {}",constraints_path),
                        Err(error) => { return Err(format!("{}",error)); }
                    }
                },
                Err(error) => { return Err(format!("{}",error)); }
            }
            let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&triggers_path)
                .map_err(|error| format!("{}",error))?;
            match file.write_all(triggers_content.as_bytes()) {
                Ok(_) => {
                    match db_neo4j.execute_script(&triggers_path) {
                        Ok(_) => println!("\nSuccessfully create and run the Cypher script : {}",triggers_path),
                        Err(error) => { return Err(format!("{}",error)); }
                    }
                },
                Err(error) => { return Err(format!("{}",error)); }
            }
            let mut file = OpenOptions::new().write(true).create(true).truncate(true).open("./Neo4j/FK.csv")
                .map_err(|error| format!("new {}",error))?;
            match file.write_all(&fk_content.as_bytes()) {
                Ok(_) => println!("\nSuccessfully write the ./Neo4j/postgresql_fk.csv file."),
                Err(error) => { return Err(format!("nEW {}",error)); }
            }
            return Ok("\nSuccessfully create and write the Headers for the Neo4j import.".to_string());
        },
        _ => { return Err(format!("Expected a Value::Object(Map<_,_>) but found :\n{}",json_object)) }
    }
}

fn extract_nodes(db_neo4j: &Neo4j,tables_folder:&str) -> Result<String, String> {
    //! Read the JSON file that contains all the lines of the PostgreSQL database and save them <br>
    //! in the CSV files in the the import folder. <br><br>
    //! **WARNING** this method need to be used after ```&self.extract_csv_headers(...)```
    
    let path = Path::new(tables_folder);
    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry_) = entry {
                    let file_name = entry_.file_name().into_string().unwrap_or_default();
                    if file_name.ends_with(".csv") {
                        let mut label = file_name.to_uppercase();
                        label.truncate(label.len()-4);
                        let headers = fs::read_to_string(format!("{}{}.csv",
                            db_neo4j.get_import_folder(),label))
                            .map_err(|error| format!("ERROR : when try to read the file : {}{}.csv\n{}",db_neo4j.get_import_folder(),label,error))?;
                        let headers = headers.split(";").map(|c| c.split(":")
                            .collect::<Vec<&str>>()[0]).collect::<Vec<&str>>();
                        let headers = headers.iter().skip(1).take(headers.len() - 2)
                            .cloned().collect::<Vec<&str>>();

                        let df = CsvReadOptions::default()
                            .with_has_header(true)
                            .try_into_reader_with_file_path(Some(entry_.path()))
                            .map_err(|e| format!("{}",e))?
                            .finish()
                            .map_err(|e| format!("{}",e))?;

                        let mut df = df.select(headers.clone())
                            .map_err(|e| format!("ERROR : when try to filter the Dataframe with the columns '{:#?}' from the file {}\n{:?}",
                            headers,file_name,e))?;
                        
                        let index_series = Series::new("index_number".into(), (0..df.height() as u64).collect::<Vec<u64>>());
                        let df = df.insert_column(0, index_series)
                            .map_err(|e| format!("ERROR : when try to insert the index column in {}\n{}",file_name,e))?;

                        let label_series = Series::new("line_number".into(), 
                            (0..df.height()).map(|c| String::clone(&label)).collect::<Vec<String>>());
                        let mut df = df.with_column(label_series)
                            .map_err(|e| format!("ERROR : when try to insert the label column in {}\n{}",file_name,e))?;

                        let path_destination = format!("{}{}.csv",db_neo4j.get_import_folder(),label);

                        let mut file = OpenOptions::new().write(true).create(false).append(true).truncate(false).open(&path_destination)
                            .map_err(|error| format!("{}",error))?;

                        if let Err(error) = CsvWriter::new(&mut file)
                            .include_header(false).with_separator(b';')
                            .finish(&mut df)
                        {
                            return Err(format!("ERROR : when try to write the Dataframe of {}\n{}",file_name,error))
                        }
                    }
                }
            }
        }
        Err(error) => { return Err(format!("{}",error)); }
    }
    Ok("\nSuccessfully extract the nodes and store them in the CSV files !".to_string())
}

fn extract_edges(db_neo4j: &Neo4j,foreign_key_path:&str) -> Result<String, String> {
    //! Read the JSON file that contains all the couple of foreign keys of the PostgreSQL database <br>
    //! and save them in the CSV files in the the import folder. <br><br>
    //! **WARNING** this method need to be used after ```&self.extract_csv_headers(...)```
    let content = fs::read_to_string(foreign_key_path)
        .map_err(|error| format!("{}",error))?;

    let json_object:Value = serde_json::from_str(&content)
        .map_err(|error| format!("{}",error))?;

    let map = json_object.as_object()
        .ok_or_else(|| format!("Error when try to parse into a Map this object :\n{}",json_object))?;

    for (file_name, lines) in map {
        let labels = file_name.split("_REF_").collect::<Vec<&str>>();
        let mut content = String::new();
        if let Some(vector) = lines.as_array() {
            for couple in vector {
                if let Some(duo) = couple.as_array() {
                    content.push_str(&format!("\n{}{};{}{};{}",
                        labels[0],duo[0],labels[1],duo[1],file_name
                    ));
                }
            }
        }

        let file_path = format!("{}{}.csv",db_neo4j.get_import_folder(),file_name);
        let mut file = OpenOptions::new().write(true).append(true).create(true).open(&file_path)
            .map_err(|error| format!("{}",error))?;
        match file.write_all(&content.as_bytes()) {
            Ok(_) => println!("\nSuccessfully write the edges in {}\n",file_path),
            Err(error) => { return Err(format!("{}",error)); }
        }
    }
    Ok("Successfully extract the edges and store them in the CSV files !".to_string())
}

pub fn generate_import_files(db_neo4j: &Neo4j, meta_data_path:&str, tables_folder:&str) -> Result<String, String> {
    //! This function generate the files needed to do the import to Neo4J. These files store the database in CSV files in the import folder of the Neo4j object.
    match create_csv_headers(db_neo4j, meta_data_path) {
        Ok(res) => {
            println!("{}",res);
            match extract_nodes(db_neo4j, tables_folder) {
                Ok(res) => {
                    println!("{}",res);
                    Ok(res)
                    /* 
                    match extract_edges(db_neo4j, foreign_key_path) {
                        Ok(res) => {
                           println!("{}\n\nThe files to do the import are ready. You can stop your neo4j database and use the function 'load_with_admin()'.",res);
                           Ok(res) 
                        },
                        Err(error) => Err(error)
                    }*/
                },
                Err(error) => Err(error)
            }
        },
        Err(error) => Err(error)
    }
}