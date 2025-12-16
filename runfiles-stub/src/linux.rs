use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(1);
}

// Compiler intrinsics (memcpy, memset)
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.add(i) = c as u8;
        i += 1;
    }
    s
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0;
    while i < n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return a as i32 - b as i32;
        }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    memcmp(s1, s2, n)
}

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

// Syscall numbers - architecture specific
#[cfg(target_arch = "x86_64")]
mod syscall_numbers {
    pub const SYS_READ: usize = 0;
    pub const SYS_WRITE: usize = 1;
    pub const SYS_OPEN: usize = 2;
    pub const SYS_CLOSE: usize = 3;
    pub const SYS_ACCESS: usize = 21;
    pub const SYS_EXECVE: usize = 59;
    pub const SYS_EXIT: usize = 60;
}

#[cfg(target_arch = "aarch64")]
mod syscall_numbers {
    pub const SYS_READ: usize = 63;
    pub const SYS_WRITE: usize = 64;
    pub const SYS_OPENAT: usize = 56;  // openat is used on aarch64
    pub const SYS_CLOSE: usize = 57;
    pub const SYS_FACCESSAT: usize = 48;  // faccessat is used on aarch64
    pub const SYS_EXECVE: usize = 221;
    pub const SYS_EXIT: usize = 93;
    pub const AT_FDCWD: i32 = -100;  // Special fd for openat/faccessat to work like open/access
}

use syscall_numbers::*;

const O_RDONLY: i32 = 0;
const STDOUT: i32 = 1;

#[cfg(target_arch = "x86_64")]
fn exit(code: i32) -> ! {
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") SYS_EXIT,
            in("rdi") code,
            options(noreturn)
        );
    }
}

#[cfg(target_arch = "aarch64")]
fn exit(code: i32) -> ! {
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_EXIT,
            in("x0") code,
            options(noreturn)
        );
    }
}

#[cfg(target_arch = "x86_64")]
fn write(fd: i32, buf: &[u8]) -> isize {
    let ret: isize;
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") SYS_WRITE,
            in("rdi") fd,
            in("rsi") buf.as_ptr(),
            in("rdx") buf.len(),
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    ret
}

#[cfg(target_arch = "aarch64")]
fn write(fd: i32, buf: &[u8]) -> isize {
    let ret: isize;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_WRITE,
            in("x0") fd,
            in("x1") buf.as_ptr(),
            in("x2") buf.len(),
            lateout("x0") ret,
        );
    }
    ret
}

#[cfg(target_arch = "x86_64")]
fn open(path: &[u8]) -> i32 {
    let ret: i32;
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") SYS_OPEN,
            in("rdi") path.as_ptr(),
            in("rsi") O_RDONLY,
            in("rdx") 0,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    ret
}

#[cfg(target_arch = "aarch64")]
fn open(path: &[u8]) -> i32 {
    let ret: i32;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_OPENAT,
            in("x0") AT_FDCWD,
            in("x1") path.as_ptr(),
            in("x2") O_RDONLY,
            in("x3") 0,
            lateout("x0") ret,
        );
    }
    ret
}

#[cfg(target_arch = "x86_64")]
fn read(fd: i32, buf: &mut [u8]) -> isize {
    let ret: isize;
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") SYS_READ,
            in("rdi") fd,
            in("rsi") buf.as_ptr(),
            in("rdx") buf.len(),
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    ret
}

#[cfg(target_arch = "aarch64")]
fn read(fd: i32, buf: &mut [u8]) -> isize {
    let ret: isize;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_READ,
            in("x0") fd,
            in("x1") buf.as_ptr(),
            in("x2") buf.len(),
            lateout("x0") ret,
        );
    }
    ret
}

#[cfg(target_arch = "x86_64")]
fn close(fd: i32) {
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") SYS_CLOSE,
            in("rdi") fd,
            lateout("rax") _,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
}

#[cfg(target_arch = "aarch64")]
fn close(fd: i32) {
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_CLOSE,
            in("x0") fd,
            lateout("x0") _,
        );
    }
}

