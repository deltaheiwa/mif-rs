use std::fs::File;
use serde::Serialize;

pub mod wolvesville;

#[allow(dead_code)]
pub fn save_to_file(data: &impl Serialize, name: &str) {
    let file = File::create(format!("res/example_data/{}.json", name)).unwrap();
    serde_json::to_writer_pretty(file, &data).unwrap();
}

