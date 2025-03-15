// To retrieve the path of the folders where you need to do some configs :
CALL dbms.listConfig() 
YIELD name, value, description
WHERE name CONTAINS 'directories' OR name CONTAINS 'home'
RETURN name, value, description;

// That Cypher query permite you to retrieve the informations that you need to configure your database
// (Especially the import folder and the neo4j_home folder)
// You can do it manually or with the method "configure_db_on_linux()"