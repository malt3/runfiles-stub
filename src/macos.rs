// macOS-specific implementation using libc
// Unlike Linux version, this uses libc and can link with libsystem

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { exit(1) }
}

// External libc functions
extern "C" {
    fn exit(code: i32) -> !;
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn open(path: *const u8, flags: i32, ...) -> i32;
    fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
    fn close(fd: i32) -> i32;
    fn execve(path: *const u8, argv: *const *const u8, envp: *const *const u8) -> i32;

    // Access to environment - macOS provides this
    static mut environ: *const *const u8;
}

// File open flags
const O_RDONLY: i32 = 0;
const STDOUT: i32 = 1;

// String utilities
fn print(s: &[u8]) {
    unsafe {
        write(STDOUT, s.as_ptr(), s.len());
    }
}

fn str_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for i in 0..a.len() {
        if a[i] != b[i] {
            return false;
        }
    }
    true
}

fn str_starts_with(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() {
        return false;
    }
    str_eq(&haystack[..needle.len()], needle)
}

fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    for i in 0..haystack.len() {
        if haystack[i] == needle {
            return Some(i);
        }
    }
    None
}

// Environment variable reading via the environ pointer
fn get_env_var(name: &[u8], buf: &mut [u8]) -> Option<usize> {
    unsafe {
        let mut env_ptr = environ;

        // Iterate through environment variables
        while !(*env_ptr).is_null() {
            let entry_ptr = *env_ptr;

            // Find the length of this environment variable string
            let mut len = 0;
            while *entry_ptr.add(len) != 0 {
                len += 1;
                if len > 4096 {  // Safety limit
                    break;
                }
            }

            // Convert to slice
            let entry = core::slice::from_raw_parts(entry_ptr, len);

            // Look for '=' separator
            if let Some(eq_pos) = find_byte(entry, b'=') {
                let key = &entry[..eq_pos];
                let value = &entry[eq_pos + 1..];

                if str_eq(key, name) {
                    let copy_len = value.len().min(buf.len());
                    buf[..copy_len].copy_from_slice(&value[..copy_len]);
                    return Some(copy_len);
                }
            }

            env_ptr = env_ptr.add(1);
        }
    }

    None
}

// Manifest entry storage
const MAX_ENTRIES: usize = 1024;
const MAX_PATH_LEN: usize = 256;

struct ManifestEntry {
    key: [u8; MAX_PATH_LEN],
    key_len: usize,
    value: [u8; MAX_PATH_LEN],
    value_len: usize,
}

struct Manifest {
    entries: [ManifestEntry; MAX_ENTRIES],
    count: usize,
}

impl Manifest {
    fn new() -> Self {
        const EMPTY_ENTRY: ManifestEntry = ManifestEntry {
            key: [0; MAX_PATH_LEN],
            key_len: 0,
            value: [0; MAX_PATH_LEN],
            value_len: 0,
        };

        Self {
            entries: [EMPTY_ENTRY; MAX_ENTRIES],
            count: 0,
        }
    }

    fn add_entry(&mut self, key: &[u8], value: &[u8]) {
        if self.count >= MAX_ENTRIES {
            return;
        }

        let entry = &mut self.entries[self.count];
        let key_len = key.len().min(MAX_PATH_LEN);
        let value_len = value.len().min(MAX_PATH_LEN);

        entry.key[..key_len].copy_from_slice(&key[..key_len]);
        entry.key_len = key_len;
        entry.value[..value_len].copy_from_slice(&value[..value_len]);
        entry.value_len = value_len;

        self.count += 1;
    }

    fn lookup(&self, key: &[u8]) -> Option<&[u8]> {
        for i in 0..self.count {
            let entry = &self.entries[i];
            if str_eq(&entry.key[..entry.key_len], key) {
                return Some(&entry.value[..entry.value_len]);
            }
        }
        None
    }
}

