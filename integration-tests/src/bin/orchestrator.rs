//! Demo program: Orchestrator that calls other demo programs
//!
//! Usage: orchestrator <command> [args...]
//!
//! Commands:
//!   hash-and-report <hash_binary> <file>
//!     - Calls hash-file binary and reports the result
//!
//!   sum-and-double <add_binary> <num1> <num2>
//!     - Calls add-numbers binary, then doubles the result
//!
//!   chain <binary1> <binary2> <file1> <file2>
//!     - Runs binary1 on file1, binary2 on file2, combines results
//!
//! This binary is designed to test runfiles environment variable propagation.
//! It expects RUNFILES_DIR or RUNFILES_MANIFEST_FILE to be set and passes them
//! to child processes.

use std::env;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <command> [args...]", args[0]);
        eprintln!("Commands: hash-and-report, sum-and-double, chain, env-check");
        return ExitCode::from(1);
    }

    let command = &args[1];

    match command.as_str() {
        "hash-and-report" => {
            if args.len() != 4 {
                eprintln!("Usage: {} hash-and-report <hash_binary> <file>", args[0]);
                return ExitCode::from(1);
            }
            hash_and_report(&args[2], &args[3])
        }
        "sum-and-double" => {
            if args.len() != 5 {
                eprintln!("Usage: {} sum-and-double <add_binary> <num1> <num2>", args[0]);
                return ExitCode::from(1);
            }
            sum_and_double(&args[2], &args[3], &args[4])
        }
        "chain" => {
            if args.len() != 6 {
                eprintln!("Usage: {} chain <hash_binary1> <hash_binary2> <file1> <file2>", args[0]);
                return ExitCode::from(1);
            }
            chain(&args[2], &args[3], &args[4], &args[5])
        }
        "env-check" => {
            env_check()
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            ExitCode::from(1)
        }
    }
}

fn hash_and_report(hash_binary: &str, file: &str) -> ExitCode {
    let output = Command::new(hash_binary)
        .arg(file)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                println!("ORCHESTRATOR:HASH_RESULT:{}", stdout.trim());
                ExitCode::SUCCESS
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!("Hash binary failed: {}", stderr);
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("Failed to execute hash binary '{}': {}", hash_binary, e);
            ExitCode::from(1)
        }
    }
}

fn sum_and_double(add_binary: &str, num1: &str, num2: &str) -> ExitCode {
    let output = Command::new(add_binary)
        .arg(num1)
        .arg(num2)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // Parse the result from "SUM:<number>"
                if let Some(sum_str) = stdout.trim().strip_prefix("SUM:") {
                    if let Ok(sum) = sum_str.parse::<i64>() {
                        let doubled = sum * 2;
                        println!("ORCHESTRATOR:DOUBLED:{}", doubled);
                        return ExitCode::SUCCESS;
                    }
                }
                eprintln!("Failed to parse sum output: {}", stdout);
                ExitCode::from(1)
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!("Add binary failed: {}", stderr);
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("Failed to execute add binary '{}': {}", add_binary, e);
            ExitCode::from(1)
        }
    }
}

fn chain(binary1: &str, binary2: &str, file1: &str, file2: &str) -> ExitCode {
    // Run first binary on first file
    let output1 = Command::new(binary1)
        .arg(file1)
        .output();

    let result1 = match output1 {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            eprintln!("Binary1 failed: {}", stderr);
            return ExitCode::from(1);
        }
        Err(e) => {
            eprintln!("Failed to execute binary1 '{}': {}", binary1, e);
            return ExitCode::from(1);
        }
    };

    // Run second binary on second file
    let output2 = Command::new(binary2)
        .arg(file2)
        .output();

    let result2 = match output2 {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            eprintln!("Binary2 failed: {}", stderr);
            return ExitCode::from(1);
        }
        Err(e) => {
            eprintln!("Failed to execute binary2 '{}': {}", binary2, e);
            return ExitCode::from(1);
        }
    };

    println!("ORCHESTRATOR:CHAIN:{}|{}", result1, result2);
    ExitCode::SUCCESS
}

fn env_check() -> ExitCode {
    // Report which runfiles environment variables are set
    let runfiles_dir = env::var("RUNFILES_DIR").ok();
    let runfiles_manifest = env::var("RUNFILES_MANIFEST_FILE").ok();
    let java_runfiles = env::var("JAVA_RUNFILES").ok();

    println!("ORCHESTRATOR:ENV_CHECK:RUNFILES_DIR={}", runfiles_dir.as_deref().unwrap_or("<unset>"));
    println!("ORCHESTRATOR:ENV_CHECK:RUNFILES_MANIFEST_FILE={}", runfiles_manifest.as_deref().unwrap_or("<unset>"));
    println!("ORCHESTRATOR:ENV_CHECK:JAVA_RUNFILES={}", java_runfiles.as_deref().unwrap_or("<unset>"));

    ExitCode::SUCCESS
}
