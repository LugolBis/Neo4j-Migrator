//! Note : This is just a demo of translating SQL queries into Cypher queries

use std::collections::HashMap;

use sqlparser::ast::*;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

#[allow(unused)]
/// Return the node from the ```TableFactor``` object.
fn from_table_factor(table_factor: TableFactor,hashmap: &mut HashMap<String, String>) -> Result<(), ()> {
    let mut res_alias = String::new();
    if let TableFactor::Table {
        name,
        alias,
        args,
        with_hints,
        version,
        with_ordinality,
        partitions,
        json_path,
        sample,
        index_hints,
    } = table_factor
    {
        if let Some(table_alias) = alias {
            res_alias.push_str(&table_alias.name.value);
        } else {
            return Err(());
        }
        if let Some(identifier) = name.0[0].clone().as_ident() {
            hashmap.insert(
                res_alias.clone(),
                format!("({}:{})", res_alias, identifier.value.to_uppercase()),
            );
            Ok(())
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

#[allow(unused)]
fn from_join_constraint(join_constraint: JoinConstraint) -> Result<(String, String, String, String), ()> {
    if let JoinConstraint::On(Expr::BinaryOp { left, op, right }) = join_constraint {
        if let Expr::CompoundIdentifier(vector1) = *left {
            if let Expr::CompoundIdentifier(vector2) = *right {
                // Return : alias1 column1 alias2 column2
                Ok((
                    String::clone(&vector1[0].value),
                    String::clone(&vector1[1].value),
                    String::clone(&vector2[0].value),
                    String::clone(&vector2[1].value),
                ))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

#[allow(unused)]
/// Return a tuple that contain the join
fn from_join(join: Join, hashmap: &mut HashMap<String, String>) -> Result<(), ()> {
    match from_table_factor(join.relation, hashmap) {
        Ok(_) => match join.join_operator {
            JoinOperator::Inner(join_constraint) => {
                if let Ok(result) = from_join_constraint(join_constraint) {
                    let node1 = String::clone(hashmap.get(&result.0).ok_or_else(|| ())?);
                    let label1 = node1.split(":").collect::<Vec<&str>>()[1].replace(")", "");
                    let node2 = String::clone(hashmap.get(&result.2).ok_or_else(|| ())?);
                    if let Some(match_clause) = hashmap.get_mut("match") {
                        match_clause.push_str(&format!(
                            "{}-[r{}{}{}:{}_ref_{}]-{},",
                            node1, result.0, result.2, result.3, label1, result.1, node2
                        ));
                    }
                    Ok(())
                } else {
                    Err(())
                }
            }
            JoinOperator::Left(join_constraint) => {
                if let Ok(result) = from_join_constraint(join_constraint) {
                    let node1 = String::clone(hashmap.get(&result.0).ok_or_else(|| ())?);
                    let label1 = node1.split(":").collect::<Vec<&str>>()[1].replace(")", "");
                    let node2 = String::clone(hashmap.get(&result.2).ok_or_else(|| ())?);
                    if let Some(match_clause) = hashmap.get_mut("match") {
                        match_clause.push_str(&format!("{},", node1));
                    }
                    if let Some(match_clause) = hashmap.get_mut("optional match") {
                        match_clause.push_str(&format!(
                            "{}-[r{}{}{}:{}_ref_{}]-{},",
                            node1, result.0, result.2, result.3, label1, result.1, node2
                        ));
                    }
                    Ok(())
                } else {
                    Err(())
                }
            }
            JoinOperator::Right(join_constraint) => {
                if let Ok(result) = from_join_constraint(join_constraint) {
                    let node1 = String::clone(hashmap.get(&result.0).ok_or_else(|| ())?);
                    let label1 = node1.split(":").collect::<Vec<&str>>()[1].replace(")", "");
                    let node2 = String::clone(hashmap.get(&result.2).ok_or_else(|| ())?);
                    if let Some(match_clause) = hashmap.get_mut("match") {
                        match_clause.push_str(&format!("{},", node2));
                    }
                    if let Some(match_clause) = hashmap.get_mut("optional match") {
                        match_clause.push_str(&format!(
                            "{}-[r{}{}{}:{}_ref_{}]-{},",
                            node1, result.0, result.2, result.3, label1, result.1, node2
                        ));
                    }
                    Ok(())
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        },
        Err(_) => Err(()),
    }
}

#[allow(unused)]
fn from_table_with_joins(table_with_joins: TableWithJoins,hashmap: &mut HashMap<String, String>) -> Result<(), ()> {
    match from_table_factor(table_with_joins.relation, hashmap) {
        Ok(_) => {
            for join in table_with_joins.joins {
                if let Err(_) = from_join(join, hashmap) {
                    return Err(());
                }
            }
            Ok(())
        }
        Err(_) => Err(()),
    }
}

#[allow(unused)]
fn from_statement(vector_twj: Vec<TableWithJoins>,hashmap: &mut HashMap<String, String>) -> Result<(), ()> {
    for table_with_joins in vector_twj {
        if let Err(_) = from_table_with_joins(table_with_joins, hashmap) {
            return Err(());
        }
    }
    Ok(())
}

#[allow(unused)]
fn select_select_item(select_item: SelectItem,hashmap: &mut HashMap<String, String>) -> Result<(), ()> {
    match select_item {
        SelectItem::UnnamedExpr(Expr::CompoundIdentifier(vector)) => {
            if let Some(return_clause) = hashmap.get_mut("return") {
                return_clause.push_str(&format!("{}.{},", vector[0].value, vector[1].value));
                Ok(())
            } else {
                Err(())
            }
        }
        _ => Err(()),
    }
}

#[allow(unused)]
fn select_statement(vector_si: Vec<SelectItem>,hashmap: &mut HashMap<String, String>) -> Result<(), ()> {
    for select_item in vector_si {
        if let Err(_) = select_select_item(select_item, hashmap) {
            return Err(());
        }
    }
    Ok(())
}

#[allow(unused)]
pub fn generate_cypher_query(sql_query: &str) -> Result<String, String> {
    let dialect = GenericDialect {}; // or AnsiDialect
    let ast = Parser::parse_sql(&dialect, sql_query).unwrap();

    match &ast[0] {
        Statement::Query(query) => {
            let query = query.clone();
            let body = query.body;
            match *body {
                SetExpr::Select(select) => {
                    let mut hashmap: HashMap<String, String> = HashMap::new();
                    hashmap.insert(String::from("match"), String::new());
                    hashmap.insert(String::from("optional match"), String::new());
                    hashmap.insert(String::from("return"), String::new());
                    if let Err(_) = from_statement(select.from, &mut hashmap) {
                        return Err(String::from(
                            "ERROR : when try to transform the 'FROM' clause.",
                        ));
                    }
                    if let Err(_) = select_statement(select.projection, &mut hashmap) {
                        return Err(String::from(
                            "ERROR : when try to transform the 'SELECT' clause.",
                        ));
                    }
                    let mut result = String::from("match ");
                    result.push_str(hashmap.get("match").unwrap());
                    result.pop();
                    let optional_match = hashmap.get("optional match").unwrap();
                    if optional_match != "" {
                        result.push_str(&format!(" optional match {}", optional_match));
                        result.pop();
                    }
                    result.push_str(&format!(" return {}", hashmap.get("return").unwrap()));

                    const KEYWORDS: [&str; 3] = ["match", "optional match", "return"];
                    let mut others_relations = String::new();
                    for (key, value) in hashmap.iter() {
                        if !KEYWORDS.contains(&String::clone(key).as_str())
                            && !result.contains(value)
                        {
                            println!("\nValue : {value}");
                            others_relations.push_str(&format!("{},", value));
                        }
                    }
                    others_relations.pop();
                    result.insert_str(5, &format!(" {} ", others_relations));
                    result.pop();
                    result.push_str(";");

                    return Ok(result);
                }
                _ => {
                    return Err(String::from(
                        "Your query is not yet supported by the funtion.",
                    ))
                }
            }
        }
        _ => Err(String::from("This function only support SQL queries.")),
    }
}

#[test]
fn test_ast_sql() {
    let query = r#"SELECT * from toto;"#;
    let dialect = GenericDialect {}; // or AnsiDialect
    let ast = Parser::parse_sql(&dialect, query).unwrap();
    println!("AST: {:#?}", ast);
}

#[test]
fn test_generation() {
    let sql_query = r#"SELECT t.order from toto t left join juju j on t.order=j.id;"#;
    let cypher_query = generate_cypher_query(sql_query).unwrap();
    println!("\nSQL : {}\nCypher : {}",sql_query,cypher_query);

    let sql_query = r#"SELECT t.order from toto t inner join juju j on t.order=j.id;"#;
    let cypher_query = generate_cypher_query(sql_query).unwrap();
    println!("\nSQL : {}\nCypher : {}",sql_query,cypher_query);
}