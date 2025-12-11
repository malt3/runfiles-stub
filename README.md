# Runfiles Stub

A minimal, dependency-free Bazel runfiles stub runner written in Rust without stdlib or libc.

## Overview

This project provides a **template-based stub runner** for Bazel runfiles that:
- Has **zero dependencies** (no stdlib, no libc, statically linked)
- Is **extremely small** (~16KB binary size)
- Supports **selective path transformation** (choose which args to resolve via runfiles)
- Works with both **manifest-based** and **directory-based** runfiles
- Uses **direct Linux syscalls** for all operations

## Architecture

The project consists of two components:

1. **`runfiles-stub`** - The minimal stub runner template (no stdlib/libc)
2. **`finalize-stub`** - A regular Rust tool to finalize templates (uses stdlib)

### Workflow

```
Template Binary (runfiles-stub)
    |
    | finalize-stub --transform=0 -o output template arg0 arg1 ...
    v
Finalized Binary
    |
    | RUNFILES_DIR=/path ./output
    v
Resolves paths & executes target
```

## Building

### Supported Architectures

The stub runner supports:
- **x86_64** (Intel/AMD 64-bit)
- **aarch64** (ARM 64-bit)

All syscalls are architecture-specific and use native calling conventions.

### Build the stub template

**For x86_64:**
```bash
cargo build --release
```

**For aarch64:**
```bash
# Install the target if needed
rustup target add aarch64-unknown-linux-gnu

# Build for aarch64
cargo build --release --target aarch64-unknown-linux-gnu
```

This produces the template binary:
- x86_64: `target/release/runfiles-stub`
- aarch64: `target/aarch64-unknown-linux-gnu/release/runfiles-stub`

### Build the finalizer tool

```bash
cd finalize-stub
cargo build --release
```

This produces `finalize-stub/target/release/finalize-stub`.

## Usage

### Basic Example: Transform All Arguments

```bash
# Create a finalized stub that resolves all args through runfiles
./finalize-stub/target/release/finalize-stub \
    -o my_stub \
    target/release/runfiles-stub \
    _main/bin/mytool \
    data/input.txt

# Run it
RUNFILES_MANIFEST_FILE=manifest.txt ./my_stub
```

### Selective Transformation

Use `--transform=N` to specify which arguments should be resolved through runfiles:

```bash
# Only transform arg0 (executable), keep others literal
./finalize-stub/target/release/finalize-stub \
    --transform=0 \
    -o my_stub \
    target/release/runfiles-stub \
    _main/bin/echo \
    --flag \
    /absolute/path

# Transform arg0 and arg2, keep arg1 literal
./finalize-stub/target/release/finalize-stub \
    --transform=0 --transform=2 \
    -o my_stub \
    target/release/runfiles-stub \
    _main/bin/tool \
    --literal-flag \
    data/file.txt
```

### Output to Stdout

```bash
# Write binary to stdout (for piping)
./finalize-stub/target/release/finalize-stub \
    target/release/runfiles-stub \
    _main/bin/tool arg1 > output.bin

chmod +x output.bin
```

### Running Finalized Stubs

Finalized stubs need runfiles environment variables:

```bash
# With manifest file
RUNFILES_MANIFEST_FILE=/path/to/manifest.txt ./my_stub

# With runfiles directory
RUNFILES_DIR=/path/to/runfiles ./my_stub
```

## Finalizer Options

```
Usage: finalize-stub [OPTIONS] <template> <arg0> [arg1 ...]

Options:
  --transform=N   Mark argument N for runfiles resolution (can be repeated)
                  If not specified, all arguments are transformed by default
  -o <output>     Write output to file (default: stdout)

Arguments:
  <template>      Path to template runfiles-stub binary
  <arg0>          Executable path
  <arg1...]       Additional arguments (up to 9 more)
```

## How It Works

### Template Placeholders

The stub template contains special placeholders embedded in the binary:

- `@@RUNFILES_ARGC@@` - Number of arguments (32 bytes)
- `@@RUNFILES_TRANSFORM_FLAGS@@` - Bitmask of which args to transform (32 bytes)
- `ARG0_PLACEHOLDER` through `ARG9_PLACEHOLDER` - Argument values (256 bytes each)

The finalizer tool finds and replaces these placeholders with actual values.

### Runtime Behavior

At runtime, the stub:

1. **Validates** it's been finalized (not still a template)
2. **Reads** RUNFILES_MANIFEST_FILE or RUNFILES_DIR environment variables
3. **Initializes** the runfiles library
4. **Resolves** each argument:
   - If transform flag is set for that arg: resolve via runfiles
   - If absolute path: keep as-is
   - Otherwise: use literal value
5. **Passes through** all environment variables from the stub's environment
6. **Executes** the target using `execve` syscall

**Environment Variable Passthrough**: The stub reads `/proc/self/environ` and passes all environment variables (including `RUNFILES_MANIFEST_FILE` and `RUNFILES_DIR`) to the executed program. This allows the executed program to also use runfiles if needed.

### Runfiles Resolution

**Manifest-based** (RUNFILES_MANIFEST_FILE):
```
# Format: key value
_main/bin/tool /absolute/path/to/tool
data/file.txt /absolute/path/to/file.txt
```

**Directory-based** (RUNFILES_DIR):
```
Joins: $RUNFILES_DIR + "/" + requested_path
```

### No Stdlib Implementation

The stub runner uses **direct Linux syscalls** for everything:
- **File I/O**: `open`/`openat`, `read`, `close` syscalls (arch-specific)
- **Execution**: `execve` syscall
- **Process control**: `exit` syscall
- **No malloc**: static buffers only
- **Custom intrinsics**: `memcpy`/`memset`/`memcmp` implementations

**Architecture-specific syscalls**:
- **x86_64**: Uses `syscall` instruction with syscall numbers in `rax`
- **aarch64**: Uses `svc #0` instruction with syscall numbers in `x8`, `openat` for file opening

This results in a minimal binary with zero runtime dependencies.

## Technical Details

### Binary Size

```bash
$ ls -lh target/release/runfiles-stub
-rwxr-xr-x  16K  runfiles-stub
```

### Dependencies

**runfiles-stub**: NONE
- No stdlib (`#![no_std]`)
- No libc
- Statically linked
- Direct syscalls only

**finalize-stub**: Standard Rust (std)
- Uses normal file I/O
- Not size-constrained

### Limitations

- **Maximum 10 arguments** (argv[0] through argv[9])
- **Linux only**: Supports x86_64 and aarch64 architectures
- **Path length limit**: 256 bytes per argument
- **Manifest size limit**: 1024 entries, 64KB file size
- **Environment size limit**: 16KB total, max 256 variables
- **Memory**: All buffers are static (no dynamic allocation)

## File Structure

```
.
├── Cargo.toml                  # runfiles-stub package
├── .cargo/
│   └── config.toml            # Static linking configuration
├── src/
│   └── main.rs                # Stub runner (no_std)
├── finalize-stub/
│   ├── Cargo.toml             # Finalizer package
│   ├── .cargo/
│   │   └── config.toml        # Override parent config
│   └── src/
│       └── main.rs            # Finalizer tool (std)
└── README.md
```

## Examples

### Example 1: Simple Echo Wrapper

```bash
# Create manifest
cat > manifest.txt << 'EOF'
my/echo /usr/bin/echo
EOF

# Finalize stub (transform only arg0)
./finalize-stub/target/release/finalize-stub \
    --transform=0 \
    -o echo_stub \
    target/release/runfiles-stub \
    my/echo \
    "Hello, World!"

# Run
RUNFILES_MANIFEST_FILE=manifest.txt ./echo_stub
# Output: Hello, World!
```

### Example 2: Full Transformation