// Check if a path exists using access() syscall with F_OK (0)
#[cfg(target_arch = "x86_64")]
fn path_exists(path: &[u8]) -> bool {
    let ret: i32;
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") SYS_ACCESS,
            in("rdi") path.as_ptr(),
            in("rsi") 0i32,  // F_OK = 0 (check existence)
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    ret == 0
}

#[cfg(target_arch = "aarch64")]
fn path_exists(path: &[u8]) -> bool {
    let ret: i32;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_FACCESSAT,
            in("x0") AT_FDCWD,
            in("x1") path.as_ptr(),
            in("x2") 0i32,  // F_OK = 0 (check existence)
            in("x3") 0i32,  // flags = 0
            lateout("x0") ret,
        );
    }
    ret == 0
}

#[cfg(target_arch = "x86_64")]
fn execve(filename: *const u8, argv: *const *const u8, envp: *const *const u8) -> i32 {
    let ret: i32;
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") SYS_EXECVE,
            in("rdi") filename,
            in("rsi") argv,
            in("rdx") envp,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    ret
}

#[cfg(target_arch = "aarch64")]
fn execve(filename: *const u8, argv: *const *const u8, envp: *const *const u8) -> i32 {
    let ret: i32;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_EXECVE,
            in("x0") filename,
            in("x1") argv,
            in("x2") envp,
            lateout("x0") ret,
        );
    }
    ret
}

// String utilities
fn print(s: &[u8]) {
    write(STDOUT, s);
}

