# Runfiles Stub

A minimal, cross-platform Bazel runfiles stub runner that replaces shell scripts with tiny native binaries.

## Why This Exists

**Problem**: Many Bazel rules create shell script wrappers to invoke tools with their runfiles dependencies. Shell scripts aren't cross-platform—bash scripts don't work on Windows, batch files don't work on Unix.

**Solution**: This project provides tiny native binaries (10-68KB) that:
- Work on **Linux, macOS, and Windows**
- Resolve Bazel runfiles paths
- Forward arguments to the actual tool
- Can be **"cross-compiled"** (finalized) from any build platform for any target platform

## Primary Use Case: Bazel Rules

Instead of generating platform-specific shell scripts like this:

```bash
#!/bin/bash
# Generated wrapper script - Linux/macOS only!
exec $RUNFILES_DIR/my_workspace/bin/tool "$@"
```

Create a universal binary stub that works everywhere:

```bash
# Build once on any platform, works on all platforms
./finalize-stub \
  --transform=0 \
  -o my_tool \
  -- \
  runfiles-stub-x86_64-linux \
  my_workspace/bin/tool

# Runtime: works with runfiles on any platform
RUNFILES_DIR=/path/to/runfiles ./my_tool --flag arg1 arg2
```

This enables Bazel rules to create tiny, platform-agnostic entrypoints that work identically on Linux, macOS, and Windows.

## Features

- **Cross-platform**: Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64)
- **True cross-compilation**: Finalize stubs for **any target platform** from **any build platform**
  - Build on Linux → create Windows/macOS stubs
  - Build on macOS → create Linux/Windows stubs
  - Build on Windows → create Linux/macOS stubs
- **Deterministic**: Same inputs always produce identical output, regardless of build platform
- **Tiny binaries**: 10-68KB depending on platform
- **Runtime arguments**: Forward `$@` to the wrapped tool
- **No dependencies**: Fully static on Linux, minimal dependencies on macOS/Windows

## Quick Start

### Download Pre-built Binaries

```bash
# Download from GitHub releases
VERSION=v0.1.20251211
wget https://github.com/malt3/runfiles-stub/releases/download/${VERSION}/runfiles-stub-x86_64-linux
wget https://github.com/malt3/runfiles-stub/releases/download/${VERSION}/finalize-stub-x86_64-linux
chmod +x finalize-stub-x86_64-linux
```

### Create a Stub

```bash
# Finalize a stub that wraps /bin/echo
./finalize-stub-x86_64-linux \
  --transform=0 \
  -o my_echo \
  -- \
  runfiles-stub-x86_64-linux \
  /bin/echo

# Create a manifest
cat > manifest.txt << 'EOF'
/bin/echo /bin/echo
EOF

# Run it - embedded args + runtime args
RUNFILES_MANIFEST_FILE=manifest.txt ./my_echo "Hello from embedded!" arg1 arg2
# Output: Hello from embedded! arg1 arg2
```

The stub:
1. Resolves `/bin/echo` through runfiles (because `--transform=0`)
2. Appends runtime arguments (`arg1 arg2`)
3. Executes: `/bin/echo "Hello from embedded!" arg1 arg2`

## How It Works

### Two-Step Process

```
┌──────────────────┐
│  Template Binary │  Generic stub for a platform (10-68KB)
│ (runfiles-stub)  │  Contains placeholder sections
└────────┬─────────┘
         │
         │ finalize-stub patches placeholders with:
         │  - Number of arguments
         │  - Which args to transform (bitmask)
         │  - Actual argument values
         │
         ▼
┌──────────────────┐
│ Finalized Binary │  Ready-to-use stub (same size)
│   (my_tool)      │  Embedded args + accepts runtime args
└────────┬─────────┘
         │
         │ At runtime:
         │  1. Reads RUNFILES_DIR or RUNFILES_MANIFEST_FILE
         │  2. Resolves embedded args through runfiles
         │  3. Appends runtime $@ arguments
         │  4. Executes target with all args
         │
         ▼
    Target Program
```

### Cross-Platform Finalization

The finalizer works on any platform to create stubs for any platform:

```bash
# On Linux, create stubs for all platforms
./finalize-stub-x86_64-linux -o stub-linux -- runfiles-stub-x86_64-linux /bin/tool
./finalize-stub-x86_64-linux -o stub-macos -- runfiles-stub-x86_64-macos /bin/tool
./finalize-stub-x86_64-linux -o stub.exe -- runfiles-stub-x86_64-windows.exe 'C:\Windows\System32\cmd.exe'

# The finalizer just patches bytes - no platform-specific logic needed!
```

This is crucial for Bazel: your **exec platform** (where the build runs) can create stubs for any **target platform** (where the output runs).

## Supported Platforms

| Platform | Architectures | Template Size | Notes |
|----------|--------------|---------------|-------|
| **Linux** | x86_64, aarch64 | 10-68KB | Fully static, no dependencies |
| **macOS** | x86_64, aarch64 | 13-49KB | Links with libSystem |
| **Windows** | x86_64 | 22KB | Links with kernel32.dll, shell32.dll |

**Finalizers** (the tool that patches templates):
- Linux: x86_64, aarch64 (static musl binaries)
- macOS: x86_64, aarch64
- Windows: x86_64

## Usage

### Basic Usage

```bash
# Syntax
finalize-stub [OPTIONS] -- <template> <arg0> [arg1 ...]

# Transform all arguments (default)
finalize-stub -o my_tool -- template my_workspace/bin/tool data/input.txt

# Transform only specific arguments
finalize-stub --transform=0,2 -o my_tool -- template /bin/tool --flag data/file
#                          ^^^                                 ^^^        ^^^^
#                      arg0 and arg2                        transform   literal   transform
```

