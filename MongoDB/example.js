// Connect to the MongoDB database
// To start the database : $ sudo systemctl start mongod
// You could easily execute this script with the command : $ mongosh -f 'path/to/test.js'

db.Livres.insertOne({
    livre_id : 0,
    titre : "Voyage au centre de la Terre",
    auteur_id : 0,
    date_publication : "1864-11-25",
    isbn : 9782070331537,
    categorie_id : 12,
    editeur_id : 0
})

db.Editeurs.insertMany([
    {
        editeur_id : 0,
        nom : 'Éditions Gallimard',
        adresse : '35 rue Sébastien Bottin, 75007 Paris, France'
    },
    {
        editeur_id : 1,
        nom : "Penguin Random House",
        adresse : "80 Strand, London WC2R 0RL, Royaume-Uni"
    },
    {
        editeur_id : 2,
        nom : "De Agostini Editore",
        adresse : "Via Giovanni da Verrazano 15, 28100 Novara, Italie"
    }
])

// Add '-5' to the editeur_id field
db.Editeurs.updateMany({}, {$inc: {editeur_id:-5}})

// Change the name of the field "nom"
db.Editeurs.updateMany({}, {$rename: {nom:"name"}})

// Store in variable 'res1' the result of the request
let res1 = db.Editeurs.find({})
// Print all documents in Editeurs
console.log("Collection Editeurs :")
printjson(res1);

console.log("\nUn seul Livre :")
printjson(db.Livres.findOne())

console.log("\n1st Aggregate on Editeurs :")
printjson(db.Editeurs.aggregate([
    {
      $match: { editeur_id: { $lt: -3 } }
    },
    {
      $group: {_id: "FirstAgregate", sumId: { $sum: "$editeur_id" } }
    }
]))

console.log("\n2nd Aggregate on Editeurs :")
printjson(db.Editeurs.aggregate([{$sort:{editeur_id:-1}},{$limit:2}]))

console.log("\nMeta-Data")

printjson(db.getCollectionInfos())

// To export a local database : mongodump -d <database_name> -o <target_directory>