# Neo4j-Migrator 💾

## What's Neo4j-Migrator ?

Neo4j-Migrator is a powerfull tool that permite you to **migrate** a **relationnal database** into a **graph database**.
With the simple access of your relationnal database it export the data/meta-data needed for the migration. After export the data into CSV files and the meta-data into JSON file, Neo4j-Migrator use them to translate your relationnal database into a graph database and generate **CSV** files. You could use these files to perform import to your Neo4j database.

## Getting started

1) Check the [Requirements](https://github.com/LugolBis/Neo4j-Migrator#requirements)
2) Configure your Neo4j database
3) Install **Neo4j-Migrator** :
   ```BashScript
   $ git clone https://github.com/LugolBis/Neo4j-Migrator.git
   ```
4) Start your Neo4j database and run **Neo4j-Migrator** :
   ```BashScript
   $ cargo run
   ```

## Requirements

### PostgreSQL

- A valid connection to a **PostgreSQL** database (address,port,username,etc.)
- The PostgreSQL CLI : **psql**

### Neo4j

- A valid connection to a **Neo4j** database (uri,username,password,etc.)
- The Neo4j CLI : **Cypher-Shell**
- The plugin **APOC**

> [!WARNING]
> You need to configure your Neo4j database to add the **APOC** plugin and allow it in your database files configuration.

<br>

| Operating System | Relationnal Database | Graph Database | Plugin | Compatibility |
|:-:|:-:|:-:|:-:|:-:|
| Linux/macOS | PostgreSQL | Neo4j **v5.26.0** | APOC **v5.26.2** | ✅ |
| other~ | other~ | other~ | other~ | ❔ |
