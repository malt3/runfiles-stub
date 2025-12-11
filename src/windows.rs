// Windows-specific implementation using Windows API
// Uses kernel32.dll functions

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { ExitProcess(1) }
}

// Windows API types
type DWORD = u32;
type BOOL = i32;
type HANDLE = *mut core::ffi::c_void;
type LPVOID = *mut core::ffi::c_void;
type LPCSTR = *const u8;
type LPSTR = *mut u8;

const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
const STD_OUTPUT_HANDLE: DWORD = 0xFFFFFFF5u32;
const GENERIC_READ: DWORD = 0x80000000;
const OPEN_EXISTING: DWORD = 3;
const FILE_ATTRIBUTE_NORMAL: DWORD = 0x80;

// STARTUPINFOA structure
#[repr(C)]
struct STARTUPINFOA {
    cb: DWORD,
    lpReserved: LPSTR,
    lpDesktop: LPSTR,
    lpTitle: LPSTR,
    dwX: DWORD,
    dwY: DWORD,
    dwXSize: DWORD,
    dwYSize: DWORD,
    dwXCountChars: DWORD,
    dwYCountChars: DWORD,
    dwFillAttribute: DWORD,
    dwFlags: DWORD,
    wShowWindow: u16,
    cbReserved2: u16,
    lpReserved2: *mut u8,
    hStdInput: HANDLE,
    hStdOutput: HANDLE,
    hStdError: HANDLE,
}

// PROCESS_INFORMATION structure
#[repr(C)]
struct PROCESS_INFORMATION {
    hProcess: HANDLE,
    hThread: HANDLE,
    dwProcessId: DWORD,
    dwThreadId: DWORD,
}