### Options

```
--transform=N[,N...]  Mark argument(s) N for runfiles resolution
                      Can use comma-separated values (--transform=0,1,2)
                      or repeat the flag (--transform=0 --transform=1)
                      Default: transform ALL arguments

-o <output>           Output file path (default: stdout)

--                    Stop parsing flags; treat remaining args as positional
```

### Runtime Arguments

Finalized stubs forward runtime arguments to the target:

```bash
# Create stub with embedded args
finalize-stub --transform=0 -o stub -- template /bin/grep "pattern"

# Run with additional args - they're forwarded as argv
./stub file1.txt file2.txt
# Executes: /bin/grep "pattern" file1.txt file2.txt
```

This is like bash `$@` - embedded args come first, runtime args are appended.

### Runfiles Environment

Stubs require runfiles environment variables:

```bash
# Manifest-based (file maps runfiles paths to absolute paths)
RUNFILES_MANIFEST_FILE=/path/to/manifest.txt ./stub

# Directory-based (simple directory layout)
RUNFILES_DIR=/path/to/runfiles ./stub
```

## Building from Source

### Prerequisites

- Rust toolchain (stable)
- For cross-compilation: platform toolchains (mingw-w64 for Windows, etc.)

### Build All Binaries

```bash
# Linux templates
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu

# macOS templates
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Windows template
cargo build --release --target x86_64-pc-windows-gnu

# Finalizers
cd finalize-stub
cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-pc-windows-gnu
```

See `.github/workflows/release.yml` for the complete build matrix.

## Architecture Details

### Platform Implementations

| Platform | Entry Point | API | Process Execution | Path Separator |
|----------|-------------|-----|-------------------|----------------|
| **Linux** | Custom `_start` | Raw syscalls (no libc) | `execve` syscall | `/` |
| **macOS** | Standard `main` | libc functions | `execve` function | `/` |
| **Windows** | Standard `main` | Win32 API (UTF-16) | `CreateProcessW` | `\` |

### Path Handling

**Input** (embedded arguments): Always Unix-style forward slashes
```
my_workspace/bin/tool
data/input.txt
```

**Output** (after runfiles resolution): Platform-native
```
Linux/macOS:  /absolute/path/to/tool
Windows:      C:\absolute\path\to\tool
```

The Windows implementation automatically converts `/` to `\` in directory-based mode.

### Binary Size Breakdown

Sizes vary by platform due to different linking requirements:

- **x86_64 Linux**: ~10KB (fully static, no libc)
- **aarch64 Linux**: ~67KB (static, larger due to RISC architecture)
- **x86_64 macOS**: ~13KB (links libSystem)
- **aarch64 macOS**: ~49KB (links libSystem, ARM64)
- **x86_64 Windows**: ~22KB (links kernel32.dll, shell32.dll)

## Use Cases for Bazel Rules

### 1. Tool Wrappers

Create consistent wrappers for tools that need runfiles:

```python
# In your Bazel rule
def _my_tool_impl(ctx):
    stub = ctx.actions.declare_file(ctx.label.name)

    ctx.actions.run(
        executable = ctx.executable._finalizer,
        arguments = [
            "--transform=0",
            "-o", stub.path,
            ctx.file._template.path,
            ctx.executable.tool.short_path,
        ],
        inputs = [ctx.file._template, ctx.executable.tool],
        outputs = [stub],
    )

    return [DefaultInfo(executable = stub)]
```

### 2. Test Runners

Wrap test executables with their data dependencies:

```python
# Create test runner stub
finalize-stub --transform=0,1 -o test_runner -- template \
    my_workspace/test/runner \
    my_workspace/test/data/fixtures.json
```

### 3. Binary Entry Points

Create tiny entry points for tools in //bin:

```python
# Instead of a shell script, create a native stub
finalize-stub --transform=0 -o bin/mytool -- template \
    my_workspace/tools/mytool
```

The stub is the same size whether it wraps a simple script or a complex binary.

## Limitations

- **Maximum 128 arguments** (embedded + runtime)
- **256 bytes per argument path**
- **Manifest limits**: 1024 entries, 64KB file size
- **Static buffers**: No dynamic allocation (all sizes fixed at compile time)

## FAQ

**Q: Why not just use shell scripts?**
A: Shell scripts aren't cross-platform. Bash doesn't work on Windows, batch files don't work on Unix. Native stubs work everywhere.

**Q: How is this different from other Bazel runfiles libraries?**
A: This creates standalone binaries, not library code. The stub is your program's entry point.

**Q: Can I use this outside Bazel?**
A: Yes! As long as you set `RUNFILES_DIR` or `RUNFILES_MANIFEST_FILE`, stubs work anywhere.

**Q: Why are the binaries different sizes?**
A: Platform differences. Linux can be fully static (smaller), macOS requires libSystem, Windows needs DLLs, ARM architectures need more instructions than x86.

**Q: Is the finalizer deterministic?**
A: Yes! The same inputs produce byte-identical outputs regardless of which platform you run the finalizer on. This is tested in CI.

## License

MIT License - See [LICENSE](LICENSE) file for details.

## Contributing

Contributions welcome! This project demonstrates:
- Cross-platform `no_std` Rust development
- Platform-specific system call interfaces
- Binary template patching techniques
- Bazel runfiles protocol implementation
