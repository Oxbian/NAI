use std::fs::File;
use std::io::{self, Write};

pub fn print_in_file(content: String) -> io::Result<()> {
    // Open the file (create it if it doesn't exist, or truncate it if it does)
    let mut file = File::create("debug.txt")?;

    // Write the content to the file
    file.write_all(content.as_bytes())?;

    Ok(())
}
