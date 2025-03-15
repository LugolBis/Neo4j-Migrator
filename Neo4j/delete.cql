// Reset the database

// Delete the Label from each node
match (n)
unwind labels(n) as label
remove n:`label`
return n;

// Delete all the nodes and their arcs
match (n)
detach delete n;

// Delete all constraints
call apoc.schema.assert({}, {});

// To delete all the triggers
CALL apoc.trigger.removeAll();