// External Windows API functions (kernel32.dll)
extern "system" {
    fn ExitProcess(exit_code: u32) -> !;
    fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
    fn WriteFile(
        hFile: HANDLE,
        lpBuffer: *const u8,
        nNumberOfBytesToWrite: DWORD,
        lpNumberOfBytesWritten: *mut DWORD,
        lpOverlapped: LPVOID,
    ) -> BOOL;
    fn CreateFileA(
        lpFileName: LPCSTR,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: LPVOID,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;
    fn ReadFile(
        hFile: HANDLE,
        lpBuffer: LPVOID,
        nNumberOfBytesToRead: DWORD,
        lpNumberOfBytesRead: *mut DWORD,
        lpOverlapped: LPVOID,
    ) -> BOOL;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn GetEnvironmentVariableA(lpName: LPCSTR, lpBuffer: LPSTR, nSize: DWORD) -> DWORD;
    fn CreateProcessW(
        lpApplicationName: *const u16,
        lpCommandLine: *mut u16,
        lpProcessAttributes: LPVOID,
        lpThreadAttributes: LPVOID,
        bInheritHandles: BOOL,
        dwCreationFlags: DWORD,
        lpEnvironment: LPVOID,
        lpCurrentDirectory: *const u16,
        lpStartupInfo: *mut STARTUPINFOA,
        lpProcessInformation: *mut PROCESS_INFORMATION,
    ) -> BOOL;
    fn GetCommandLineW() -> *const u16;
    fn LocalFree(hMem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
}

// External Windows API functions (shell32.dll)
#[link(name = "shell32")]
extern "system" {
    fn CommandLineToArgvW(lpCmdLine: *const u16, pNumArgs: *mut i32) -> *mut *mut u16;
}

// String utilities
fn print(s: &[u8]) {
    unsafe {
        let stdout = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut written: DWORD = 0;
        WriteFile(
            stdout,
            s.as_ptr(),
            s.len() as DWORD,
            &mut written,
            core::ptr::null_mut(),
        );
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

// Environment variable reading
fn get_env_var(name: &[u8], buf: &mut [u8]) -> Option<usize> {
    unsafe {
        // Ensure name is null-terminated
        let mut name_with_null = [0u8; 256];
        let name_len = name.len().min(255);
        name_with_null[..name_len].copy_from_slice(&name[..name_len]);
        name_with_null[name_len] = 0;

        let size = GetEnvironmentVariableA(
            name_with_null.as_ptr(),
            buf.as_mut_ptr(),
            buf.len() as DWORD,
        );

        if size > 0 && size < buf.len() as DWORD {
            Some(size as usize)
        } else {
            None
        }
    }
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
        // Ensure path is null-terminated
        let mut path_with_null = [0u8; MAX_PATH_LEN + 1];
        let path_len = path.len().min(MAX_PATH_LEN);
        path_with_null[..path_len].copy_from_slice(&path[..path_len]);
        path_with_null[path_len] = 0;

        let handle = CreateFileA(
            path_with_null.as_ptr(),
            GENERIC_READ,
            0,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            core::ptr::null_mut(),
        );

        if handle == INVALID_HANDLE_VALUE {
            return None;
        }

        let mut file_buf = [0u8; 65536];
        let mut bytes_read: DWORD = 0;
        let success = ReadFile(
            handle,
            file_buf.as_mut_ptr() as LPVOID,
            file_buf.len() as DWORD,
            &mut bytes_read,
            core::ptr::null_mut(),
        );
        CloseHandle(handle);

        if success == 0 || bytes_read == 0 {
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
                if let Some(manifest) = load_manifest(&manifest_path[..len]) {
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
        // If path is absolute (Windows: starts with drive letter or \\), don't resolve
        if path.len() >= 2 && ((path[0].is_ascii_alphabetic() && path[1] == b':') || (path[0] == b'\\' && path[1] == b'\\')) {
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
                if pos < MAX_PATH_LEN && pos > 0 && result[pos - 1] != b'\\' && result[pos - 1] != b'/' {
                    result[pos] = b'\\';
                    pos += 1;
                }

                // Copy path, converting forward slashes to backslashes
                // Input is always Unix-style (a/b/c), output should be Windows-style (a\b\c)
                let path_len = path.len().min(MAX_PATH_LEN - pos);
                for i in 0..path_len {
                    result[pos + i] = if path[i] == b'/' { b'\\' } else { path[i] };
                }

                Some(result)
            }
        }
    }
}

// Placeholders for stub runner (will be replaced in final binary)
const ARG_SIZE: usize = 256;

#[used]
#[link_section = ".runfiles"]
static mut ARGC_PLACEHOLDER: [u8; 32] = *b"@@RUNFILES_ARGC@@\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

#[used]
#[link_section = ".runfiles"]
static mut TRANSFORM_FLAGS: [u8; 32] = *b"@@RUNFILES_TRANSFORM_FLAGS@@\0\0\0\0";

#[used]
#[link_section = ".runfiles"]
static mut ARG0_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles"]
static mut ARG1_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles"]
static mut ARG2_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles"]
static mut ARG3_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles"]
static mut ARG4_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles"]
static mut ARG5_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles"]
static mut ARG6_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles"]
static mut ARG7_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles"]
static mut ARG8_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles"]
static mut ARG9_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

// Get the length of a null-terminated string
fn strlen(s: &[u8]) -> usize {
    let mut len = 0;
    while len < s.len() && s[len] != 0 {
        len += 1;
    }
    len
}

// Get the length of a null-terminated wide string
fn wstrlen(s: *const u16) -> usize {
    let mut len = 0;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}

// Convert UTF-8 to UTF-16 (simplified, ASCII-compatible only)
fn utf8_to_wide(utf8: &[u8], out: &mut [u16]) -> usize {
    let mut out_len = 0;
    for i in 0..utf8.len() {
        if out_len >= out.len() {
            break;
        }
        if utf8[i] == 0 {
            break;
        }
        // Simple conversion: assume ASCII range
        out[out_len] = utf8[i] as u16;
        out_len += 1;
    }
    out_len
}

// Check if placeholder is still in template state
fn is_template_placeholder(placeholder: &[u8]) -> bool {
    if placeholder.len() < 17 {
        return false;
    }
    str_starts_with(placeholder, b"@@RUNFILES_")
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    unsafe {
        // Parse runtime arguments from command line (keep as UTF-16)
        let cmdline = GetCommandLineW();
        let mut runtime_argc: i32 = 0;
        let runtime_argv_wide = CommandLineToArgvW(cmdline, &mut runtime_argc);

        // Store pointers to runtime args (skip argv[0])
        let mut runtime_args: [*const u16; 128] = [core::ptr::null(); 128];
        let mut runtime_args_count = 0usize;

        if !runtime_argv_wide.is_null() && runtime_argc > 1 {
            for i in 1..runtime_argc as usize {
                if runtime_args_count >= 128 {
                    break;
                }
                runtime_args[runtime_args_count] = *runtime_argv_wide.add(i);
                runtime_args_count += 1;
            }
        }

        // Check if ARGC is still a placeholder
        if is_template_placeholder(&ARGC_PLACEHOLDER) {
            print(b"ERROR: This is a template stub runner.\r\n");
            print(b"You must finalize it by replacing the placeholders before use.\r\n");
            print(b"The ARGC_PLACEHOLDER has not been replaced.\r\n");
            ExitProcess(1);
        }

        // Parse argc from placeholder
        let argc_str = &ARGC_PLACEHOLDER;
        let argc_len = strlen(argc_str);
        if argc_len == 0 {
            print(b"ERROR: ARGC is empty\r\n");
            ExitProcess(1);
        }

        // Parse argc as decimal number
        let mut argc: usize = 0;
        for i in 0..argc_len {
            let c = argc_str[i];
            if c >= b'0' && c <= b'9' {
                argc = argc * 10 + (c - b'0') as usize;
            } else {
                print(b"ERROR: ARGC contains non-digit characters\r\n");
                ExitProcess(1);
            }
        }

        if argc == 0 || argc > 10 {
            print(b"ERROR: Invalid argc (must be 1-10)\r\n");
            ExitProcess(1);
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
                    print(b"ERROR: TRANSFORM_FLAGS contains non-digit characters\r\n");
                    ExitProcess(1);
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
                print(b"ERROR: Failed to initialize runfiles\r\n");
                print(b"Set RUNFILES_DIR or RUNFILES_MANIFEST_FILE\r\n");
                ExitProcess(1);
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
        let mut resolved_paths: [[u8; MAX_PATH_LEN]; 128] = [[0; MAX_PATH_LEN]; 128];
        let mut total_argc = 0usize;

        // Resolve embedded arguments
        for i in 0..argc {
            let arg_data = arg_placeholders[i];
            let arg_len = strlen(arg_data);

            if arg_len == 0 {
                print(b"ERROR: Argument ");
                let digit = [b'0' + i as u8];
                print(&digit);
                print(b" is empty\r\n");
                ExitProcess(1);
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
        }
        total_argc = argc;

        // Build command line for CreateProcessW (UTF-16)
        // Command line includes embedded args + runtime args
        let mut cmdline_wide = [0u16; 8192]; // Large buffer for UTF-16
        let mut cmdline_pos = 0usize;

        // Add embedded arguments (convert from UTF-8 to UTF-16)
        for i in 0..argc {
            let arg_len = strlen(&resolved_paths[i]);
            let arg_slice = &resolved_paths[i][..arg_len];

            // Check if we need quotes (if path contains spaces)
            let needs_quotes = find_byte(arg_slice, b' ').is_some();

            if needs_quotes && cmdline_pos < cmdline_wide.len() {
                cmdline_wide[cmdline_pos] = b'"' as u16;
                cmdline_pos += 1;
            }

            // Convert UTF-8 to UTF-16 and copy
            let converted_len = utf8_to_wide(arg_slice, &mut cmdline_wide[cmdline_pos..]);
            cmdline_pos += converted_len;

            if needs_quotes && cmdline_pos < cmdline_wide.len() {
                cmdline_wide[cmdline_pos] = b'"' as u16;
                cmdline_pos += 1;
            }

            // Add space between arguments
            if (i < argc - 1 || runtime_args_count > 0) && cmdline_pos < cmdline_wide.len() {
                cmdline_wide[cmdline_pos] = b' ' as u16;
                cmdline_pos += 1;
            }
        }

        // Add runtime arguments (already UTF-16, just copy)
        for i in 0..runtime_args_count {
            let runtime_arg = runtime_args[i];
            let arg_len = wstrlen(runtime_arg);

            // Check if we need quotes
            let mut needs_quotes = false;
            for j in 0..arg_len {
                if *runtime_arg.add(j) == b' ' as u16 {
                    needs_quotes = true;
                    break;
                }
            }

            if needs_quotes && cmdline_pos < cmdline_wide.len() {
                cmdline_wide[cmdline_pos] = b'"' as u16;
                cmdline_pos += 1;
            }

            // Copy wide string
            let copy_len = arg_len.min(cmdline_wide.len() - cmdline_pos);
            for j in 0..copy_len {
                cmdline_wide[cmdline_pos + j] = *runtime_arg.add(j);
            }
            cmdline_pos += copy_len;

            if needs_quotes && cmdline_pos < cmdline_wide.len() {
                cmdline_wide[cmdline_pos] = b'"' as u16;
                cmdline_pos += 1;
            }

            // Add space between arguments (except after last)
            if i < runtime_args_count - 1 && cmdline_pos < cmdline_wide.len() {
                cmdline_wide[cmdline_pos] = b' ' as u16;
                cmdline_pos += 1;
            }
        }

        // Null-terminate command line
        if cmdline_pos < cmdline_wide.len() {
            cmdline_wide[cmdline_pos] = 0;
        }

        // Convert first argument (executable path) to UTF-16 for CreateProcessW
        let mut exe_path_wide = [0u16; MAX_PATH_LEN];
        let exe_len = strlen(&resolved_paths[0]);
        utf8_to_wide(&resolved_paths[0][..exe_len], &mut exe_path_wide);

        // Create the process
        let mut si: STARTUPINFOA = core::mem::zeroed();
        si.cb = core::mem::size_of::<STARTUPINFOA>() as DWORD;
        let mut pi: PROCESS_INFORMATION = core::mem::zeroed();

        let success = CreateProcessW(
            exe_path_wide.as_ptr(),     // Application name (UTF-16)
            cmdline_wide.as_mut_ptr(),  // Command line (UTF-16)
            core::ptr::null_mut(),      // Process attributes
            core::ptr::null_mut(),      // Thread attributes
            1,                          // Inherit handles
            0,                          // Creation flags
            core::ptr::null_mut(),      // Environment
            core::ptr::null(),          // Current directory
            &mut si,
            &mut pi,
        );

        if success == 0 {
            print(b"ERROR: CreateProcess failed\r\n");
            ExitProcess(1);
        }

        // Close handles (we don't need them)
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);

        // Free the argv array allocated by CommandLineToArgvW
        if !runtime_argv_wide.is_null() {
            LocalFree(runtime_argv_wide as *mut core::ffi::c_void);
        }

        // Exit successfully - the child process is now running
        ExitProcess(0);
    }
}
