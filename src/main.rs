use markdown::{CompileOptions, Options};
use parser::{
    ir::{Node, Table},
    lookup::{self, CodeFile},
};

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::get,
};
use tower::ServiceBuilder;
use tower_http::services::ServeDir;

mod markdown_gen;

const PAGE_404: &str = include_str!("../assets/404.md");

struct AppState {
    html_template: String,

    ge_data: Table,
    ve_data: Table,

    code: HashMap<String, CodeFile>,
}

impl AppState {
    fn new(ge_data: Table, ve_data: Table, code: HashMap<String, CodeFile>) -> Self {
        let html_template =
            std::fs::read_to_string("assets/template.html").expect("Could not find HTML template!");
        Self {
            html_template,
            ge_data,
            ve_data,
            code,
        }
    }

    fn template_page(&self, tln: &str, md: &str) -> String {
        let md_html = markdown::to_html_with_options(
            md,
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
        self.html_template
            .replace("{{TLN}}", tln)
            .replace("{{PAGE_CONTENT}}", &md_html)
            .replace("{{URL_PREFIX}}", "http://localhost:3000")
    }

    fn get_404(&self) -> String {
        self.template_page("BeamNG", PAGE_404)
    }

    fn get_data(&self, path: Vec<String>) -> Option<Node> {
        let first = path.first()?;
        match first.as_str() {
            "GE" => {
                let remaining: VecDeque<String> = path.into_iter().skip(1).collect();
                if remaining.is_empty() {
                    Some(Node::Table(self.ge_data.clone()))
                } else {
                    self.ge_data.get_data(remaining)
                }
            }
            "VE" => {
                let remaining: VecDeque<String> = path.into_iter().skip(1).collect();
                if remaining.is_empty() {
                    Some(Node::Table(self.ve_data.clone()))
                } else {
                    self.ve_data.get_data(remaining)
                }
            }
            _ => None,
        }
    }

    fn get_page(&self, path: String) -> Option<(String, String)> {
        let path = path.trim_matches('/').to_string();
        let path_md = format!("content/{}.md", path);

        let path_split: Vec<_> = path.split("/").map(|s| s.to_string()).collect();
        let first = path_split.first()?.clone();

        let node = self.get_data(path_split)?;
        let md = if std::path::Path::new(&path_md).exists() {
            if let Ok(content) = std::fs::read_to_string(&path_md) {
                markdown_gen::gen_page_md_template(&content, node, &self.code)
            } else {
                println!("How the fuck did this happen?? What???");
                markdown_gen::gen_page_md(node, &self.code)
            }
        } else {
            markdown_gen::gen_page_md(node, &self.code)
        };
        Some((first, md))
    }
}

#[tokio::main]
async fn main() {
    let code_json = std::fs::read_to_string("code_files.json").expect("Failed to read GE data!");
    let ge_json = std::fs::read_to_string("GE_parsed.json").expect("Failed to read GE data!");
    let ve_json = std::fs::read_to_string("VE_parsed.json").expect("Failed to read VE data!");

    let code: HashMap<String, CodeFile> =
        serde_json::from_str(&code_json).expect("Failed to parse json data!");

    let mut ge: Table = serde_json::from_str(&ge_json).expect("Failed to parse json data!");
    println!("Parsed GE data succesfully!");

    let mut ve: Table = serde_json::from_str(&ve_json).expect("Failed to parse json data!");
    println!("Parsed VE data succesfully!");

    ge.sort_alphanumerical();
    ve.sort_alphanumerical();

    let app_state = Arc::new(AppState::new(ge, ve, code));
    println!("App state succesfully created!");

    let app = Router::new()
        .route("/", get(root))
        .nest_service(
            "/static",
            ServiceBuilder::new().service(ServeDir::new("static")),
        )
        .route("/{*article_name}", get(article_resolver))
        .with_state(app_state);

    println!("Starting the web server now...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root(s: State<Arc<AppState>>) -> (StatusCode, Html<String>) {
    article_resolver(Path(String::from("root")), s).await
}

async fn article_resolver(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Html<String>) {
    let path = path.trim_end_matches('/').to_string();
    if let Some((tln, content)) = state.get_page(path) {
        let generated = state.template_page(&tln, &content);
        (StatusCode::OK, Html(generated))
    } else {
        (StatusCode::NOT_FOUND, Html(state.get_404()))
    }
}
