use crate::ir::*;

pub fn gen_md(root: Table) -> String {
    let Table {
        children,
        functions,
        ..
    } = root;
    let mut children_content = String::new();
    for child in children {
        children_content.push_str("<ul>");
        children_content.push_str(&table_to_md(&child));
        children_content.push_str("</ul>");
    }
    let mut functions_content = String::new();
    for func in functions {
        functions_content.push_str("<ul>");
        functions_content.push_str(&function_to_md(&func));
        functions_content.push_str("</ul>");
    }
    format!("<li>\n{children_content}\n{functions_content}\n</li>")
}

fn table_to_md(table: &Table) -> String {
    let Table {
        full_name,
        name,
        children,
        functions,
    } = table;
    let mut children_content = String::new();
    for child in children {
        children_content.push_str("<ul>");
        children_content.push_str(&table_to_md(child));
        children_content.push_str("</ul>");
    }
    let mut functions_content = String::new();
    for func in functions {
        functions_content.push_str("<ul>");
        functions_content.push_str(&function_to_md(func));
        functions_content.push_str("</ul>");
    }
    format!(
        "<details><summary>{full_name}</summary>
<li>
{children_content}
{functions_content}
</li>
</details>
"
    )
}

fn function_to_md(function: &Function) -> String {
    let Function {
        full_name, name, ..
    } = function;
    format!(
        "<details><summary>function {full_name}</summary><pre><code>
function {name}
</pre></code></details>
"
    )
}