// Load manifest file
fn load_manifest(path: &[u8]) -> Option<Manifest> {
    unsafe {
        let fd = open(path.as_ptr(), O_RDONLY);
        if fd < 0 {
            return None;
        }

        let mut file_buf = [0u8; 65536];
        let bytes_read = read(fd, file_buf.as_mut_ptr(), file_buf.len());
        close(fd);

        if bytes_read <= 0 {
            return None;
        }

        let mut manifest = Manifest::new();
        let data = &file_buf[..bytes_read as usize];
        let mut pos = 0;

        while pos < data.len() {
            let line_start = pos;
            while pos < data.len() && data[pos] != b'\n' {
                pos += 1;
            }

            let line = &data[line_start..pos];

            if let Some(space_pos) = find_byte(line, b' ') {
                let key = &line[..space_pos];
                let value = &line[space_pos + 1..];
                manifest.add_entry(key, value);
            }

            pos += 1;
        }

        Some(manifest)
    }
}

// Runfiles implementation
enum RunfilesMode {
    ManifestBased(Manifest),
    DirectoryBased([u8; MAX_PATH_LEN], usize),
}

struct Runfiles {
    mode: RunfilesMode,
}

impl Runfiles {
    fn create() -> Option<Self> {
        let mut manifest_path = [0u8; MAX_PATH_LEN];

        // Try RUNFILES_MANIFEST_FILE first
        if let Some(len) = get_env_var(b"RUNFILES_MANIFEST_FILE", &mut manifest_path) {
            if len > 0 {
                let mut path_with_null = [0u8; MAX_PATH_LEN + 1];
                path_with_null[..len].copy_from_slice(&manifest_path[..len]);

                if let Some(manifest) = load_manifest(&path_with_null[..len + 1]) {
                    return Some(Self {
                        mode: RunfilesMode::ManifestBased(manifest),
                    });
                }
            }
        }

        // Try RUNFILES_DIR
        let mut runfiles_dir = [0u8; MAX_PATH_LEN];
        if let Some(len) = get_env_var(b"RUNFILES_DIR", &mut runfiles_dir) {
            if len > 0 {
                return Some(Self {
                    mode: RunfilesMode::DirectoryBased(runfiles_dir, len),
                });
            }
        }

        None
    }

    fn rlocation(&self, path: &[u8]) -> Option<[u8; MAX_PATH_LEN]> {
        // If path is absolute, don't resolve through runfiles
        if path.len() > 0 && path[0] == b'/' {
            return None;
        }

        match &self.mode {
            RunfilesMode::ManifestBased(manifest) => {
                if let Some(resolved) = manifest.lookup(path) {
                    let mut result = [0u8; MAX_PATH_LEN];
                    let len = resolved.len().min(MAX_PATH_LEN);
                    result[..len].copy_from_slice(&resolved[..len]);
                    return Some(result);
                }
                None
            }
            RunfilesMode::DirectoryBased(dir, dir_len) => {
                let mut result = [0u8; MAX_PATH_LEN];
                let mut pos = 0;

                // Copy directory
                let copy_len = (*dir_len).min(MAX_PATH_LEN);
                result[..copy_len].copy_from_slice(&dir[..copy_len]);
                pos += copy_len;

                // Add separator if needed
                if pos < MAX_PATH_LEN && pos > 0 && result[pos - 1] != b'/' {
                    result[pos] = b'/';
                    pos += 1;
                }

                // Copy path
                let path_len = path.len().min(MAX_PATH_LEN - pos);
                result[pos..pos + path_len].copy_from_slice(&path[..path_len]);

                Some(result)
            }
        }
    }
}

// Placeholders for stub runner (will be replaced in final binary)
const ARG_SIZE: usize = 256;

#[used]
#[link_section = "__DATA,__runfiles"]
static mut ARGC_PLACEHOLDER: [u8; 32] = *b"@@RUNFILES_ARGC@@\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

#[used]
#[link_section = "__DATA,__runfiles"]
static mut TRANSFORM_FLAGS: [u8; 32] = *b"@@RUNFILES_TRANSFORM_FLAGS@@\0\0\0\0";