fn print_number(mut n: usize) {
    let mut buf = [0u8; 20]; // Enough for 64-bit numbers
    let mut i = 0;

    if n == 0 {
        write(STDOUT, b"0");
        return;
    }

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    // Print in reverse order
    while i > 0 {
        i -= 1;
        write(STDOUT, &buf[i..i+1]);
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

// Static buffer for reading environment during initialization
// Using a static buffer here to avoid stack overflow from large stack allocation
static mut GET_ENV_BUF: [u8; MAX_ENV_SIZE] = [0; MAX_ENV_SIZE];

// Environment variable reading
fn get_env_var(name: &[u8], buf: &mut [u8]) -> Option<usize> {
    let environ_buf = unsafe { &mut GET_ENV_BUF };
    let fd = open(b"/proc/self/environ\0");
    if fd < 0 {
        return None;
    }

    let bytes_read = read(fd, environ_buf);
    close(fd);

    if bytes_read <= 0 {
        return None;
    }

    let environ_data = &environ_buf[..bytes_read as usize];
    let mut pos = 0;

    while pos < environ_data.len() {
        let start = pos;
        while pos < environ_data.len() && environ_data[pos] != 0 {
            pos += 1;
        }

        let entry = &environ_data[start..pos];
        if let Some(eq_pos) = find_byte(entry, b'=') {
            let key = &entry[..eq_pos];
            let value = &entry[eq_pos + 1..];

            if str_eq(key, name) {
                let copy_len = value.len().min(buf.len());
                buf[..copy_len].copy_from_slice(&value[..copy_len]);
                return Some(copy_len);
            }
        }

        pos += 1;
    }

    None
}

// Manifest entry storage (simplified - using static arrays)
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
    let fd = open(path);
    if fd < 0 {
        return None;
    }

    let mut file_buf = [0u8; 65536];
    let bytes_read = read(fd, &mut file_buf);
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
                let mut path_with_null = [0u8; MAX_PATH_LEN + 1];
                path_with_null[..len].copy_from_slice(&manifest_path[..len]);

                if let Some(manifest) = load_manifest(&path_with_null[..len + 1]) {
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

        // Try to find runfiles next to the executable
        // Check for <executable>.runfiles_manifest file (preferred)
        // Then check for <executable>.runfiles directory
        if let Some(exe_path) = executable_path {
            let exe_len = str_len(exe_path);
            if exe_len > 0 {
                // Try <executable>.runfiles_manifest file first
                if exe_len + 19 < MAX_PATH_LEN {  // +19 for ".runfiles_manifest\0"
                    let mut manifest_file_path = [0u8; MAX_PATH_LEN + 1];

                    // Copy executable path
                    manifest_file_path[..exe_len].copy_from_slice(&exe_path[..exe_len]);

                    // Append ".runfiles_manifest" (18 characters)
                    manifest_file_path[exe_len..exe_len + 18].copy_from_slice(b".runfiles_manifest");
                    let manifest_file_len = exe_len + 18;

                    // Try to load the manifest file
                    if let Some(manifest) = load_manifest(&manifest_file_path[..manifest_file_len + 1]) {
                        // Also determine the runfiles directory for RUNFILES_DIR envvar
                        // The directory is <executable>.runfiles
                        let mut dir_path = [0u8; MAX_PATH_LEN];
                        let dir_len = if exe_len + 9 < MAX_PATH_LEN {
                            dir_path[..exe_len].copy_from_slice(&exe_path[..exe_len]);
                            dir_path[exe_len..exe_len + 9].copy_from_slice(b".runfiles");
                            exe_len + 9
                        } else {
                            0
                        };

                        let mut manifest_path_without_null = [0u8; MAX_PATH_LEN];
                        manifest_path_without_null[..manifest_file_len].copy_from_slice(&manifest_file_path[..manifest_file_len]);

                        return Some(Self {
                            mode: RunfilesMode::ManifestBased(manifest),
                            manifest_path: Some((manifest_path_without_null, manifest_file_len)),
                            dir_path: if dir_len > 0 { Some((dir_path, dir_len)) } else { None },
                        });
                    }
                }

                // Try <executable>.runfiles directory
                if exe_len + 10 < MAX_PATH_LEN {  // +10 for ".runfiles\0"
                    let mut runfiles_dir = [0u8; MAX_PATH_LEN];

                    // Copy executable path
                    runfiles_dir[..exe_len].copy_from_slice(&exe_path[..exe_len]);

                    // Append ".runfiles"
                    runfiles_dir[exe_len..exe_len + 9].copy_from_slice(b".runfiles");
                    runfiles_dir[exe_len + 9] = 0; // null terminator

                    // Check if directory exists using access() syscall
                    if path_exists(&runfiles_dir[..exe_len + 10]) {
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
// Each placeholder uses a distinctive pattern starting with @@RUNFILES_
const ARG_SIZE: usize = 256;

#[used]
#[link_section = ".runfiles_stubs"]
static mut ARGC_PLACEHOLDER: [u8; 32] = *b"@@RUNFILES_ARGC@@\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

#[used]
#[link_section = ".runfiles_stubs"]
static mut TRANSFORM_FLAGS: [u8; 32] = *b"@@RUNFILES_TRANSFORM_FLAGS@@\0\0\0\0";

#[used]
#[link_section = ".runfiles_stubs"]
static mut EXPORT_RUNFILES_ENV: [u8; 32] = *b"@@RUNFILES_EXPORT_ENV@@\0\0\0\0\0\0\0\0\0";

#[used]
#[link_section = ".runfiles_stubs"]
static mut ARG0_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles_stubs"]
static mut ARG1_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles_stubs"]
static mut ARG2_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles_stubs"]
static mut ARG3_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles_stubs"]
static mut ARG4_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles_stubs"]
static mut ARG5_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles_stubs"]
static mut ARG6_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles_stubs"]
static mut ARG7_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles_stubs"]
static mut ARG8_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

#[used]
#[link_section = ".runfiles_stubs"]
static mut ARG9_PLACEHOLDER: [u8; ARG_SIZE] = [b'@'; ARG_SIZE];

// Get the length of a null-terminated string (Rust-style, takes slice)
fn str_len(s: &[u8]) -> usize {
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

// Environment variable storage
// These limits are based on the Linux kernel's ARG_MAX and related limits for execve().
// Linux supports up to 6 MiB total for argv + envp combined, with a 2 MiB per-string limit.
// The actual limit is dynamic and derived from RLIMIT_STACK at execution time.
// See: include/uapi/linux/binfmts.h in the Linux kernel source
// Reference: https://gist.github.com/malt3/c1439aa16208a74194accb025ab1cc5b
const MAX_ENV_SIZE: usize = 6291456;  // 6 MiB - matches Linux upper bound for total args+env
const MAX_ENV_VARS: usize = 1024;     // Max number of environment variables

static mut ENVIRON_DATA: [u8; MAX_ENV_SIZE] = [0; MAX_ENV_SIZE];
static mut ENVIRON_PTRS: [*const u8; MAX_ENV_VARS + 1] = [core::ptr::null(); MAX_ENV_VARS + 1];

// Read and parse environment variables from /proc/self/environ
fn get_environ() -> *const *const u8 {
    unsafe {
        // Read environment from /proc/self/environ
        let fd = open(b"/proc/self/environ\0");
        if fd < 0 {
            // If we can't read environ, return empty environment
            ENVIRON_PTRS[0] = core::ptr::null();
            return ENVIRON_PTRS.as_ptr();
        }

        let bytes_read = read(fd, &mut ENVIRON_DATA);
        close(fd);

        if bytes_read <= 0 {
            ENVIRON_PTRS[0] = core::ptr::null();
            return ENVIRON_PTRS.as_ptr();
        }

        // Check if environment data was truncated
        let data_len = bytes_read as usize;
        if data_len >= MAX_ENV_SIZE {
            print(b"ERROR: Environment data exceeds buffer limit of ");
            print_number(MAX_ENV_SIZE);
            print(b" bytes\n");
            print(b"Environment was truncated. This indicates the total environment size is too large.\n");
            print(b"Consider reducing the number or size of environment variables.\n");
            exit(1);
        }

        // Parse environment variables (null-separated entries)
        let mut env_count = 0;
        let mut pos = 0;

        while pos < data_len && env_count < MAX_ENV_VARS {
            // Skip empty entries
            if ENVIRON_DATA[pos] == 0 {
                pos += 1;
                continue;
            }

            // Mark start of this environment variable
            ENVIRON_PTRS[env_count] = ENVIRON_DATA.as_ptr().add(pos);
            env_count += 1;

            // Find the end (null byte)
            while pos < data_len && ENVIRON_DATA[pos] != 0 {
                pos += 1;
            }

            // Move past the null byte
            pos += 1;
        }

        // Check if we hit the max number of environment variables
        if env_count >= MAX_ENV_VARS && pos < data_len {
            print(b"ERROR: Number of environment variables exceeds limit of ");
            print_number(MAX_ENV_VARS);
            print(b"\n");
            print(b"Consider reducing the number of environment variables.\n");
            exit(1);
        }

        // Null-terminate the pointer array
        ENVIRON_PTRS[env_count] = core::ptr::null();

        ENVIRON_PTRS.as_ptr()
    }
}

// Build modified environment with runfiles variables
// Storage for modified environment
static mut MODIFIED_ENV_DATA: [u8; MAX_ENV_SIZE] = [0; MAX_ENV_SIZE];
static mut MODIFIED_ENV_PTRS: [*const u8; MAX_ENV_VARS + 1] = [core::ptr::null(); MAX_ENV_VARS + 1];

fn build_runfiles_environ(runfiles: Option<&Runfiles>) -> *const *const u8 {
    unsafe {
        let base_env = get_environ();

        // If no runfiles info, just return base environment
        let rf = match runfiles {
            Some(r) => r,
            None => return base_env,
        };

        let mut new_env_count = 0;
        let mut data_pos = 0;

        // Helper to add an environment variable
        let mut add_env_var = |name: &[u8], value: &[u8]| {
            if data_pos + name.len() + 1 + value.len() + 1 > MAX_ENV_SIZE {
                return false; // Out of space
            }
            if new_env_count >= MAX_ENV_VARS {
                return false; // Too many vars
            }

            // Mark start of this var
            MODIFIED_ENV_PTRS[new_env_count] = MODIFIED_ENV_DATA.as_ptr().add(data_pos);
            new_env_count += 1;

            // Copy name=value
            MODIFIED_ENV_DATA[data_pos..data_pos + name.len()].copy_from_slice(name);
            data_pos += name.len();
            MODIFIED_ENV_DATA[data_pos] = b'=';
            data_pos += 1;
            MODIFIED_ENV_DATA[data_pos..data_pos + value.len()].copy_from_slice(value);
            data_pos += value.len();
            MODIFIED_ENV_DATA[data_pos] = 0;
            data_pos += 1;

            true
        };

        // Add runfiles environment variables first
        if let Some((path, len)) = rf.manifest_path {
            if !add_env_var(b"RUNFILES_MANIFEST_FILE", &path[..len]) {
                print(b"ERROR: Failed to add RUNFILES_MANIFEST_FILE to environment\n");
                print(b"Environment buffer limit exceeded. Total size limit: ");
                print_number(MAX_ENV_SIZE);
                print(b" bytes, max variables: ");
                print_number(MAX_ENV_VARS);
                print(b"\n");
                exit(1);
            }
        }

        if let Some((path, len)) = rf.dir_path {
            if !add_env_var(b"RUNFILES_DIR", &path[..len]) {
                print(b"ERROR: Failed to add RUNFILES_DIR to environment\n");
                print(b"Environment buffer limit exceeded. Total size limit: ");
                print_number(MAX_ENV_SIZE);
                print(b" bytes, max variables: ");
                print_number(MAX_ENV_VARS);
                print(b"\n");
                exit(1);
            }
            if !add_env_var(b"JAVA_RUNFILES", &path[..len]) {
                print(b"ERROR: Failed to add JAVA_RUNFILES to environment\n");
                print(b"Environment buffer limit exceeded. Total size limit: ");
                print_number(MAX_ENV_SIZE);
                print(b" bytes, max variables: ");
                print_number(MAX_ENV_VARS);
                print(b"\n");
                exit(1);
            }
        }

        // Copy existing environment (skip runfiles vars that we're setting)
        let mut i = 0;
        let mut env_dropped = false;
        while !(*base_env.add(i)).is_null() {
            let env_ptr = *base_env.add(i);
            let mut env_len = 0;
            while *env_ptr.add(env_len) != 0 {
                env_len += 1;
            }

            let env_slice = core::slice::from_raw_parts(env_ptr, env_len);

            // Skip if this is a runfiles var we're replacing
            let is_runfiles_var = env_slice.starts_with(b"RUNFILES_MANIFEST_FILE=")
                || env_slice.starts_with(b"RUNFILES_DIR=")
                || env_slice.starts_with(b"JAVA_RUNFILES=");

            if !is_runfiles_var {
                if data_pos + env_len + 1 <= MAX_ENV_SIZE && new_env_count < MAX_ENV_VARS {
                    MODIFIED_ENV_PTRS[new_env_count] = MODIFIED_ENV_DATA.as_ptr().add(data_pos);
                    new_env_count += 1;

                    MODIFIED_ENV_DATA[data_pos..data_pos + env_len].copy_from_slice(env_slice);
                    data_pos += env_len;
                    MODIFIED_ENV_DATA[data_pos] = 0;
                    data_pos += 1;
                } else {
                    env_dropped = true;
                }
            }

            i += 1;
        }

        // Check if any environment variables were dropped
        if env_dropped {
            print(b"ERROR: Failed to copy all environment variables\n");
            print(b"Environment buffer limit exceeded. Total size limit: ");
            print_number(MAX_ENV_SIZE);
            print(b" bytes, max variables: ");
            print_number(MAX_ENV_VARS);
            print(b"\n");
            print(b"Current usage: ");
            print_number(data_pos);
            print(b" bytes, ");
            print_number(new_env_count);
            print(b" variables\n");
            print(b"Consider reducing the number or size of environment variables.\n");
            exit(1);
        }

        // Null-terminate the pointer array
        MODIFIED_ENV_PTRS[new_env_count] = core::ptr::null();

        MODIFIED_ENV_PTRS.as_ptr()
    }
}

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".global _start",
    "_start:",
    "mov rdi, rsp",                 // Pass stack pointer as first argument
    "call _start_rust",             // Call the actual start function
);

#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(
    ".global _start",
    "_start:",
    "mov x0, sp",                   // Pass stack pointer as first argument
    "b _start_rust",                // Jump to the actual start function
);

#[no_mangle]
pub extern "C" fn _start_rust(initial_sp: *const usize) -> ! {
    unsafe {
        // Stack layout: [sp] = argc, [sp + 8] = argv[0], [sp + 16] = argv[1], ...
        let runtime_argc = *initial_sp;
        let runtime_argv = (initial_sp as usize + 8) as *const *const u8;

        // Check if ARGC is still a placeholder
        if is_template_placeholder(&ARGC_PLACEHOLDER) {
            print(b"ERROR: This is a template stub runner.\n");
            print(b"You must finalize it by replacing the placeholders before use.\n");
            print(b"The ARGC_PLACEHOLDER has not been replaced.\n");
            exit(1);
        }

        // Parse argc from placeholder
        let argc_str = &ARGC_PLACEHOLDER;
        let argc_len = str_len(argc_str);
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
        let flags_len = str_len(flags_str);
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

        // Parse export_runfiles_env flag
        let export_env_str = &EXPORT_RUNFILES_ENV;
        let export_env_len = str_len(export_env_str);
        let export_runfiles_env = if !is_template_placeholder(export_env_str) && export_env_len > 0 {
            export_env_str[0] == b'1'
        } else {
            true // Default to true if not set
        };

        // Check if any arguments need transformation
        // Create a mask for only the arguments we have (argc args)
        let argc_mask = if argc >= 32 {
            0xFFFFFFFF
        } else {
            (1u32 << argc) - 1
        };
        let needs_transform = (transform_flags & argc_mask) != 0;
        let needs_runfiles = needs_transform || export_runfiles_env;

        // Get executable path from runtime argv[0] (the stub's actual path) for runfiles fallback
        let executable_path = if runtime_argc > 0 {
            let argv0_ptr = *runtime_argv;
            let mut exe_len = 0;
            while *argv0_ptr.add(exe_len) != 0 && exe_len < MAX_PATH_LEN {
                exe_len += 1;
            }
            if exe_len > 0 {
                Some(core::slice::from_raw_parts(argv0_ptr, exe_len))
            } else {
                None
            }
        } else {
            None
        };

        // Initialize runfiles only if needed
        let runfiles = if needs_runfiles {
            if let Some(rf) = Runfiles::create(executable_path) {
                Some(rf)
            } else {
                print(b"ERROR: Failed to initialize runfiles\n");
                print(b"Set RUNFILES_DIR or RUNFILES_MANIFEST_FILE, or ensure <executable>.runfiles/ directory exists\n");
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
        let mut resolved_paths: [[u8; MAX_PATH_LEN]; 128] = [[0; MAX_PATH_LEN]; 128];
        let mut resolved_ptrs: [*const u8; 129] = [core::ptr::null(); 129];
        let mut total_argc = 0usize;

        // Resolve embedded arguments
        for i in 0..argc {
            let arg_data = arg_placeholders[i];
            let arg_len = str_len(arg_data);

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
                // Try to resolve through runfiles (which we know exists if we need transformation)
                if let Some(ref rf) = runfiles {
                    if let Some(resolved) = rf.rlocation(arg_slice) {
                        resolved_paths[i] = resolved;
                    } else {
                        // If not found in runfiles, use the path as-is
                        let copy_len = arg_len.min(MAX_PATH_LEN);
                        resolved_paths[i][..copy_len].copy_from_slice(&arg_slice[..copy_len]);
                    }
                } else {
                    // This should never happen - we checked needs_runfiles before
                    // But use path as-is for safety
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
            for i in 1..runtime_argc {
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

        // Build environment (with runfiles vars if export_runfiles_env is true)
        let envp = if export_runfiles_env {
            build_runfiles_environ(runfiles.as_ref())
        } else {
            get_environ()
        };

        // Execute the target program
        let ret = execve(executable, resolved_ptrs.as_ptr(), envp);

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
