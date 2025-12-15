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
const INFINITE: DWORD = 0xFFFFFFFF;
const CREATE_UNICODE_ENVIRONMENT: DWORD = 0x00000400;

// STARTUPINFOW structure (wide char version for CreateProcessW)
#[repr(C)]
struct STARTUPINFOW {
    cb: DWORD,
    lpReserved: *mut u16,
    lpDesktop: *mut u16,
    lpTitle: *mut u16,
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
        lpStartupInfo: *mut STARTUPINFOW,
        lpProcessInformation: *mut PROCESS_INFORMATION,
    ) -> BOOL;
    fn GetCommandLineW() -> *const u16;
    fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    fn GetExitCodeProcess(hProcess: HANDLE, lpExitCode: *mut DWORD) -> BOOL;
    fn GetLastError() -> DWORD;
}

// We don't use CommandLineToArgvW to avoid shell32.dll dependency
// Instead we implement custom command-line parsing following Windows rules

// Parse Windows command line into arguments
// Returns number of arguments parsed (excluding argv[0])
// Stores argument pointers in output array
fn parse_command_line(
    cmdline: *const u16,
    argv_out: &mut [*const u16; 128],
    argv_len_out: &mut [usize; 128],
) -> usize {
    unsafe {
        let mut pos = 0usize;
        let mut argc = 0usize;

        // Skip leading whitespace
        while *cmdline.add(pos) != 0 && (*cmdline.add(pos) == b' ' as u16 || *cmdline.add(pos) == b'\t' as u16) {
            pos += 1;
        }

        // Skip argv[0] (executable path)
        let quoted = *cmdline.add(pos) == b'"' as u16;
        if quoted {
            pos += 1; // Skip opening quote
            while *cmdline.add(pos) != 0 && *cmdline.add(pos) != b'"' as u16 {
                pos += 1;
            }
            if *cmdline.add(pos) == b'"' as u16 {
                pos += 1; // Skip closing quote
            }
        } else {
            while *cmdline.add(pos) != 0 && *cmdline.add(pos) != b' ' as u16 && *cmdline.add(pos) != b'\t' as u16 {
                pos += 1;
            }
        }

        // Parse remaining arguments
        while *cmdline.add(pos) != 0 && argc < 128 {
            // Skip whitespace
            while *cmdline.add(pos) != 0 && (*cmdline.add(pos) == b' ' as u16 || *cmdline.add(pos) == b'\t' as u16) {
                pos += 1;
            }

            if *cmdline.add(pos) == 0 {
                break;
            }

            // Start of argument
            let arg_start = pos;
            let in_quotes = *cmdline.add(pos) == b'"' as u16;

            if in_quotes {
                pos += 1; // Skip opening quote
                // Find closing quote
                while *cmdline.add(pos) != 0 && *cmdline.add(pos) != b'"' as u16 {
                    pos += 1;
                }
                // Store argument (skip quotes in length calculation)
                argv_out[argc] = cmdline.add(arg_start + 1);
                argv_len_out[argc] = pos - arg_start - 1;

                if *cmdline.add(pos) == b'"' as u16 {
                    pos += 1; // Skip closing quote
                }
            } else {
                // Unquoted argument - find whitespace
                while *cmdline.add(pos) != 0 && *cmdline.add(pos) != b' ' as u16 && *cmdline.add(pos) != b'\t' as u16 {
                    pos += 1;
                }
                argv_out[argc] = cmdline.add(arg_start);
                argv_len_out[argc] = pos - arg_start;
            }

            argc += 1;
        }

        argc
    }
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

// Manifest entry storage - use static buffers to avoid stack overflow
// Windows has a default 1MB stack limit, so we store large data in .bss
const MAX_ENTRIES: usize = 256;  // Reduced from 1024 to save memory
const MAX_PATH_LEN: usize = 512; // Increased to support longer Windows paths

// Static storage for manifest data (in .bss segment, not stack)
static mut MANIFEST_KEYS: [[u8; MAX_PATH_LEN]; MAX_ENTRIES] = [[0; MAX_PATH_LEN]; MAX_ENTRIES];
static mut MANIFEST_VALUES: [[u8; MAX_PATH_LEN]; MAX_ENTRIES] = [[0; MAX_PATH_LEN]; MAX_ENTRIES];
static mut MANIFEST_KEY_LENS: [usize; MAX_ENTRIES] = [0; MAX_ENTRIES];
static mut MANIFEST_VALUE_LENS: [usize; MAX_ENTRIES] = [0; MAX_ENTRIES];
static mut MANIFEST_COUNT: usize = 0;

// Static storage for file buffer
static mut FILE_BUF: [u8; 65536] = [0; 65536];

// Static storage for resolved paths
static mut RESOLVED_PATHS: [[u8; MAX_PATH_LEN]; 128] = [[0; MAX_PATH_LEN]; 128];

struct Manifest {
    // Empty struct - all data is in statics
}

impl Manifest {
    fn reset() {
        unsafe {
            MANIFEST_COUNT = 0;
            // No need to zero the arrays - we track lengths
        }
    }

    fn add_entry(key: &[u8], value: &[u8]) {
        unsafe {
            if MANIFEST_COUNT >= MAX_ENTRIES {
                return;
            }

            let idx = MANIFEST_COUNT;
            let key_len = key.len().min(MAX_PATH_LEN);
            let value_len = value.len().min(MAX_PATH_LEN);

            MANIFEST_KEYS[idx][..key_len].copy_from_slice(&key[..key_len]);
            MANIFEST_KEY_LENS[idx] = key_len;
            MANIFEST_VALUES[idx][..value_len].copy_from_slice(&value[..value_len]);
            MANIFEST_VALUE_LENS[idx] = value_len;

            MANIFEST_COUNT += 1;
        }
    }

    fn lookup(key: &[u8]) -> Option<&'static [u8]> {
        unsafe {
            for i in 0..MANIFEST_COUNT {
                let entry_key = &MANIFEST_KEYS[i][..MANIFEST_KEY_LENS[i]];
                if str_eq(entry_key, key) {
                    return Some(&MANIFEST_VALUES[i][..MANIFEST_VALUE_LENS[i]]);
                }
            }
            None
        }
    }
}

