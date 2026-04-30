use std::collections::HashMap;

use parser::{ir::*, lookup::CodeFile};

const TABLE_TEMPLATE: &str = include_str!("../assets/table.md");
const FUNC_TEMPLATE: &str = include_str!("../assets/function.md");

pub fn gen_page_md_template(
    template: &str,
    full_path: Vec<String>,
    node: Node,
    code: &HashMap<String, CodeFile>,
) -> String {
    match node {
        Node::Table(t) => gen_table_page_md(template, full_path, t, code),
        Node::Function(f) => gen_function_page_md(template, f, code),
    }
}

pub fn gen_page_md(full_path: Vec<String>, node: Node, code: &HashMap<String, CodeFile>) -> String {
    match node {
        Node::Table(_) => gen_page_md_template(TABLE_TEMPLATE, full_path, node, code),
        Node::Function(_) => gen_page_md_template(FUNC_TEMPLATE, full_path, node, code),
    }
}

fn func_inline_md(func: &Function, code: &HashMap<String, CodeFile>) -> String {
    let link = func.full_name.replace(".", "/");

    let func_def_default = format!("function {}(...)", func.name);
    let func_def = match func.name.as_str() {
        "__index" | "__newindex" => func_def_default.clone(),
        _ => code
            .get(&func.info.source)
            .and_then(|cf| {
                if func.info.func_def_lines.0 > 0 {
                    cf.get_func_def((func.info.func_def_lines.0 as usize).saturating_sub(1))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| func_def_default.clone()),
    }
    .trim_start()
    .to_string();
    format!("- [<code class=\"hl\">{}</code>](<{}>)", func_def, link)
}

pub fn gen_table_page_md(
    template: &str,
    full_path: Vec<String>,
    table: Table,
    code: &HashMap<String, CodeFile>,
) -> String {
    let Table {
        full_name,
        name,
        children,
        functions,
    } = table;

    let mut table_path = Vec::new();
    let mut link_builder = Vec::new();
    for p in full_path {
        link_builder.push(p.clone());
        let link = link_builder.join("/");
        table_path.push(format!("<a href=\"/{link}\"><code>{p}</code></a>"));
    }
    let table_path = table_path.join(" / ");

    let mut children_content = Vec::new();
    for child in children {
        let link = child.full_name.replace(".", "/");
        children_content.push(format!("- [`{}`](<{}>)", child.name, link));
    }
    let mut functions_content = Vec::new();
    for func in functions {
        let s = func_inline_md(&func, code);
        functions_content.push(s);
    }

    template
        .replace("{{TABLE_PATH}}", &table_path)
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

    let (func_def_code, offset) = if let Some(cf) = code.get(&info.source)
        && info.func_def_lines.0 >= 0
    {
        let (fd, offset) = cf.get_func_decl_with_comments(
            info.func_def_lines.0 as usize,
            info.func_def_lines.1 as usize,
        );
        (fd.join("\n"), offset)
    } else {
        (format!("function {name}(...)"), 0)
    };

    let func_def_line_start = (info.func_def_lines.0.max(0) as usize)
        .saturating_sub(offset + 2)
        .max(0);

    let func_def = format!(
        "<pre><code class=\"highlight-lua\" data-ln-start-from=\"{func_def_line_start}\">
-- @/{}:{}
{func_def_code}
</code></pre>",
        info.source, info.func_def_lines.0
    );

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
                    "<pre><code class=\"highlight-lua\" data-ln-start-from=\"{}\">{call_code}</code></pre>", i.saturating_sub(1)
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
        .replace("{{FUNC_DEF_LINE}}", &(info.func_def_lines.0).to_string())
        .replace("{{FUNC_CALLERS}}", &func_callers.join("\n"))
}
