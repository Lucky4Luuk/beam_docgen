use std::collections::VecDeque;

use crate::data;

#[derive(Debug)]
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

#[derive(Debug, Clone)]
pub struct Table {
    pub full_name: String,
    pub name: String,
    pub children: Vec<Table>,
    pub functions: Vec<Function>,
}

impl Table {
    pub fn from_data_table(game_dir: &str, parent: &str, node: data::Table) -> Self {
        let full_name = format!("{parent}.{}", node.docs_name);

        let mut children = Vec::new();
        let mut functions = Vec::new();

        for (_name, child) in node.params {
            match child {
                data::Node::Table(t) => {
                    children.push(Self::from_data_table(game_dir, &full_name, t))
                }
                data::Node::Function(f) => {
                    functions.push(Function::from_data_function(game_dir, &full_name, f))
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
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub full_name: String,
    pub name: String,
    pub source: String,

    pub func_def: String,
    pub func_def_line: isize,
}

impl Function {
    pub fn from_data_function(game_dir: &str, parent: &str, func: data::Function) -> Self {
        let source = func.source.replace("@/", "").replace("@", "");
        let func_def = if func.source.contains("[C]") || func.linedefined < 0 {
            format!("function {}(...)", func.docs_name)
        } else {
            let path = format!("{game_dir}{}", source);
            let func_def_line = func.linedefined.saturating_sub(1);

            std::fs::read_to_string(&path)
                .and_then(|contents| {
                    contents
                        .lines()
                        .nth(func_def_line as usize)
                        .map(|s| s.to_string())
                        .ok_or(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "could not find function definition line {} in {}",
                                func_def_line + 1,
                                path
                            ),
                        ))
                })
                .map_err(|e| match e.kind() {
                    std::io::ErrorKind::NotFound => String::from("source file not found"),
                    std::io::ErrorKind::PermissionDenied => {
                        String::from("permission denied reading")
                    }
                    std::io::ErrorKind::InvalidData => e.to_string(),
                    std::io::ErrorKind::UnexpectedEof => String::from("unexpected eof"),
                    _ => format!("failed to read {}", e),
                })
                .unwrap_or_else(|e| {
                    format!(
                        "function {}(...) -- (err reading file: {})",
                        func.docs_name, e
                    )
                })
        };

        let full_name = format!("{parent}.{}", func.docs_name);
        Self {
            full_name,
            name: func.docs_name,
            source,

            func_def,
            func_def_line: func.lastlinedefined,
        }
    }
}