// Load manifest file - uses static FILE_BUF to avoid stack overflow
fn load_manifest(path: &[u8]) -> Option<Manifest> {
    unsafe {
        // Reset manifest state
        Manifest::reset();

        // Ensure path is null-terminated
        let mut path_with_null = [0u8; 1024];
        let path_len = path.len().min(1023);
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

        // Use static FILE_BUF instead of stack allocation
        let mut bytes_read: DWORD = 0;
        let success = ReadFile(
            handle,
            FILE_BUF.as_mut_ptr() as LPVOID,
            FILE_BUF.len() as DWORD,
            &mut bytes_read,
            core::ptr::null_mut(),
        );
        CloseHandle(handle);

        if success == 0 || bytes_read == 0 {
            return None;
        }

        let data = &FILE_BUF[..bytes_read as usize];
        let mut pos = 0;

        while pos < data.len() {
            let line_start = pos;
            while pos < data.len() && data[pos] != b'\n' {
                pos += 1;
            }

            let line = &data[line_start..pos];

            if let Some(space_pos) = find_byte(line, b' ') {
                let key = &line[..space_pos];
                let mut value = &line[space_pos + 1..];

                // Strip trailing \r if present (Windows line endings)
                if !value.is_empty() && value[value.len() - 1] == b'\r' {
                    value = &value[..value.len() - 1];
                }

                Manifest::add_entry(key, value);
            }

            pos += 1;
        }

        Some(Manifest {})
    }
}

// Runfiles implementation
enum RunfilesMode {
    ManifestBased(Manifest),
    DirectoryBased([u8; MAX_PATH_LEN], usize),
}

struct Runfiles {
    mode: RunfilesMode,
    // Paths for environment variables (when export_runfiles_env is true)
    manifest_path: Option<([u8; MAX_PATH_LEN], usize)>, // RUNFILES_MANIFEST_FILE
    dir_path: Option<([u8; MAX_PATH_LEN], usize)>,      // RUNFILES_DIR and JAVA_RUNFILES
}

