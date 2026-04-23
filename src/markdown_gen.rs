use crate::ir::*;

const TABLE_TEMPLATE: &'static str = include_str!("../assets/table.md");
const FUNC_TEMPLATE: &'static str = include_str!("../assets/function.md");

pub fn gen_page_md(node: Node) -> String {
    match node {
        Node::Table(t) => gen_table_page_md(t),
        Node::Function(f) => gen_function_page_md(f),
    }
}

pub fn gen_table_page_md(table: Table) -> String {
    let Table {
        full_name,
        name,
        children,
        functions,
    } = table;

    let mut children_content = Vec::new();
    for child in children {
        let link = child.full_name.replace(".", "/");
        children_content.push(format!("- [`{}`]({})", child.name, link));
    }
    let mut functions_content = Vec::new();
    for func in functions {
        let link = func.full_name.replace(".", "/");
        functions_content.push(format!("- [`function {}`]({})", func.name, link));
    }

    TABLE_TEMPLATE
        .replace("{{TABLE_NAME}}", &name)
        .replace("{{TABLE_CHILDREN}}", &children_content.join("\n"))
        .replace("{{TABLE_FUNCTIONS}}", &functions_content.join("\n"))
}

pub fn gen_function_page_md(function: Function) -> String {
    let Function {
        full_name,
        name,
        source,
        func_def,
        func_def_line,
    } = function;

    FUNC_TEMPLATE.replace("{{FUNCTION_NAME}}", &name)
}
