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

    pub fn get_data(&self, mut path: Vec<String>) -> Option<Node> {
        let last = path.pop()?;
        println!("Last: {}", last);
        if path.is_empty() {
            if let Some(f) = self.functions.iter().find(|f| f.name == last) {
                Some(Node::Function(f.clone()))
            } else if let Some(t) = self.children.iter().find(|t| t.name == last) {
                Some(Node::Table(t.clone()))
            } else {
                println!("Couldn't find any end node!");
                None
            }
        } else {
            if let Some(t) = self.children.iter().find(|t| t.name == last) {
                println!("Not a root node so we continue.");
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
        let source = func.source.replace("@/", "");
        let func_def = if func.source.contains("[C]") || func.linedefined < 0 {
            format!("function {}(...)", func.docs_name)
        } else {
            let path = format!("{game_dir}{}", source);
            let func_def_line = func.linedefined.saturating_sub(1);

            std::fs::read_to_string(&path)
                .ok()
                .and_then(|contents| {
                    contents
                        .lines()
                        .nth(func_def_line as usize)
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| format!("function {}(...)", func.docs_name))
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