impl Runfiles {
    fn create(executable_path: Option<&[u8]>) -> Option<Self> {
        let mut manifest_path = [0u8; MAX_PATH_LEN];

        // Try RUNFILES_MANIFEST_FILE first
        if let Some(len) = get_env_var(b"RUNFILES_MANIFEST_FILE", &mut manifest_path) {
            if len > 0 {
                if let Some(manifest) = load_manifest(&manifest_path[..len]) {
                    return Some(Self {
                        mode: RunfilesMode::ManifestBased(manifest),
                        manifest_path: Some((manifest_path, len)),
                        dir_path: None,
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
                    manifest_path: None,
                    dir_path: Some((runfiles_dir, len)),
                });
            }
        }

        // Try to infer runfiles directory from executable path
        // Pattern: <executable_path>.runfiles\
        if let Some(exe_path) = executable_path {
            let exe_len = strlen(exe_path);
            if exe_len > 0 && exe_len + 10 < MAX_PATH_LEN {  // +10 for ".runfiles\0"
                let mut runfiles_dir = [0u8; MAX_PATH_LEN];

                // Copy executable path
                runfiles_dir[..exe_len].copy_from_slice(&exe_path[..exe_len]);

                // Append ".runfiles"
                runfiles_dir[exe_len..exe_len + 9].copy_from_slice(b".runfiles");
                runfiles_dir[exe_len + 9] = 0; // null terminator

                // Check if directory exists by trying to open it
                unsafe {
                    const FILE_FLAG_BACKUP_SEMANTICS: DWORD = 0x02000000;  // Needed to open directories
                    let handle = CreateFileA(
                        runfiles_dir.as_ptr(),
                        GENERIC_READ,
                        0,
                        core::ptr::null_mut(),
                        OPEN_EXISTING,
                        FILE_FLAG_BACKUP_SEMANTICS,
                        core::ptr::null_mut(),
                    );
                    if handle != INVALID_HANDLE_VALUE {
                        CloseHandle(handle);
                        // Remove null terminator for internal storage
                        return Some(Self {
                            mode: RunfilesMode::DirectoryBased(runfiles_dir, exe_len + 9),
                            manifest_path: None,
                            dir_path: Some((runfiles_dir, exe_len + 9)),
                        });
                    }
                }
            }
        }

        None
    }

    fn rlocation(&self, path: &[u8], result_idx: usize) -> Option<&'static [u8]> {
        // If path is absolute (Windows: starts with drive letter or \\), don't resolve
        if path.len() >= 2 && ((path[0].is_ascii_alphabetic() && path[1] == b':') || (path[0] == b'\\' && path[1] == b'\\')) {
            return None;
        }

        match &self.mode {
            RunfilesMode::ManifestBased(_manifest) => {
                // Use static lookup
                if let Some(resolved) = Manifest::lookup(path) {
                    unsafe {
                        let len = resolved.len().min(MAX_PATH_LEN);
                        // Copy path, converting forward slashes to backslashes
                        // Manifest values may contain Unix-style paths (forward slashes)
                        for i in 0..len {
                            RESOLVED_PATHS[result_idx][i] = if resolved[i] == b'/' { b'\\' } else { resolved[i] };
                        }
                        RESOLVED_PATHS[result_idx][len] = 0; // null terminate
                        return Some(&RESOLVED_PATHS[result_idx][..len]);
                    }
                }
                None
            }
            RunfilesMode::DirectoryBased(dir, dir_len) => {
                unsafe {
                    let mut pos = 0;

                    // Copy directory
                    let copy_len = (*dir_len).min(MAX_PATH_LEN);
                    RESOLVED_PATHS[result_idx][..copy_len].copy_from_slice(&dir[..copy_len]);
                    pos += copy_len;

                    // Add separator if needed
                    if pos < MAX_PATH_LEN && pos > 0 && RESOLVED_PATHS[result_idx][pos - 1] != b'\\' && RESOLVED_PATHS[result_idx][pos - 1] != b'/' {
                        RESOLVED_PATHS[result_idx][pos] = b'\\';
                        pos += 1;
                    }

                    // Copy path, converting forward slashes to backslashes
                    // Input is always Unix-style (a/b/c), output should be Windows-style (a\b\c)
                    let path_len = path.len().min(MAX_PATH_LEN - pos);
                    for i in 0..path_len {
                        RESOLVED_PATHS[result_idx][pos + i] = if path[i] == b'/' { b'\\' } else { path[i] };
                    }
                    let total_len = pos + path_len;
                    RESOLVED_PATHS[result_idx][total_len] = 0; // null terminate

                    Some(&RESOLVED_PATHS[result_idx][..total_len])
                }
            }
        }
    }
}