#[used]
#[link_section = "__DATA,__runfiles"]
static mut ARG0_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = "__DATA,__runfiles"]
static mut ARG1_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = "__DATA,__runfiles"]
static mut ARG2_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = "__DATA,__runfiles"]
static mut ARG3_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = "__DATA,__runfiles"]
static mut ARG4_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = "__DATA,__runfiles"]
static mut ARG5_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = "__DATA,__runfiles"]
static mut ARG6_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = "__DATA,__runfiles"]
static mut ARG7_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = "__DATA,__runfiles"]
static mut ARG8_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = "__DATA,__runfiles"]
static mut ARG9_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

// Get the length of a null-terminated string
fn strlen(s: &[u8]) -> usize {
    let mut len = 0;
    while len < s.len() && s[len] != 0 {
        len += 1;
    }
    len
}

// Check if placeholder is still in template state
fn is_template_placeholder(placeholder: &[u8]) -> bool {
    if placeholder.len() < 17 {
        return false;
    }
    str_starts_with(placeholder, b"@@RUNFILES_")
}

#[no_mangle]
pub extern "C" fn main(runtime_argc: i32, runtime_argv: *const *const u8) -> ! {
    unsafe {
        // Check if ARGC is still a placeholder
        if is_template_placeholder(&ARGC_PLACEHOLDER) {
            print(b"ERROR: This is a template stub runner.\n");
            print(b"You must finalize it by replacing the placeholders before use.\n");
            print(b"The ARGC_PLACEHOLDER has not been replaced.\n");
            exit(1);
        }

        // Parse argc from placeholder
        let argc_str = &ARGC_PLACEHOLDER;
        let argc_len = strlen(argc_str);
        if argc_len == 0 {
            print(b"ERROR: ARGC is empty\n");
            exit(1);
        }

        // Parse argc as decimal number
        let mut argc: usize = 0;
        for i in 0..argc_len {
            let c = argc_str[i];
            if c >= b'0' && c <= b'9' {
                argc = argc * 10 + (c - b'0') as usize;
            } else {
                print(b"ERROR: ARGC contains non-digit characters\n");
                exit(1);
            }
        }

        if argc == 0 || argc > 10 {
            print(b"ERROR: Invalid argc (must be 1-10)\n");
            exit(1);
        }

        // Parse transform flags (bitmask of which args to transform)
        let flags_str = &TRANSFORM_FLAGS;
        let flags_len = strlen(flags_str);
        let mut transform_flags: u32 = 0;

        if !is_template_placeholder(flags_str) && flags_len > 0 {
            // Parse as decimal number (bitmask)
            for i in 0..flags_len {
                let c = flags_str[i];
                if c >= b'0' && c <= b'9' {
                    transform_flags = transform_flags * 10 + (c - b'0') as u32;
                } else {
                    print(b"ERROR: TRANSFORM_FLAGS contains non-digit characters\n");
                    exit(1);
                }
            }
        }
        // If flags not set, default to transforming all args
        if flags_len == 0 || is_template_placeholder(flags_str) {
            transform_flags = 0xFFFFFFFF; // Transform all by default
        }

        // Check if any arguments need transformation
        let argc_mask = if argc >= 32 {
            0xFFFFFFFF
        } else {
            (1u32 << argc) - 1
        };
        let needs_runfiles = (transform_flags & argc_mask) != 0;

        // Initialize runfiles only if needed
        let runfiles = if needs_runfiles {
            if let Some(rf) = Runfiles::create() {
                Some(rf)
            } else {
                print(b"ERROR: Failed to initialize runfiles\n");
                print(b"Set RUNFILES_DIR or RUNFILES_MANIFEST_FILE\n");
                exit(1);
            }
        } else {
            None
        };

        // Get arg placeholders
        let arg_placeholders: [&[u8; ARG_SIZE]; 10] = [
            &ARG0_PLACEHOLDER,
            &ARG1_PLACEHOLDER,
            &ARG2_PLACEHOLDER,
            &ARG3_PLACEHOLDER,
            &ARG4_PLACEHOLDER,
            &ARG5_PLACEHOLDER,
            &ARG6_PLACEHOLDER,
            &ARG7_PLACEHOLDER,
            &ARG8_PLACEHOLDER,
            &ARG9_PLACEHOLDER,
        ];

        // Storage for resolved paths (embedded args + runtime args)
        // We need space for embedded args (up to 10) plus runtime args (runtime_argc - 1, excluding stub path)
        let mut resolved_paths: [[u8; MAX_PATH_LEN]; 128] = [[0; MAX_PATH_LEN]; 128];
        let mut resolved_ptrs: [*const u8; 129] = [core::ptr::null(); 129];
        let mut total_argc = 0usize;

        // Resolve embedded arguments
        for i in 0..argc {
            let arg_data = arg_placeholders[i];
            let arg_len = strlen(arg_data);

            if arg_len == 0 {
                print(b"ERROR: Argument ");
                let digit = [b'0' + i as u8];
                print(&digit);
                print(b" is empty\n");
                exit(1);
            }

            let arg_slice = &arg_data[..arg_len];

            // Check if this argument should be transformed
            let should_transform = (transform_flags & (1 << i)) != 0;

            if should_transform {
                // Try to resolve through runfiles
                if let Some(ref rf) = runfiles {
                    if let Some(resolved) = rf.rlocation(arg_slice) {
                        resolved_paths[i] = resolved;
                    } else {
                        // If not found in runfiles, use the path as-is
                        let copy_len = arg_len.min(MAX_PATH_LEN);
                        resolved_paths[i][..copy_len].copy_from_slice(&arg_slice[..copy_len]);
                    }
                } else {
                    // Use path as-is
                    let copy_len = arg_len.min(MAX_PATH_LEN);
                    resolved_paths[i][..copy_len].copy_from_slice(&arg_slice[..copy_len]);
                }
            } else {
                // Use path as-is without transformation
                let copy_len = arg_len.min(MAX_PATH_LEN);
                resolved_paths[i][..copy_len].copy_from_slice(&arg_slice[..copy_len]);
            }

            resolved_ptrs[i] = resolved_paths[i].as_ptr();
        }
        total_argc = argc;

        // Append runtime arguments (skip argv[0] which is the stub itself)
        if runtime_argc > 1 {
            for i in 1..runtime_argc as usize {
                if total_argc >= 128 {
                    print(b"ERROR: Too many total arguments (embedded + runtime > 128)\n");
                    exit(1);
                }

                // Get runtime argument
                let runtime_arg_ptr = *runtime_argv.add(i);

                // Find length of runtime argument
                let mut arg_len = 0;
                while *runtime_arg_ptr.add(arg_len) != 0 && arg_len < MAX_PATH_LEN {
                    arg_len += 1;
                }

                // Copy runtime argument to resolved_paths
                let copy_len = arg_len.min(MAX_PATH_LEN);
                let runtime_arg_slice = core::slice::from_raw_parts(runtime_arg_ptr, copy_len);
                resolved_paths[total_argc][..copy_len].copy_from_slice(runtime_arg_slice);

                resolved_ptrs[total_argc] = resolved_paths[total_argc].as_ptr();
                total_argc += 1;
            }
        }

        // NULL-terminate the argv array
        resolved_ptrs[total_argc] = core::ptr::null();

        // Get the executable path (first argument)
        let executable = resolved_ptrs[0];

        // Execute the target program with the original environment
        let ret = execve(executable, resolved_ptrs.as_ptr(), environ);

        // If execve returns, it failed
        print(b"ERROR: execve failed with code ");
        let digit = if ret < 0 {
            print(b"-");
            (-ret) as u8 + b'0'
        } else {
            ret as u8 + b'0'
        };
        print(&[digit]);
        print(b"\n");
        exit(1);
    }
}
