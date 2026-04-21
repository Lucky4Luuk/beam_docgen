use beam_dump_parser::data::*;

fn main() {
    let raw = std::fs::read_to_string("Engine.json").expect("Failed to read file!");

    let parsed: ApiData = serde_json::from_str(&raw).expect("Failed to parse json data!");
    println!("{:?}", parsed);
}
