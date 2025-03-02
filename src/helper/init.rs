use std::fs::OpenOptions;
use std::io::{self, Write};

pub fn print_in_file(content: String) -> io::Result<()> {
    // Open the file (create it if it doesn't exist, or truncate it if it does)
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("debug.txt")
        .unwrap();

    if let Err(e) = writeln!(file, "{}", content) {
        eprintln!("Couldn't write to file: {}", e);
    }

    Ok(())
}
