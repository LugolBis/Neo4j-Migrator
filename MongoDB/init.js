// Script to delete the content of the database

// Drop each Collection
db.getCollectionNames().forEach(function(c) { if (c.indexOf("system.") == -1) db[c].drop(); })

// To only delete the documents of these collections :
// db.Editeurs.deleteMany({})

// Create Type constraint
db.createCollection( "Editeurs",
   { validator: { $or:
      [
         { editeur_id: { $type: "number" } },
         { nom: { $type: "string" } },
         { adresse: { $type: "string" } }
      ]
   }
})

// Create Unique constraint 
db.Editeurs.createIndex( { editeur_id : 1 }, { unique: true } )

db.createCollection("Livres",
   {
      validator : {
         $or : [
            {livre_id : { $type : "number" }},
            {auteur_id : { $type : "number" }},
            {date_publication : { $type : "string" }},
            {isbn : { $type : "number" }},
            {categorie_id : { $type : "number" }},
            {categorie_id : { $type : "number" }},
         ]
      }
   }
)

db.Livres.createIndex( { livre_id : 1 }, { unique: true } )