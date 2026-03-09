# Neo4j-Migrator 💾

## What's Neo4j-Migrator ?

Neo4j-Migrator is a powerfull tool that permite you to **migrate** a **relationnal database** into a **graph database**.
With the simple access of your relationnal database it export the data/meta-data needed for the migration. After export the data into CSV files and the meta-data into JSON file, Neo4j-Migrator use them to translate your relationnal database into a graph database and generate **CSV** files. You could use these files to perform import to your Neo4j database.

## Getting started

1) Check the [Requirements](https://github.com/LugolBis/Neo4j-Migrator#requirements)
2) Configure your Neo4j database
3) Install **Neo4j-Migrator** CLI :
   ```BashScript
   $ cargo install --git https://github.com/LugolBis/MyShortcuts.git
   ```
4) Start your **PostgreSQL** and **complete**/run :
   ```BashScript
   neo4j-migrator \
      -pg_host=localhost \
      -pg_port=5432 \
      -pg_user=your_postgres_user \
      -pg_password=your_password \
      -pg_database=target_postgres_db \
      -neo4j_database=target_neo4j_db \
      -neo4j_import_folder=/path/to/neo4j/your_db/import/
   ```

   ![NOTE] : If needed you can use `-work_folder=` argument to set the folder used by **Neo4j-Migrator** (default value is `./neo4j_migrator/`).
5) You can now use `cypher-shell` or anything else who's better (not hard to find) to execute generated Cypher scripts `constraints.cql` and `triggers.cql`.

## Requirements

### PostgreSQL

- A valid connection to a **PostgreSQL** database (address,port,username,etc.)

<br>

| Operating System | Relationnal Database | Graph Database | Compatibility |
|:-:|:-:|:-:|:-:|
| Linux/macOS | PostgreSQL | Neo4j | ✅ |
| other~ | other~ | other~ | ❔ |
