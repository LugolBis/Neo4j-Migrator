// Main script that initialize the connection and call the other scripts
const db = connect('mongodb://localhost:27017/bibliotheque');

try {
    load("./MongoDB/init.js");
    load("./MongoDB/example.js")
} catch (error) {
    console.log(`Error : ${error}`);
}