//! A simple CLI for the parser, so we can parse and serialize data for later usage

use parser::*;

const GAME_DIR: &str = "H:/SteamLibrary/steamapps/common/BeamNG.drive/";

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Expected first CLI argument");

    let raw = std::fs::read_to_string(path).expect("Failed to read data!");

    let parsed = data::ApiData::from_json(&raw).expect("Failed to parse data!");
    println!("File read and parsed!");

    if let data::Node::Table(table) = parsed.root {
        let mut node = ir::Table::from_data_table("", table);
        node.sort_alphanumerical();
        println!("IR built and sorted!");

        println!("Starting source look up now...");
        let code = lookup::look_up_src_info(&mut node, GAME_DIR);

        println!("Writing to outputs now...");

        let s = serde_json::to_string(&code).unwrap();
        std::fs::write("code_files.json", s).expect("Failed to write output.json!");

        let s = serde_json::to_string(&node).unwrap();
        std::fs::write("output.json", s).expect("Failed to write output.json!");
    } else {
        panic!("Unsupported data type! For this CLI, the data MUST start with a table.");
    }
}