// Environment building for export mode
const MAX_ENV_SIZE: usize = 16384;
const MAX_ENV_VARS: usize = 256;

// External Windows API function for environment access
extern "system" {
    fn GetEnvironmentStringsW() -> *mut u16;
    fn FreeEnvironmentStringsW(lpszEnvironmentBlock: *mut u16) -> BOOL;
}

static mut MODIFIED_ENV_DATA: [u16; MAX_ENV_SIZE / 2] = [0; MAX_ENV_SIZE / 2];

fn build_runfiles_environ(runfiles: Option<&Runfiles>) -> *mut core::ffi::c_void {
    unsafe {
        let mut data_pos = 0usize;

        // Helper to add an environment variable (converts UTF-8 to UTF-16)
        let mut add_env_var = |key: &[u8], value: &[u8]| {
            let total_len = key.len() + 1 + value.len() + 1; // "KEY=VALUE\0" in UTF-16
            if data_pos + total_len > MODIFIED_ENV_DATA.len() {
                return false;
            }

            // Copy key (UTF-8 to UTF-16, simple ASCII conversion)
            for &byte in key {
                MODIFIED_ENV_DATA[data_pos] = byte as u16;
                data_pos += 1;
            }

            // Add '='
            MODIFIED_ENV_DATA[data_pos] = b'=' as u16;
            data_pos += 1;

            // Copy value
            for &byte in value {
                MODIFIED_ENV_DATA[data_pos] = byte as u16;
                data_pos += 1;
            }

            // Null terminate
            MODIFIED_ENV_DATA[data_pos] = 0;
            data_pos += 1;

            true
        };

        // Add RUNFILES_MANIFEST_FILE if we have it
        if let Some(rf) = runfiles {
            if let Some((ref path, len)) = rf.manifest_path {
                add_env_var(b"RUNFILES_MANIFEST_FILE", &path[..len]);
            }
        }

        // Add RUNFILES_DIR if we have it
        if let Some(rf) = runfiles {
            if let Some((ref path, len)) = rf.dir_path {
                add_env_var(b"RUNFILES_DIR", &path[..len]);
                add_env_var(b"JAVA_RUNFILES", &path[..len]);
            }
        }

        // Copy existing environment, filtering out runfiles vars
        let env_block = GetEnvironmentStringsW();
        if !env_block.is_null() {
            let mut pos = 0;

            // Environment block is a series of null-terminated UTF-16 strings
            loop {
                // Find the next null terminator
                let entry_start = pos;
                while *env_block.add(pos) != 0 {
                    pos += 1;
                    if pos > 16384 {  // Safety limit
                        break;
                    }
                }

                let entry_len = pos - entry_start;
                if entry_len == 0 {
                    // Empty string marks end of environment block
                    break;
                }

                // Check if this is a runfiles variable we should skip (compare in UTF-16)
                let entry_ptr = env_block.add(entry_start);
                let should_skip =
                    // Check for RUNFILES_MANIFEST_FILE=
                    (entry_len > 23 && {
                        let prefix = b"RUNFILES_MANIFEST_FILE=";
                        let mut matches = true;
                        for i in 0..23 {
                            if *entry_ptr.add(i) != prefix[i] as u16 {
                                matches = false;
                                break;
                            }
                        }
                        matches
                    }) ||
                    // Check for RUNFILES_DIR=
                    (entry_len > 13 && {
                        let prefix = b"RUNFILES_DIR=";
                        let mut matches = true;
                        for i in 0..13 {
                            if *entry_ptr.add(i) != prefix[i] as u16 {
                                matches = false;
                                break;
                            }
                        }
                        matches
                    }) ||
                    // Check for JAVA_RUNFILES=
                    (entry_len > 14 && {
                        let prefix = b"JAVA_RUNFILES=";
                        let mut matches = true;
                        for i in 0..14 {
                            if *entry_ptr.add(i) != prefix[i] as u16 {
                                matches = false;
                                break;
                            }
                        }
                        matches
                    });

                if !should_skip {
                    // Copy this environment variable
                    if data_pos + entry_len + 1 <= MODIFIED_ENV_DATA.len() {
                        for i in 0..entry_len {
                            MODIFIED_ENV_DATA[data_pos + i] = *entry_ptr.add(i);
                        }
                        MODIFIED_ENV_DATA[data_pos + entry_len] = 0;
                        data_pos += entry_len + 1;
                    }
                }

                pos += 1; // Skip past the null terminator
            }

            FreeEnvironmentStringsW(env_block);
        }

        // Add final null terminator to mark end of environment block
        if data_pos < MODIFIED_ENV_DATA.len() {
            MODIFIED_ENV_DATA[data_pos] = 0;
        }

        MODIFIED_ENV_DATA.as_mut_ptr() as *mut core::ffi::c_void
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
static mut EXPORT_RUNFILES_ENV: [u8; 32] = *b"@@RUNFILES_EXPORT_ENV@@\0\0\0\0\0\0\0\0\0";

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
        // Get command line
        let cmdline = GetCommandLineW();

        // Parse runtime arguments using custom parser (no shell32.dll needed)
        let mut runtime_argv: [*const u16; 128] = [core::ptr::null(); 128];
        let mut runtime_argv_len: [usize; 128] = [0; 128];
        let runtime_args_count = parse_command_line(cmdline, &mut runtime_argv, &mut runtime_argv_len);

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

        // Parse export_runfiles_env flag (defaults to true)
        let export_str = &EXPORT_RUNFILES_ENV;
        let export_len = strlen(export_str);
        let export_runfiles_env = if !is_template_placeholder(export_str) && export_len > 0 {
            // Parse as "1" (true) or "0" (false)
            export_str[0] != b'0'
        } else {
            true // Default to true
        };

        // Check if any arguments need transformation
        let argc_mask = if argc >= 32 {
            0xFFFFFFFF
        } else {
            (1u32 << argc) - 1
        };
        let needs_transform = (transform_flags & argc_mask) != 0;
        let needs_runfiles = needs_transform || export_runfiles_env;

        // Parse argv[0] from command line manually
        // Command line format: either "path\to\exe" args... or path\to\exe args...
        // We extract the first token (argv[0]) for runfiles fallback
        let mut exe_path_buf = [0u8; MAX_PATH_LEN];
        let mut exe_len = 0;
        let mut pos = 0usize;

        // Skip leading whitespace
        while *cmdline.add(pos) != 0 && (*cmdline.add(pos) == b' ' as u16 || *cmdline.add(pos) == b'\t' as u16) {
            pos += 1;
        }

        // Check if first char is a quote
        let quoted = *cmdline.add(pos) == b'"' as u16;
        if quoted {
            pos += 1; // Skip opening quote
        }

        // Extract argv[0]
        while exe_len < MAX_PATH_LEN && *cmdline.add(pos) != 0 {
            let wchar = *cmdline.add(pos);

            // Check for end of argv[0]
            if quoted {
                if wchar == b'"' as u16 {
                    break; // End of quoted string
                }
            } else {
                if wchar == b' ' as u16 || wchar == b'\t' as u16 {
                    break; // End of unquoted string
                }
            }

            // Simple UTF-16 to ASCII conversion
            exe_path_buf[exe_len] = (wchar & 0xFF) as u8;
            exe_len += 1;
            pos += 1;
        }

        let executable_path = if exe_len > 0 {
            Some(&exe_path_buf[..exe_len] as &[u8])
        } else {
            None
        };

        // Initialize runfiles only if needed
        let runfiles = if needs_runfiles {
            if let Some(rf) = Runfiles::create(executable_path) {
                Some(rf)
            } else {
                print(b"ERROR: Failed to initialize runfiles\r\n");
                print(b"Set RUNFILES_DIR or RUNFILES_MANIFEST_FILE, or ensure <executable>.runfiles\\ directory exists\r\n");
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

        // Resolve embedded arguments - uses static RESOLVED_PATHS
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
                    if rf.rlocation(arg_slice, i).is_none() {
                        // If not found in runfiles, use the path as-is
                        let copy_len = arg_len.min(MAX_PATH_LEN);
                        RESOLVED_PATHS[i][..copy_len].copy_from_slice(&arg_slice[..copy_len]);
                        RESOLVED_PATHS[i][copy_len] = 0;
                    }
                    // else: rlocation already wrote to RESOLVED_PATHS[i]
                } else {
                    // Use path as-is
                    let copy_len = arg_len.min(MAX_PATH_LEN);
                    RESOLVED_PATHS[i][..copy_len].copy_from_slice(&arg_slice[..copy_len]);
                    RESOLVED_PATHS[i][copy_len] = 0;
                }
            } else {
                // Use path as-is without transformation
                let copy_len = arg_len.min(MAX_PATH_LEN);
                RESOLVED_PATHS[i][..copy_len].copy_from_slice(&arg_slice[..copy_len]);
                RESOLVED_PATHS[i][copy_len] = 0;
            }
        }

        // Build command line for CreateProcessW (UTF-16)
        // Command line includes embedded args + runtime args
        let mut cmdline_wide = [0u16; 8192]; // Large buffer for UTF-16
        let mut cmdline_pos = 0usize;

        // Add embedded arguments (convert from UTF-8 to UTF-16)
        for i in 0..argc {
            let arg_len = strlen(&RESOLVED_PATHS[i]);
            let arg_slice = &RESOLVED_PATHS[i][..arg_len];

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
            let runtime_arg = runtime_argv[i];
            let arg_len = runtime_argv_len[i];

            // Check if we need quotes (scan for spaces)
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

        // Build environment with runfiles variables if export is enabled
        let envp = if export_runfiles_env {
            build_runfiles_environ(runfiles.as_ref())
        } else {
            core::ptr::null_mut()
        };

        // Create the process
        let mut si: STARTUPINFOW = core::mem::zeroed();
        si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
        let mut pi: PROCESS_INFORMATION = core::mem::zeroed();

        // Determine creation flags
        // If we have a UTF-16 environment block, we need CREATE_UNICODE_ENVIRONMENT
        let creation_flags = if export_runfiles_env {
            CREATE_UNICODE_ENVIRONMENT
        } else {
            0
        };

        // Pass NULL for lpApplicationName and put everything in lpCommandLine
        // This is the recommended approach for CreateProcessW
        let success = CreateProcessW(
            core::ptr::null(),          // Application name (NULL - use command line)
            cmdline_wide.as_mut_ptr(),  // Command line (UTF-16) - includes argv[0]
            core::ptr::null_mut(),      // Process attributes
            core::ptr::null_mut(),      // Thread attributes
            1,                          // Inherit handles
            creation_flags,             // Creation flags (with CREATE_UNICODE_ENVIRONMENT if needed)
            envp,                       // Environment
            core::ptr::null(),          // Current directory
            &mut si,
            &mut pi,
        );

        if success == 0 {
            print(b"ERROR: CreateProcess failed\r\n");
            ExitProcess(1);
        }

        // Wait for the child process to complete
        WaitForSingleObject(pi.hProcess, INFINITE);

        // Get the child process's exit code
        let mut exit_code: DWORD = 0;
        GetExitCodeProcess(pi.hProcess, &mut exit_code);

        // Close handles
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);

        // Exit with the child process's exit code
        ExitProcess(exit_code);
    }
}
