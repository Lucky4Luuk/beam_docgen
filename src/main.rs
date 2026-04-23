use markdown::{CompileOptions, Options};
use parser::{data, ir};

use std::{collections::VecDeque, sync::Arc};

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

const GAME_DIR: &str = "H:/SteamLibrary/steamapps/common/BeamNG.drive/";

const PAGE_404: &str = include_str!("../assets/404.md");

struct AppState {
    html_template: String,

    ge_data: ir::Table,
    ve_data: ir::Table,
}

impl AppState {
    fn new(ge_data: ir::Table, ve_data: ir::Table) -> Self {
        let html_template =
            std::fs::read_to_string("assets/template.html").expect("Could not find HTML template!");
        Self {
            html_template,
            ge_data,
            ve_data,
        }
    }

    fn template_page(&self, md: &str) -> String {
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
            .replace("{{PAGE_CONTENT}}", &md_html)
            .replace("{{URL_PREFIX}}", "http://localhost:3000")
    }

    fn get_404(&self) -> String {
        self.template_page(PAGE_404)
    }

    fn get_data(&self, path: Vec<String>) -> Option<ir::Node> {
        let first = path.first()?;
        match first.as_str() {
            "GE" => {
                let remaining: VecDeque<String> = path.into_iter().skip(1).collect();
                if remaining.is_empty() {
                    Some(ir::Node::Table(self.ge_data.clone()))
                } else {
                    self.ge_data.get_data(remaining)
                }
            }
            "VE" => {
                let remaining: VecDeque<String> = path.into_iter().skip(1).collect();
                if remaining.is_empty() {
                    Some(ir::Node::Table(self.ve_data.clone()))
                } else {
                    self.ve_data.get_data(remaining)
                }
            }
            _ => None,
        }
    }

    fn get_page(&self, path: String) -> Option<String> {
        let path_split: Vec<_> = path.split("/").map(|s| s.to_string()).collect();

        let node = self.get_data(path_split)?;
        let md = markdown_gen::gen_page_md(node);
        Some(md)
    }
}

#[tokio::main]
async fn main() {
    let ge_raw = std::fs::read_to_string("GE.json").expect("Failed to read GE data!");
    let ve_raw = std::fs::read_to_string("VE.json").expect("Failed to read VE data!");

    let ge_parsed = data::ApiData::from_json(&ge_raw).expect("Failed to parse json data!");
    println!("Parsed GE data succesfully!");

    let ve_parsed = data::ApiData::from_json(&ve_raw).expect("Failed to parse json data!");
    println!("Parsed VE data succesfully!");

    if let data::Node::Table(ge_table) = ge_parsed.root
        && let data::Node::Table(ve_table) = ve_parsed.root
    {
        let mut ge_node = ir::Table::from_data_table(GAME_DIR, "", ge_table);
        ge_node.sort_alphanumerical();

        let mut ve_node = ir::Table::from_data_table(GAME_DIR, "", ve_table);
        ve_node.sort_alphanumerical();

        let app_state = Arc::new(AppState::new(ge_node, ve_node));
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
    } else {
        panic!("Not yet supported to start with anything besides a table!");
    }
}

async fn root(s: State<Arc<AppState>>) -> (StatusCode, Html<String>) {
    article_resolver(Path(String::from("root")), s).await
}

async fn article_resolver(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Html<String>) {
    let path = path.trim_end_matches('/').to_string();
    if let Some(content) = state.get_page(path) {
        let generated = state.template_page(&content);
        (StatusCode::OK, Html(generated))
    } else {
        (StatusCode::NOT_FOUND, Html(state.get_404()))
    }
}
