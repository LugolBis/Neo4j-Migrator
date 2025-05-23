//! This module contains the logic to transform the data from the relationnal database to neo4j data

use std::env;
use std::fs::OpenOptions;
use std::fs::{self, DirEntry};
use std::io::Write;
use std::path::Path;

use polars::prelude::{CsvReadOptions, CsvWriter, Series, Column, DataFrame, StringChunked, IntoColumn, SerWriter, SerReader, NamedFrom, DataFrameJoinOps};
use serde_json::Value;

use crate::neo4j::*;
use crate::utils::*;

const HEADERS_FK: &str = ":START_ID;:END_ID;:TYPE\n";

/// Generate **CSV** files who contains the **HEADERS** needed to generate and organise the
/// data to be imported to Neo4j.
fn process_meta_data(db_neo4j: &Neo4j,meta_data_path: &str,foreign_key_path: &str) -> Result<String, String> {
    if let Err(error) = clean_directory(&db_neo4j.get_import_folder()) {
        return Err(error);
    }

    let content = fs::read_to_string(meta_data_path).map_err(|error| format!("{}", error))?;

    let json_object: Value =
        serde_json::from_str(&content).map_err(|error| format!("{}", error))?;

    let constraints_path = format!(
        "{}/Neo4j/constraints.cql",
        env::current_dir()
            .map_err(|error| format!("{}", error))?
            .display()
    );

    let triggers_path = format!(
        "{}/Neo4j/triggers.cql",
        env::current_dir()
            .map_err(|error| format!("{}", error))?
            .display()
    );

    let mut constraints_content = String::new();
    let mut triggers_content = String::new();
    let mut fk_content = String::new();

    match json_object {
        Value::Array(vector) => {
            for table in vector {
                let label = String::from(table["table_name"].as_str().ok_or_else(|| {
                    format!("Error when try to get the 'table_name' field in {}", table)
                })?)
                .to_uppercase();
                let mut headers = String::from(":ID;");
                let mut foreign_keys: Vec<String> = Vec::new();

                let columns = table["columns"].as_array().ok_or_else(|| {
                    format!("Error when try to get the 'columns' field in {}", table)
                })?;

                process_columns(columns,label.as_str(),&mut constraints_content, &mut triggers_content, &mut headers, &mut foreign_keys, &mut fk_content)?;

                headers.push_str(":LABEL\n");
                let file_path = format!("{}{}.csv", db_neo4j.get_import_folder(), label);
                write_file(headers, &file_path)?;
                println!("\nSuccessfully write the headers in {}\n", file_path);

                for fk in foreign_keys {
                    let file_path = format!("{}{}.csv", db_neo4j.get_import_folder(), fk);
                    write_file(String::from(HEADERS_FK), &file_path)?;
                    println!("\nSuccessfully write the fk headers in {}\n", file_path);
                }
            }

            match write_file(constraints_content, &constraints_path) {
                Ok(_) => match db_neo4j.execute_script(&constraints_path) {
                    Ok(_) => println!(
                        "\nSuccessfully create and run the Cypher script : {}\n",
                        constraints_path
                    ),
                    Err(error) => {
                        return Err(format!("{}", error));
                    }
                },
                Err(error) => {
                    return Err(format!("{}", error));
                }
            }

            match write_file(triggers_content, &triggers_path) {
                Ok(_) => match db_neo4j.execute_script(&triggers_path) {
                    Ok(_) => println!(
                        "\nSuccessfully create and run the Cypher script : {}\n",
                        triggers_path
                    ),
                    Err(error) => {
                        return Err(format!("{}", error));
                    }
                },
                Err(error) => {
                    return Err(format!("{}", error));
                }
            }

            write_file(fk_content, foreign_key_path)?;
            println!("\nSuccessfully write the {} file", foreign_key_path);

            Ok(String::from("\nSuccessfully create and write the Headers for the Neo4j import."))
        }
        _ => {
            Err(format!("Expected a Value::Object(Map<_,_>) but found :\n{}",json_object))
        }
    }
}