```bash
# Finalize with all args transformed
./finalize-stub/target/release/finalize-stub \
    -o tool_stub \
    target/release/runfiles-stub \
    my/tool \
    data/input

# With manifest that resolves both
cat > manifest.txt << 'EOF'
my/tool /usr/bin/tool
data/input /path/to/input
EOF

RUNFILES_MANIFEST_FILE=manifest.txt ./tool_stub
```

### Example 3: Directory-Based Runfiles

```bash
# Set up runfiles directory
mkdir -p /tmp/runfiles/_main/bin
cp /usr/bin/echo /tmp/runfiles/_main/bin/tool

# Finalize
./finalize-stub/target/release/finalize-stub \
    -o stub \
    target/release/runfiles-stub \
    _main/bin/tool \
    "test"

# Run with directory
RUNFILES_DIR=/tmp/runfiles ./stub
```

### Example 4: Environment Variable Passthrough

```bash
# Create stub that runs 'env' command
cat > manifest.txt << 'EOF'
my/env /usr/bin/env
EOF

./finalize-stub/target/release/finalize-stub \
    --transform=0 \
    -o env_stub \
    target/release/runfiles-stub \
    my/env

# Run with custom environment variables
# All env vars are passed through to the executed program
CUSTOM_VAR=hello RUNFILES_MANIFEST_FILE=manifest.txt ./env_stub | grep CUSTOM_VAR
# Output: CUSTOM_VAR=hello

# The RUNFILES_* variables are also passed through
RUNFILES_MANIFEST_FILE=manifest.txt ./env_stub | grep RUNFILES
# Output: RUNFILES_MANIFEST_FILE=manifest.txt
```

This allows executed programs to:
- Access custom environment variables
- Use the runfiles library themselves (via inherited RUNFILES_* variables)
- Access PATH and other system variables

## Cross-Architecture Support

The stub runner uses Rust's conditional compilation (`#[cfg(target_arch = "...")]`) to provide native implementations for each architecture:

### x86_64 Implementation
- **Syscall instruction**: `syscall`
- **Register usage**: `rax` (syscall number), `rdi`, `rsi`, `rdx` (arguments)
- **Syscall numbers**: Standard x86_64 Linux syscall numbers
  - `SYS_READ=0`, `SYS_WRITE=1`, `SYS_OPEN=2`, `SYS_CLOSE=3`, `SYS_EXECVE=59`, `SYS_EXIT=60`

### aarch64 Implementation
- **Syscall instruction**: `svc #0`
- **Register usage**: `x8` (syscall number), `x0-x5` (arguments)
- **Syscall numbers**: aarch64-specific Linux syscall numbers
  - `SYS_READ=63`, `SYS_WRITE=64`, `SYS_OPENAT=56`, `SYS_CLOSE=57`, `SYS_EXECVE=221`, `SYS_EXIT=93`
- **Note**: Uses `openat` with `AT_FDCWD` instead of `open` (more modern approach)

The same Rust source code compiles to native binaries for both architectures with no runtime overhead.

## Design Goals

1. **Minimal Size**: No unnecessary dependencies or bloat
2. **Security**: Statically linked, no dynamic library risks
3. **Flexibility**: Selective transformation lets you mix runfiles and literal paths
4. **Simplicity**: Single binary, no runtime configuration beyond env vars
5. **Performance**: Direct syscalls, no abstraction overhead

## License

This project demonstrates building minimal, dependency-free executables in Rust.

## Contributing

This is a demonstration project showing:
- How to build `no_std` Rust binaries
- Direct Linux syscall usage with architecture-specific implementations
- Cross-architecture support using Rust's conditional compilation
- Binary template patching
- Bazel runfiles implementation

Feel free to use this as a reference for similar projects.

### Adding New Architectures

To add support for a new architecture:
1. Add a new `mod syscall_numbers` block with `#[cfg(target_arch = "...")]`
2. Implement architecture-specific inline assembly for each syscall function
3. Update `.cargo/config.toml` with linker flags for the new target
4. Test the build with `cargo build --release --target <triple>`
