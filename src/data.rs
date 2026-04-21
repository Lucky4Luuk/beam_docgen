use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Table {
    #[serde(rename = "__docs_name")]
    docs_name: String,

    #[serde(flatten)]
    params: HashMap<String, Node>,
}

#[derive(Debug, Deserialize)]
pub struct Function {
    #[serde(rename = "__docs_name")]
    docs_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Value {
    #[serde(rename = "__docs_name")]
    docs_name: String,

    v: String,
}

#[derive(Debug, Deserialize)]
pub struct Other {
    #[serde(rename = "__docs_name")]
    docs_name: String,

    kind: String,
}

#[derive(Debug, Deserialize)]
pub struct Cycle {
    #[serde(rename = "__docs_name")]
    docs_name: String,

    name: String,
}

#[derive(Debug, Deserialize)]
pub enum Node {
    Table(Table),
    Function(Function),

    Cycle(Cycle),

    Value(Value),
    Other(Other),
}

/// Every DocGen data dump starts as a table, so that will be our starting point too.
#[derive(Debug, Deserialize)]
pub struct ApiData {
    root: Node,
}
