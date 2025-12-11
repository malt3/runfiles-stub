use std::env;
use std::fs;
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::process;

const ARG_SIZE: usize = 256;
const ARGC_SIZE: usize = 32;

fn find_pattern(data: &[u8], pattern: &[u8]) -> Option<usize> {
    data.windows(pattern.len())
        .position(|window| window == pattern)
}

fn find_nth_pattern(data: &[u8], pattern: &[u8], n: usize) -> Option<usize> {
    let mut count = 0;
    let mut pos = 0;

    while pos < data.len() {
        if let Some(offset) = find_pattern(&data[pos..], pattern) {
            if count == n {
                return Some(pos + offset);
            }
            count += 1;
            // Skip past the entire matched pattern to avoid overlapping matches
            pos += offset + pattern.len();
        } else {
            break;
        }
    }
    None
}

fn replace_at(data: &mut [u8], offset: usize, new_value: &[u8], fixed_size: usize) -> Result<(), String> {
    if new_value.len() > fixed_size {
        return Err(format!(
            "Value too long: {} bytes > {} bytes max",
            new_value.len(),
            fixed_size
        ));
    }

    // Zero out the entire region
    for i in 0..fixed_size {
        data[offset + i] = 0;
    }

    // Copy new value
    data[offset..offset + new_value.len()].copy_from_slice(new_value);

    Ok(())
}

fn finalize_stub(template_path: &str, output_path: Option<&str>, argv: &[String], transform_flags: u32) -> Result<(), String> {
    if argv.is_empty() {
        return Err("At least one argument (argv[0]) is required".to_string());
    }

    if argv.len() > 10 {
        return Err("Maximum 10 arguments supported (argv[0] to argv[9])".to_string());
    }

    // Prevent overwriting the input file
    if let Some(output) = output_path {
        let template_canon = fs::canonicalize(template_path)
            .map_err(|e| format!("Failed to resolve template path: {}", e))?;
        let output_canon = fs::canonicalize(output).ok();

        if output_canon.as_ref() == Some(&template_canon) {
            return Err("Output path cannot be the same as template path (would overwrite input)".to_string());
        }
    }

    // Read template
    let mut data = fs::read(template_path)
        .map_err(|e| format!("Failed to read template {}: {}", template_path, e))?;

    // Find and replace ARGC
    let argc_pattern = b"@@RUNFILES_ARGC@@";
    let argc_pos = find_pattern(&data, argc_pattern)
        .ok_or("ARGC placeholder not found in template")?;

    let argc_str = argv.len().to_string();
    replace_at(&mut data, argc_pos, argc_str.as_bytes(), ARGC_SIZE)?;

    eprintln!("Replaced ARGC with: {}", argc_str);

    // Find and replace TRANSFORM_FLAGS
    let flags_pattern = b"@@RUNFILES_TRANSFORM_FLAGS@@";
    let flags_pos = find_pattern(&data, flags_pattern)
        .ok_or("TRANSFORM_FLAGS placeholder not found in template")?;

    let flags_str = transform_flags.to_string();
    replace_at(&mut data, flags_pos, flags_str.as_bytes(), 32)?;

    eprintln!("Replaced TRANSFORM_FLAGS with: {} (0b{:b})", flags_str, transform_flags);

    // Find and replace ARG placeholders
    let arg_pattern = &[b'@'; ARG_SIZE];

    for (i, arg) in argv.iter().enumerate() {
        let arg_pos = find_nth_pattern(&data, arg_pattern, i)
            .ok_or(format!("ARG{} placeholder not found in template", i))?;

        replace_at(&mut data, arg_pos, arg.as_bytes(), ARG_SIZE)?;
        eprintln!("Replaced ARG{} with: {}", i, arg);
    }

    // Write output
    if let Some(output) = output_path {
        fs::write(output, &data)
            .map_err(|e| format!("Failed to write output {}: {}", output, e))?;

        // Make executable (Unix only)
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(output)
                .map_err(|e| format!("Failed to get metadata: {}", e))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(output, perms)
                .map_err(|e| format!("Failed to set permissions: {}", e))?;
        }

        eprintln!("\nFinalized stub written to: {}", output);
        eprintln!("Total arguments: {}", argv.len());
    } else {
        // Write to stdout
        io::stdout().write_all(&data)
            .map_err(|e| format!("Failed to write to stdout: {}", e))?;
    }

    Ok(())
}

fn print_usage() {
    eprintln!("Usage: finalize-stub [OPTIONS] <template> <arg0> [arg1 ...]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --transform=N   Mark argument N for runfiles resolution (can be repeated)");
    eprintln!("                  If not specified, all arguments are transformed by default");
    eprintln!("  -o <output>     Write output to file (default: stdout)");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  <template>      Path to template runfiles-stub binary");
    eprintln!("  <arg0>          Executable path");
    eprintln!("  [arg1...]       Additional arguments (up to 9 more)");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  # Transform all arguments, write to stdout:");
    eprintln!("  finalize-stub template _main/bin/mytool data/input.txt > output");
    eprintln!();
    eprintln!("  # Transform only arg0, write to file:");
    eprintln!("  finalize-stub --transform=0 -o finalized template _main/bin/mytool --flag");
    eprintln!();
    eprintln!("  # Transform arg0 and arg2:");
    eprintln!("  finalize-stub --transform=0 --transform=2 -o output template cmd arg1 data/file");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage();
        process::exit(1);
    }

    // Parse flags
    let mut transform_indices = Vec::new();
    let mut output_file: Option<String> = None;
    let mut pos = 1;

    while pos < args.len() {
        if let Some(idx_str) = args[pos].strip_prefix("--transform=") {
            match idx_str.parse::<u32>() {
                Ok(idx) if idx < 10 => transform_indices.push(idx),
                _ => {
                    eprintln!("Error: Invalid --transform value: {}", idx_str);
                    eprintln!("Must be a number between 0 and 9");
                    process::exit(1);
                }
            }
            pos += 1;
        } else if args[pos] == "-o" {
            if pos + 1 >= args.len() {
                eprintln!("Error: -o requires an argument");
                process::exit(1);
            }
            output_file = Some(args[pos + 1].clone());
            pos += 2;
        } else {
            // No more flags
            break;
        }
    }

    // Check remaining args
    if args.len() - pos < 2 {
        print_usage();
        process::exit(1);
    }

    let template = &args[pos];
    let argv: Vec<String> = args[pos + 1..].to_vec();

    // Calculate transform flags bitmask
    let transform_flags = if transform_indices.is_empty() {
        // Default: transform all
        0xFFFFFFFF
    } else {
        // Only transform specified indices
        let mut flags = 0u32;
        for idx in transform_indices {
            flags |= 1 << idx;
        }
        flags
    };

    match finalize_stub(template, output_file.as_deref(), &argv, transform_flags) {
        Ok(()) => {
            if let Some(output) = output_file {
                eprintln!("\nSuccess! Run with:");
                eprintln!("  RUNFILES_DIR=<dir> {}", output);
                eprintln!("  or");
                eprintln!("  RUNFILES_MANIFEST_FILE=<file> {}", output);
            }
            // If writing to stdout, don't print success message (binary data was written)
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
