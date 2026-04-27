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

const PAGE_ROOT: &str = include_str!("../assets/root.md");
const PAGE_SEARCH: &str = include_str!("../assets/search.md");
const PAGE_404: &str = include_str!("../assets/404.md");

struct AppState {
    html_template: String,
    url_prefix: String,

    ge_data: Table,
    ve_data: Table,

    code: HashMap<String, CodeFile>,

    // Caches to speed up searching
    all_func_names: Vec<(String, String)>,
    all_child_names: Vec<(String, String)>,
}

impl AppState {
    fn new(ge_data: Table, ve_data: Table, code: HashMap<String, CodeFile>) -> Self {
        let html_template =
            std::fs::read_to_string("assets/template.html").expect("Could not find HTML template!");

        let url_prefix = std::env::var("URL_PREFIX")
            .ok()
            .unwrap_or(String::from("http://localhost:3030"));
        println!("URL_PREFIX = {url_prefix}");

        let mut all_func_names = ge_data.get_all_function_names();
        all_func_names.append(&mut ve_data.get_all_function_names());

        let mut all_child_names = ge_data.get_all_children_names();
        all_child_names.append(&mut ve_data.get_all_children_names());

        Self {
            html_template,
            url_prefix,

            ge_data,
            ve_data,

            code,

            all_func_names,
            all_child_names,
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
            .replace("{{URL_PREFIX}}", &self.url_prefix)
    }

    fn get_root(&self) -> String {
        self.template_page("BeamNG", PAGE_ROOT)
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

    fn get_search_page(&self, query: &str) -> String {
        let search_results = self.search_docs(query);
        let mut content = Vec::new();
        for (full_name, _, _) in search_results {
            let link = full_name.replace(".", "/");
            content.push(format!("- [`{}`](<{}>)", full_name, link));
        }
        let content = content.join("\n");
        let content = PAGE_SEARCH.replace("{{SEARCH_RESULTS}}", &content);
        self.template_page("BeamNG", &content)
    }

    fn search_docs(&self, query: &str) -> Vec<(&String, &String, usize)> {
        fn query_dist(query: &str, a: &str, b: &str) -> f32 {
            inner_query_dist(query, a).min(inner_query_dist(query, b))
        }

        fn inner_query_dist(query: &str, candidate: &str) -> f32 {
            if query.is_empty() {
                return candidate.len() as f32;
            }
            if candidate.is_empty() {
                return query.len() as f32;
            }

            let q = query.as_bytes();
            let c = candidate.as_bytes();

            let mut prev: Vec<u16> = (0..=c.len() as u16).collect();
            let mut curr = vec![0u16; c.len() + 1];

            for (i, &qb) in q.iter().enumerate() {
                curr[0] = (i + 1) as u16;

                for (j, &cb) in c.iter().enumerate() {
                    let cost = if qb == cb { 0 } else { 1 };

                    let del = prev[j + 1] + 1;
                    let ins = curr[j] + 1;
                    let sub = prev[j] + cost;

                    curr[j + 1] = del.min(ins).min(sub);
                }

                std::mem::swap(&mut prev, &mut curr);
            }

            let mut dist = prev[c.len()] as f32;

            // --- Substring bonus (big impact for search quality) ---
            let lcs = longest_common_substring(q, c) as f32;
            dist -= lcs * 0.7; // tune weight

            // --- Prefix bonus (important for function names) ---
            let prefix = common_prefix_len(q, c) as f32;
            dist -= prefix * 0.5;

            dist.max(0.0)
        }

        fn longest_common_substring(a: &[u8], b: &[u8]) -> usize {
            let mut dp = vec![0; b.len() + 1];
            let mut max_len = 0;

            for i in 0..a.len() {
                let mut prev = 0;
                for j in 0..b.len() {
                    let temp = dp[j + 1];
                    if a[i] == b[j] {
                        dp[j + 1] = prev + 1;
                        max_len = max_len.max(dp[j + 1]);
                    } else {
                        dp[j + 1] = 0;
                    }
                    prev = temp;
                }
            }

            max_len
        }

        fn common_prefix_len(a: &[u8], b: &[u8]) -> usize {
            a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
        }

        let searchers = self
            .all_func_names
            .iter()
            .chain(self.all_child_names.iter())
            .collect::<Vec<_>>();

        let max_dist = 3;
        let mut dists = searchers
            .into_iter()
            .filter_map(|(full_name, name)| {
                let dist = query_dist(query, full_name, name);
                let dist = (dist * 2.0) as usize;
                if dist < max_dist {
                    Some((full_name, name, dist))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        dists.sort_by_key(|(_, _, dist)| *dist);

        dists.truncate(25);

        dists
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
        .route("/search", get(search))
        .route("/{*article_name}", get(article_resolver))
        .with_state(app_state);

    println!("Starting the web server now...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root(State(state): State<Arc<AppState>>) -> (StatusCode, Html<String>) {
    let page = state.get_root();
    (StatusCode::OK, Html(page))
}

async fn search(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> (StatusCode, Html<String>) {
    let query = params.get("q").map(String::as_str).unwrap_or("");
    let generated = state.get_search_page(query);
    (StatusCode::OK, Html(generated))
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
