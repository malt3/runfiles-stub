//! Demo program: Calculate SHA256 hash of a file
//!
//! Usage: hash-file <path>
//! Outputs: SHA256:<hex_hash>

use sha2::{Digest, Sha256};
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        return ExitCode::from(1);
    }

    let file_path = &args[1];

    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file '{}': {}", file_path, e);
            return ExitCode::from(1);
        }
    };

    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buffer[..n]),
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                return ExitCode::from(1);
            }
        }
    }

    let hash = hasher.finalize();
    println!("SHA256:{:x}", hash);

    ExitCode::SUCCESS
}
