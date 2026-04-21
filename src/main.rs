use beam_dump_parser::{data, ir, markdown_gen};
use markdown::{CompileOptions, Options};

const HTML_TEMPLATE: &'static str = include_str!("../assets/template.html");
const SAKURA_CSS: &'static str = include_str!("../assets/sakura.css");

fn main() {
    let raw = std::fs::read_to_string("GE.json").expect("Failed to read file!");

    let parsed: data::ApiData = serde_json::from_str(&raw).expect("Failed to parse json data!");
    println!("Parsed data succesfully!");

    if let data::Node::Table(table) = parsed.root {
        let root = ir::Table::from_data_table("", table);

        let md = markdown_gen::gen_md(root);
        std::fs::write("GE.md", &md).expect("Failed to write MD to file!");

        println!("Markdown generated!");

        let md_html = markdown::to_html_with_options(
            &md,
            &Options {
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();
        let html = HTML_TEMPLATE
            .replace("{{CONTENT}}", &md_html)
            .replace("{{SAKURA_CSS}}", SAKURA_CSS);
        std::fs::write("GE.html", html).expect("Failed to write MD to file!");

        println!("HTML generated and saved!");
    }
}
