//! Demo program: Add numbers given as command line arguments
//!
//! Usage: add-numbers <num1> <num2> [num3 ...]
//! Outputs: SUM:<result>

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <num1> <num2> [num3 ...]", args[0]);
        return ExitCode::from(1);
    }

    let mut sum: i64 = 0;

    for arg in &args[1..] {
        match arg.parse::<i64>() {
            Ok(n) => sum += n,
            Err(e) => {
                eprintln!("Error parsing '{}' as number: {}", arg, e);
                return ExitCode::from(1);
            }
        }
    }

    println!("SUM:{}", sum);

    ExitCode::SUCCESS
}
