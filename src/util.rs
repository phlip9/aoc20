use anyhow::{Context, Result};
use std::{fs::File, io::Read, path::Path};

const NEWLINE: u8 = 0x0A;

pub fn read_file_bytes(path: &str) -> Result<Vec<u8>> {
    let path = Path::new(path);
    let mut file = File::open(path).context("Failed to open file")?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).context("Failed to read file")?;

    Ok(buf)
}

pub fn split_bytes_lines(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
    bytes
        .split(|byte| *byte == NEWLINE)
        .take_while(|piece| !piece.is_empty())
}
