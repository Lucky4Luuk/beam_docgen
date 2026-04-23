use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Table {
    #[serde(rename = "__docs_name")]
    pub docs_name: String,

    #[serde(flatten)]
    pub params: HashMap<String, Node>,
}

#[derive(Debug, Deserialize)]
pub struct Function {
    #[serde(rename = "__docs_name")]
    pub docs_name: String,
    pub source: String,
    pub linedefined: isize,
    pub lastlinedefined: isize,
}

#[derive(Debug, Deserialize)]
pub struct Value {
    #[serde(rename = "__docs_name")]
    pub docs_name: String,

    pub v: String,
}

#[derive(Debug, Deserialize)]
pub struct Other {
    #[serde(rename = "__docs_name")]
    pub docs_name: String,

    pub kind: String,
}

#[derive(Debug, Deserialize)]
pub struct Cycle {
    #[serde(rename = "__docs_name")]
    pub docs_name: String,

    pub name: String,
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
    pub root: Node,
}

impl ApiData {
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let s: Self = serde_json::from_str(json)?;
        Ok(s)
    }
}
