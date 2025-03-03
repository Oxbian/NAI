use chrono::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;

pub fn warn(content: String) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("log.txt")
        .unwrap();
    let utc: DateTime<Local> = Local::now();
    writeln!(file, "[{}] {}", utc, content);
}
