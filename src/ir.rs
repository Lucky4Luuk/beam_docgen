use crate::data;

#[derive(Debug)]
pub struct Table {
    pub full_name: String,
    pub name: String,
    pub children: Vec<Table>,
    pub functions: Vec<Function>,
}

impl Table {
    pub fn from_data_table(parent: &str, node: data::Table) -> Self {
        let full_name = format!("{parent}/{}", node.docs_name);

        let mut children = Vec::new();
        let mut functions = Vec::new();

        for (_name, child) in node.params {
            match child {
                data::Node::Table(t) => children.push(Self::from_data_table(&full_name, t)),
                data::Node::Function(f) => {
                    functions.push(Function::from_data_function(&full_name, f))
                }
                _ => {}
            }
        }

        Self {
            full_name,
            name: node.docs_name,
            children,
            functions,
        }
    }
}

#[derive(Debug)]
pub struct Function {
    pub full_name: String,
    pub name: String,
    pub source: String,
}

impl Function {
    // TODO: This should use the source and active lines data to
    //       look at the function definition and search for callers and such
    pub fn from_data_function(parent: &str, func: data::Function) -> Self {
        let full_name = format!("{parent}/{}", func.docs_name);
        Self {
            full_name,
            name: func.docs_name,
            source: func.source,
        }
    }
}