/// Process on the meta-data for each column.
fn process_columns(
    columns: &Vec<Value>,
    label: &str,
    constraints_content: &mut String,
    triggers_content: &mut String,
    headers: &mut String,
    foreign_keys: &mut Vec<String>,
    fk_content: &mut String
) -> Result<(), String> {
    for column in columns {
        let column_name =
            String::from(column["column_name"].as_str().ok_or_else(|| {
                format!(
                    "Error when try to get the 'column_name' field in {}",
                    column
                )
            })?);
        let function_name = format!("{}_{}", label.to_lowercase(), column_name);
        match &column["foreign_key"] {
            Value::Null => {
                let pg_data_type = column["data_type"].as_str().ok_or_else(|| {
                    format!("Error when try to get the 'data_type' field in {}", column)
                })?;
                let data_type = Neo4j::convert_postgresql_type(pg_data_type)
                    .map_err(|error| format!("{}", error))?;

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
                triggers_content.push_str(&format!(r#"CALL apoc.trigger.add('type_{}',"MATCH (m:{}) WHERE m.{} IS NOT NULL AND NOT valueType(m.{}) = '{}' CALL apoc.util.validate(true, 'ERROR : The type of the field {} need to be a {} .', []) RETURN m",{{phase: 'before'}});{}"#
                    ,function_name,label,column_name,column_name,data_type,column_name,data_type,"\n"));
                headers.push_str(&format!("{}:{};", column_name, data_type));
            }
            Value::Array(vector) => {
                let key = String::from(
                    vector[0]["referenced_table"].as_str().ok_or_else(|| {
                        format!(
                            "Error when try to get the 'referenced_table' field in {}",
                            vector[0]
                        )
                    })?,
                )
                .to_uppercase();
                let column_ref_name = String::from(
                    vector[0]["referenced_column"].as_str().ok_or_else(|| {
                        format!(
                            "Error when try to get the 'referenced_column' field in {}",
                            vector[0]
                        )
                    })?,
                );
                foreign_keys.push(format!(
                    "{}_ref_{}",
                    label,
                    column_name.to_uppercase()
                ));
                fk_content.push_str(&format!(
                    "{}_ref_{};{};{}\n",
                    label, key, column_name, column_ref_name
                ));
            }
            _ => {
                return Err(format!(
                    "Error when try to match the 'foreign_keys' field in {}",
                    column
                ));
            }
        }
    }
    Ok(())
}

/// This simple function write the ```content``` in the ```file_path```<br>
/// It use the ```OpenOptions``` struct with the following args :<br>
/// write = true ; create = true ; truncate = true
fn write_file(content: String, file_path: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
        .map_err(|error| format!("ERROR : when try to open the follosing file : {}\n {}", file_path, error))?;
    match file.write_all(&content.as_bytes()) {
        Ok(_) => Ok(()),
        Err(error) => Err(format!("ERROR : when try to write in {}\n {}", file_path, error))
    }
}

