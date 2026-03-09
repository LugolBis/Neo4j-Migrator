use neo4j_migrator::cli;
use polars::prelude::IntoVec;
use std::env::args;

#[tokio::main]
async fn main() {
    let pl_vec = args().into_vec();
    let args = pl_vec.iter().map(|a| a.as_str()).collect::<Vec<&str>>();

    match cli::main(args).await {
        Ok(_) => println!("\n\nSuccessfully migrate the database to Neo4j !"),
        Err(error) => println!("{}", error),
    }
}
