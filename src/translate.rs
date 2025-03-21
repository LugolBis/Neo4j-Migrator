// This Module is for translating simple SQL queries into Cypher queries
use sqlparser::tokenizer::*;
use sqlparser::dialect::GenericDialect;

pub fn generate_cypher_query(sql_query:&str) {
    todo!();

    let dialect = GenericDialect {};
    let tokens = Tokenizer::new(&dialect, &sql_query).tokenize().unwrap();
    
    let mut cypher_query = String::new();
    for token in tokens {
        println!("{}",token)
    }
}

#[test]
pub fn test_tokenizer_sql() {
    let query = r#"SELECT l.order from toto"#;
    let dialect = GenericDialect {};
    let tokens = Tokenizer::new(&dialect, &query).tokenize().unwrap();
    for token in tokens {
        println!("{:?}",token)
    }
}