/// Scan the folder that contains the CSV files that contains the tables imported from the PostgreSQL database<br>
/// and save them in the CSV files in the the import folder. <br><br>
/// **WARNING** : This method need to be used after ```&self.extract_csv_headers(...)```
fn extract_nodes(db_neo4j: &Neo4j, tables_folder: &str) -> Result<String, String> {
    let path = Path::new(tables_folder);
    match fs::read_dir(path) {
        Ok(entries) => {
            let entries = entries
                .filter(|e| e.is_ok())
                .map(|x| x.unwrap())
                .collect::<Vec<DirEntry>>();
            for entry in entries {
                let file_name = entry.file_name().into_string().unwrap_or_default();
                if file_name.ends_with(".csv") {
                    let mut label = file_name.to_uppercase();
                    label.truncate(label.len() - 4);
                    let headers = fs::read_to_string(format!(
                        "{}{}.csv",
                        db_neo4j.get_import_folder(),
                        label
                    ))
                    .map_err(|error| {
                        format!(
                            "ERROR : when try to read the file : {}{}.csv\n{}",
                            db_neo4j.get_import_folder(),
                            label,
                            error
                        )
                    })?;
                    let headers = headers
                        .split(";")
                        .map(|c| c.split(":").collect::<Vec<&str>>()[0])
                        .collect::<Vec<&str>>();
                    let headers = headers
                        .iter()
                        .skip(1)
                        .take(headers.len() - 2)
                        .cloned()
                        .collect::<Vec<&str>>();

                    let df = CsvReadOptions::default()
                        .with_has_header(true)
                        .try_into_reader_with_file_path(Some(entry.path()))
                        .map_err(|e| format!("{}", e))?
                        .finish()
                        .map_err(|e| format!("{}", e))?;

                    let mut df = df.select(headers.clone())
                            .map_err(|e| format!("ERROR : when try to filter the Dataframe with the columns '{:#?}' from the file {}\n{:?}",
                            headers,file_name,e))?;

                    let index_series = Series::new(
                        "neo4j_id_for_import".into(),
                        (0..df.height() as u64)
                            .map(|x| format!("{}{}", label, x))
                            .collect::<Vec<String>>(),
                    );
                    let df = df.insert_column(0, index_series).map_err(|e| {
                        format!(
                            "ERROR : when try to insert the index column in {}\n{}",
                            file_name, e
                        )
                    })?;

                    let label_series = Series::new(
                        "line_number".into(),
                        (0..df.height())
                            .map(|_| String::clone(&label))
                            .collect::<Vec<String>>(),
                    );
                    let mut df = df.with_column(label_series).map_err(|e| {
                        format!(
                            "ERROR : when try to insert the label column in {}\n{}",
                            file_name, e
                        )
                    })?;

                    let path_destination = format!("{}{}.csv", db_neo4j.get_import_folder(), label);

                    let mut file = OpenOptions::new()
                        .create(false)
                        .append(true)
                        .truncate(false)
                        .open(&path_destination)
                        .map_err(|error| format!("{}", error))?;

                    if let Err(error) = CsvWriter::new(&mut file)
                        .include_header(false)
                        .with_separator(b';')
                        .finish(&mut df)
                    {
                        return Err(format!(
                            "ERROR : when try to write the Dataframe of {}\n{}",
                            file_name, error
                        ));
                    }
                }
            }
        }
        Err(error) => {
            return Err(format!("{}", error));
        }
    }
    Ok(String::from(
        "\nSuccessfully extract the nodes and store them in the CSV files !",
    ))
}

