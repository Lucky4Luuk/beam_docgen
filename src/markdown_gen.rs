use std::collections::HashMap;

use parser::{ir::*, lookup::CodeFile};

const TABLE_TEMPLATE: &str = include_str!("../assets/table.md");
const FUNC_TEMPLATE: &str = include_str!("../assets/function.md");

pub fn gen_page_md_template(
    func_template: &str,
    node: Node,
    code: &HashMap<String, CodeFile>,
) -> String {
    match node {
        Node::Table(t) => gen_table_page_md(t),
        Node::Function(f) => gen_function_page_md(func_template, f, code),
    }
}

pub fn gen_page_md(node: Node, code: &HashMap<String, CodeFile>) -> String {
    gen_page_md_template(FUNC_TEMPLATE, node, code)
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
        .replace("{{TABLE_FULL_NAME}}", &full_name)
        .replace("{{TABLE_CHILDREN}}", &children_content.join("\n"))
        .replace("{{TABLE_FUNCTIONS}}", &functions_content.join("\n"))
}

pub fn gen_function_page_md(
    func_template: &str,
    function: Function,
    code: &HashMap<String, CodeFile>,
) -> String {
    let Function {
        full_name,
        name,
        info,
        callers,
    } = function;

    let func_def = if let Some(cf) = code.get(&info.source)
        && info.func_def_lines.0 >= 0
    {
        cf.get_func_def_with_comments(
            info.func_def_lines.0 as usize,
            info.func_def_lines.1 as usize,
        )
        .join("\n")
    } else {
        format!("function {name}(...)")
    };

    let mut func_callers = Vec::new();
    for (src, line_numbers) in &callers {
        if let Some(cf) = code.get(src) {
            let mut s = vec![format!("<details><summary>@/{src}</summary>")];
            for i in line_numbers {
                if src == &info.source && (*i as isize) == info.func_def_lines.0 {
                    continue;
                }
                let call_code = cf.get_section(i - 1, i + 1).join("\n");
                s.push(format!(
                    "<pre><code class=\"highlight-lua\" data-ln-start-from=\"{}\">{call_code}</code></pre>", i - 1
                ));
            }
            if s.len() > 1 {
                s.push(String::from("</details>"));
                func_callers.push(s.join("\n"));
            }
        }
    }

    func_template
        .replace("{{FUNC_NAME}}", &name)
        .replace("{{FUNC_FULL_NAME}}", &full_name)
        .replace("{{FUNC_SRC}}", &info.source)
        .replace("{{FUNC_DEF}}", &func_def)
        .replace("{{FUNC_DEF_LINE}}", &info.func_def_lines.0.to_string())
        .replace("{{FUNC_CALLERS}}", &func_callers.join("\n"))
}
