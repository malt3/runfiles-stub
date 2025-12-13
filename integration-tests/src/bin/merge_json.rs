//! Demo program: Merge two JSON files
//!
//! Usage: merge-json <file1.json> <file2.json>
//! Outputs: MERGED:<json_result>
//!
//! Performs a shallow merge where keys from file2 override keys from file1.

use serde_json::Value;
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <file1.json> <file2.json>", args[0]);
        return ExitCode::from(1);
    }

    let file1_path = &args[1];
    let file2_path = &args[2];

    // Read and parse first file
    let content1 = match fs::read_to_string(file1_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading '{}': {}", file1_path, e);
            return ExitCode::from(1);
        }
    };

    let json1: Value = match serde_json::from_str(&content1) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing '{}' as JSON: {}", file1_path, e);
            return ExitCode::from(1);
        }
    };

    // Read and parse second file
    let content2 = match fs::read_to_string(file2_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading '{}': {}", file2_path, e);
            return ExitCode::from(1);
        }
    };

    let json2: Value = match serde_json::from_str(&content2) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing '{}' as JSON: {}", file2_path, e);
            return ExitCode::from(1);
        }
    };

    // Merge the two JSON objects
    let merged = match (json1, json2) {
        (Value::Object(mut map1), Value::Object(map2)) => {
            for (k, v) in map2 {
                map1.insert(k, v);
            }
            Value::Object(map1)
        }
        (v1, v2) => {
            // If not both objects, create an array
            Value::Array(vec![v1, v2])
        }
    };

    // Output the merged result
    let output = serde_json::to_string(&merged).unwrap();
    println!("MERGED:{}", output);

    ExitCode::SUCCESS
}