/// Read the JSON file that contains all the couple of foreign keys of the PostgreSQL database <br>
/// and save them in the CSV files in the the import folder. <br><br>
/// **WARNING** this method need to be used after ```&self.extract_csv_headers(...)```
fn extract_relationships(db_neo4j: &Neo4j, tables_folder: &str, foreign_key_path: &str) -> Result<String, String> {
    let lines = fs::read_to_string(foreign_key_path).map_err(|error| format!("{}", error))?;
    let lines = lines.split("\n").collect::<Vec<&str>>();

    for line in lines {
        if line != "" {
            let elements = line.split(";").collect::<Vec<&str>>();
            let tables = elements[0].split("_ref_").collect::<Vec<&str>>();
            let table1 = tables[0];
            let table2 = tables[1];
            let column1 = elements[1];
            let column2 = elements[2];
            let label = format!("{}_ref_{}", table1, column1.to_uppercase());

            let mut df1 = CsvReadOptions::default()
                .with_has_header(true)
                .try_into_reader_with_file_path(Some(
                    format!("{}{}.csv", tables_folder, table1.to_lowercase()).into(),
                ))
                .map_err(|e| format!("{}", e))?
                .finish()
                .map_err(|e| format!("{}", e))?;

            let mut df1_id =
                generate_id_column(&df1, table1, "row_id1").map_err(|e| format!("{}", e))?;
            df1_id.rename("row_id1".into());

            let df1 = df1.insert_column(0, df1_id).map_err(|e| format!("{}", e))?;

            let mut df2 = CsvReadOptions::default()
                .with_has_header(true)
                .try_into_reader_with_file_path(Some(
                    format!("{}{}.csv", tables_folder, table2.to_lowercase()).into(),
                ))
                .map_err(|e| format!("{}", e))?
                .finish()
                .map_err(|e| format!("{}", e))?;

            let mut df2_id =
                generate_id_column(&df2, table2, "row_id2").map_err(|e| format!("{}", e))?;
            df2_id.rename("row_id2".into());

            let df2 = df2.insert_column(0, df2_id).map_err(|e| format!("{}", e))?;

            let mut df = df1
                .inner_join(&df2, [column1], [column2])
                .map_err(|e| format!("{}", e))?
                .select(["row_id1", "row_id2"])
                .map_err(|e| format!("{}", e))?;

            let mut df = df
                .with_column(Series::new(
                    "line_number".into(),
                    (0..df.height())
                        .map(|_| String::clone(&label))
                        .collect::<Vec<String>>(),
                ))
                .map_err(|e| {
                    format!(
                        "ERROR : when try to insert the label column in {}\n{}",
                        label, e
                    )
                })?;

            let file_path = format!("{}{}.csv", db_neo4j.get_import_folder(), label);
            let mut file = OpenOptions::new()
                .write(true)
                .create(false)
                .append(true)
                .truncate(false)
                .open(&file_path)
                .map_err(|error| format!("{}", error))?;

            if let Err(error) = CsvWriter::new(&mut file)
                .include_header(false)
                .with_separator(b';')
                .finish(&mut df)
            {
                return Err(format!(
                    "ERROR : when try to write the Dataframe of {}\n{}",
                    file_path, error
                ));
            }
        }
    }
    Ok(String::from(
        "\nSuccessfully extract the edges and store them in the CSV files !",
    ))
}

/// This function generate the files needed to do the import to Neo4J. These files store the database in CSV files in the import folder of the Neo4j object.
pub fn generate_import_files(db_neo4j: &Neo4j,meta_data_path: &str,tables_folder: &str,foreign_key_path: &str) -> Result<String, String> {
    match process_meta_data(db_neo4j, meta_data_path, foreign_key_path) {
        Ok(res) => {
            println!("{}", res);
            match extract_nodes(db_neo4j, tables_folder) {
                Ok(res) => {
                    println!("{}", res);
                    match extract_relationships(db_neo4j, tables_folder, foreign_key_path) {
                        Ok(res) => {
                            println!("{}\n\nThe files to do the import are ready. You can stop your neo4j database and use the function 'load_with_admin()'.",res);
                            Ok(res)
                        }
                        Err(error) => Err(error),
                    }
                }
                Err(error) => Err(error),
            }
        }
        Err(error) => Err(error),
    }
}

/// Generate an 'Id column' necessary for the import of the relationships.<br>
/// It's basically to match the alias names called ID in the format of the csv files<br>
/// for the import to neo4j.
fn generate_id_column(df: &DataFrame, label: &str, column_name: &str) -> Result<Column, String> {
    Ok(df
        .with_row_index(column_name.into(), None)
        .map_err(|e| format!("{}", e))?
        .column(column_name)
        .map_err(|e| format!("{}", e))?
        .u32()
        .map_err(|e| format!("{}", e))?
        .into_iter()
        .map(|opt_name: Option<u32>| opt_name.map(|index: u32| format!("{}{}", label, index)))
        .collect::<StringChunked>()
        .into_column())
}
