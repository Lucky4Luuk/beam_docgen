use crate::ir::*;

pub fn gen_md(root: Table) -> String {
    let Table {
        children,
        functions,
        ..
    } = root;
    let mut children_content = String::new();
    for child in children {
        children_content.push_str("<li>");
        children_content.push_str(&table_to_md(&child));
        children_content.push_str("</li>");
    }
    let mut functions_content = String::new();
    for func in functions {
        functions_content.push_str("<li>");
        functions_content.push_str(&function_to_md(&func));
        functions_content.push_str("</li>");
    }
    format!("<ul>\n{children_content}\n{functions_content}\n</ul>")
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
        children_content.push_str("<li>");
        children_content.push_str(&table_to_md(child));
        children_content.push_str("</li>");
    }
    let mut functions_content = String::new();
    for func in functions {
        functions_content.push_str("<li>");
        functions_content.push_str(&function_to_md(func));
        functions_content.push_str("</li>");
    }
    format!(
        "<details><summary>{full_name}</summary>
<ul>
{children_content}
{functions_content}
</ul>
</details>
"
    )
}

fn function_to_md(function: &Function) -> String {
    let Function {
        full_name,
        name,
        source,
        func_def,
        func_def_line,
    } = function;
    format!(
        "<details><summary>function {full_name}</summary><pre><code class=\"language-lua\">
-- @/{source}:{func_def_line}
{func_def}
</code></pre></details>
"
    )
}
