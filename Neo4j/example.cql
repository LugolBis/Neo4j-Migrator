// This file contains Cypher queries used during experimentation or to explain how DB-Migrator work

// The cypher queries generated and stored in 'constraints.cql' look like this :

// Constraint to assert the property is unique (like a primary key)
CREATE CONSTRAINT unique_editeur_id IF NOT EXISTS
FOR (e:Editeur) 
REQUIRE e.id IS UNIQUE;

// Constraint to assert the property isn't null
CREATE CONSTRAINT nonull_editeur_id IF NOT EXISTS 
FOR (e:Editeur) 
REQUIRE e.id IS not null;

// The cypher queries generated and stored in 'triggers.cql' look like this :

// Use the APOC.tigger
// WARNING : you need to configure APOC -- For more details read 'config.cql'
CALL apoc.trigger.add(
  'type_LABEL_property',
  "MATCH (m:Movie) 
   WHERE m.year IS NOT NULL AND NOT valueType(m.year) = 'LONG'
   CALL apoc.util.validate(true, 'ERREUR : La propriété year doit être un entier.', [])
   RETURN m",
  {phase: 'before'}
);

// How about import ?! Here different way to import your data :

// With LOAD CSV :
// Note that you could apply this with local files and add "WITH HEADERS" for more control
// DEPRECATED
LOAD CSV FROM 'https://cdn.jsdelivr.net/gh/LugolBis/DB-Migrator@master/Data/editeurs.csv' AS line
WITH line
WHERE size(line) >= 3 AND line[0] <> ''
MERGE (e:Editeur {id: toInteger(line[0])})
SET e.nom = line[1], e.adresse = line[2]
RETURN count(e);

// With APOC :
CALL apoc.periodic.iterate(
  'CALL apoc.load.json("file:///big_data.json") YIELD value RETURN value',
  'CREATE (n:Node) SET n = value',
  {batchSize: 10000, parallel: true}
);

// With the shell command : neo4j-admin database import
// This is the solution choosed to import the data in DB-Migrator

// Example of manual creation
CREATE (charlie:Person:Actor {name: 'Charlie Sheen'}),
       (martin:Person:Actor {name: 'Martin Sheen'}),
       (michael:Person:Actor {name: 'Michael Douglas'}),
       (oliver:Person:Director {name: 'Oliver Stone'}),
       (rob:Person:Director {name: 'Rob Reiner'}),
       (wallStreet:Movie {title: 'Wall Street'}),
       (charlie)-[:ACTED_IN {role: 'Bud Fox'}]->(wallStreet),
       (martin)-[:ACTED_IN {role: 'Carl Fox'}]->(wallStreet),
       (michael)-[:ACTED_IN {role: 'Gordon Gekko'}]->(wallStreet),
       (oliver)-[:DIRECTED]->(wallStreet),
       (thePresident:Movie {title: 'The American President'}),
       (martin)-[:ACTED_IN {role: 'A.J. MacInerney'}]->(thePresident),
       (michael)-[:ACTED_IN {role: 'President Andrew Shepherd'}]->(thePresident),
       (rob)-[:DIRECTED]->(thePresident);

match (a1:Actor {name: 'Charlie Sheen'})
match (b1:Actor {name: 'Martin Sheen'})
with [a1, b1] AS FK
call apoc.util.validate(
    any(node in FK where node is null),
    'Issue with the foreign keys : %s', 
    [FK]
)
create (lulu: Person {name: 'Oliver Stone'})
FOREACH (
node in FK |
create (lulu)-[r:REF_]->(node)
SET r.type = 'REF_' + apoc.node.labels(node)[0]
)
return lulu;

match (n) return n;

// Example of find a specific path
match path = (l:Person{name: 'Oliver Stone'})-[]->(n) return path;