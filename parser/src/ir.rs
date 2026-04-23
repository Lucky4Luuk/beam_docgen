use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::data;

#[derive(Debug, Serialize, Deserialize)]
pub enum Node {
    Table(Table),
    Function(Function),
}

impl Node {
    pub fn name(&self) -> &str {
        match self {
            Self::Table(t) => &t.name,
            Self::Function(f) => &f.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub full_name: String,
    pub name: String,
    pub children: Vec<Table>,
    pub functions: Vec<Function>,
}

impl Table {
    pub fn from_data_table(parent: &str, node: data::Table) -> Self {
        let full_name = format!("{parent}.{}", node.docs_name);

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

    pub fn sort_alphanumerical(&mut self) {
        self.children.sort_by(|a, b| {
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase())
        });
        self.functions.sort_by(|a, b| {
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase())
        });

        for child in &mut self.children {
            child.sort_alphanumerical();
        }
    }

    pub fn get_data(&self, mut path: VecDeque<String>) -> Option<Node> {
        let first = path.pop_front()?;
        if path.is_empty() {
            if let Some(f) = self.functions.iter().find(|f| f.name == first) {
                Some(Node::Function(f.clone()))
            } else if let Some(t) = self.children.iter().find(|t| t.name == first) {
                Some(Node::Table(t.clone()))
            } else {
                None
            }
        } else {
            if let Some(t) = self.children.iter().find(|t| t.name == first) {
                t.get_data(path)
            } else {
                println!("Path is not empty and cannot find {first}!");
                None
            }
        }
    }

    pub fn get_all_functions(&self) -> Vec<(String, String)> {
        let mut funcs: Vec<(String, String)> = self
            .functions
            .iter()
            .map(|f| (f.full_name.clone(), f.name.clone()))
            .collect();

        for child in &self.children {
            funcs.append(&mut child.get_all_functions());
        }

        funcs
    }

    pub fn get_function_mut_by_full_name(&mut self, full_name: &str) -> Option<&mut Function> {
        if let Some(f) = self.functions.iter_mut().find(|f| f.full_name == full_name) {
            return Some(f);
        }

        for child in &mut self.children {
            if let Some(f) = child.get_function_mut_by_full_name(full_name) {
                return Some(f);
            }
        }

        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuncCaller {
    pub line: usize,

    pub surrounding_code: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuncInfo {
    /// Source location
    pub source: String,
    /// The line at which the actual function definition is done (so the actual `function <name>()` line)
    pub func_def_lines: (isize, isize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub full_name: String,
    pub name: String,

    pub info: FuncInfo,

    pub callers: HashMap<String, Vec<usize>>,
}

impl Function {
    pub fn from_data_function(parent: &str, func: data::Function) -> Self {
        let source = func.source.replace("@/", "").replace("@", "");

        let full_name = format!("{parent}.{}", func.docs_name);
        Self {
            full_name,
            name: func.docs_name,

            info: FuncInfo {
                source,
                func_def_lines: (func.linedefined, func.lastlinedefined),
            },

            callers: HashMap::new(),
        }
    }

    pub fn set_calls(&mut self, calls: HashMap<String, Vec<usize>>) {
        self.callers = calls;
    }
}
