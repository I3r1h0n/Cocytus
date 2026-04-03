// FFI bindings to wimlib
use std::{ffi::c_void, path::{Path, PathBuf}};

use crate::{debug, error::AppError};
use crate::utils::{to_wide, wide_to_string};

unsafe extern "C" {
    fn wimlib_global_init(init_flags: i32) -> i32;
    fn wimlib_global_cleanup();
    fn wimlib_open_wim(wim_file: *const u16, open_flags: i32, wim_ret: *mut *mut c_void) -> i32;
    fn wimlib_extract_paths(
        wim: *mut c_void,
        image: i32,
        target: *const u16,
        paths: *const *const u16,
        num_paths: usize,
        extract_flags: i32,
    ) -> i32;
    fn wimlib_free(wim: *mut c_void);
    fn wimlib_get_error_string(code: i32) -> *const u16;
    fn wimlib_iterate_dir_tree(
        wim: *mut c_void,
        image: i32,
        path: *const u16,
        flags: i32,
        cb: Option<WimlibIterateCallback>,
        user_ctx: *mut c_void,
    ) -> i32;
    fn wimlib_get_wim_info(wim: *mut c_void, info: *mut WimlibWimInfo) -> i32;
    fn wimlib_get_image_name(wim: *mut c_void, image: i32) -> *const u16;
}

/// Callback type for `wimlib_iterate_dir_tree`
type WimlibIterateCallback = unsafe extern "C" fn(
    dentry: *const WimlibDirEntry,
    user_ctx: *mut c_void,
) -> i32;

/// Partial representation of `wimlib_dir_entry`
#[repr(C)]
struct WimlibDirEntry {
    filename: *const u16,
    dos_name: *const u16,
    full_path: *const u16,
    depth: usize,
    security_descriptor: *const u8,
    security_descriptor_size: usize,
    attributes: u32,
}

const WIMLIB_EXTRACT_FLAG_NO_ACLS: i32 = 0x00000040;
const WIMLIB_EXTRACT_FLAG_NO_ATTRIBUTES: i32 = 0x00100000;
const WIMLIB_EXTRACT_FLAG_NO_PRESERVE_DIR_STRUCTURE: i32 = 0x00200000;

const WIMLIB_ITERATE_DIR_TREE_FLAG_RECURSIVE: i32 = 0x00000001;
const WIMLIB_ITERATE_DIR_TREE_FLAG_CHILDREN: i32 = 0x00000002;

const WIMLIB_FILE_ATTRIBUTE_DIRECTORY: u32 = 0x00000010;

fn wimlib_error(code: i32) -> AppError {
    let message = unsafe { wide_to_string(wimlib_get_error_string(code)) };
    AppError::Wim { code, message }
}

/// RAII wrapper that calls `wimlib_free` + `wimlib_global_cleanup` on drop
struct WimHandle(*mut c_void);

impl WimHandle {
    /// Initialise wimlib and open a WIM file.
    fn open(wim_path: &Path) -> Result<Self, AppError> {
        unsafe {
            let ret = wimlib_global_init(0);
            if ret != 0 {
                return Err(wimlib_error(ret));
            }

            let wim_path_wide = to_wide(&wim_path.to_string_lossy());
            let mut wim: *mut c_void = std::ptr::null_mut();
            let ret = wimlib_open_wim(wim_path_wide.as_ptr(), 0, &mut wim);
            if ret != 0 {
                wimlib_global_cleanup();
                return Err(wimlib_error(ret));
            }

            Ok(WimHandle(wim))
        }
    }

    fn ptr(&self) -> *mut c_void {
        self.0
    }
}

impl Drop for WimHandle {
    fn drop(&mut self) {
        unsafe {
            wimlib_free(self.0);
            wimlib_global_cleanup();
        }
    }
}

/// Find install.wim or boot.wim on the mounted drive
pub fn find_wim(drive: &str) -> Result<PathBuf, AppError> {
    let candidates = [
        format!("{drive}\\sources\\install.wim"),
        format!("{drive}\\sources\\boot.wim"),
    ];

    for candidate in &candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            debug!("Found WIM: {}", path.display());
            return Ok(path);
        }
    }

    Err(AppError::WimNotFound)
}

/// Extract a file from a WIM image into `output_dir`
pub fn extract_file(
    wim_path: &Path,
    output_dir: &Path,
    image_index: i32,
    inner_path: &str,
) -> Result<(), AppError> {
    let wim = WimHandle::open(wim_path)?;

    let extract_path = to_wide(inner_path);
    let paths_array: [*const u16; 1] = [extract_path.as_ptr()];
    let target = to_wide(&output_dir.to_string_lossy());

    let extract_flags = WIMLIB_EXTRACT_FLAG_NO_ACLS
        | WIMLIB_EXTRACT_FLAG_NO_ATTRIBUTES
        | WIMLIB_EXTRACT_FLAG_NO_PRESERVE_DIR_STRUCTURE;

    let ret = unsafe {
        wimlib_extract_paths(
            wim.ptr(),
            image_index,
            target.as_ptr(),
            paths_array.as_ptr(),
            1,
            extract_flags,
        )
    };

    if ret != 0 {
        return Err(wimlib_error(ret));
    }

    // Derive the filename from the inner path for the log message
    let filename = inner_path.rsplit(['\\', '/']).next().unwrap_or(inner_path);
    debug!("Extracted {filename} to: {}", output_dir.join(filename).display());

    Ok(())
}

/// Callback that collects filenames into a `Vec<String>`
unsafe extern "C" fn list_cb(dentry: *const WimlibDirEntry, user_ctx: *mut c_void) -> i32 {
    unsafe {
        let entries = &mut *(user_ctx as *mut Vec<String>);
        let name = wide_to_string((*dentry).filename);
        entries.push(name);
    }
    0
}

/// List immediate children of `dir_path` inside a WIM image
pub fn list_dir(
    wim_path: &Path,
    image_index: i32,
    dir_path: &str,
) -> Result<Vec<String>, AppError> {
    let wim = WimHandle::open(wim_path)?;

    let dir_wide = to_wide(dir_path);
    let mut entries: Vec<String> = Vec::new();

    let ret = unsafe {
        wimlib_iterate_dir_tree(
            wim.ptr(),
            image_index,
            dir_wide.as_ptr(),
            WIMLIB_ITERATE_DIR_TREE_FLAG_CHILDREN,
            Some(list_cb),
            &mut entries as *mut Vec<String> as *mut c_void,
        )
    };

    if ret != 0 {
        return Err(wimlib_error(ret));
    }

    Ok(entries)
}

/// Possible PE file extension list
const PE_EXTENSIONS: &[&str] = &[
    ".exe", 
    ".dll", 
    ".sys", 
    ".drv", 
    ".efi", 
    ".ocx", 
    ".scr", 
    ".cpl", 
    ".ax", 
    ".acm", 
    ".tsp",
];

/// Check if file is PE, based on file extension
fn is_pe_extension(path: &str) -> bool {
    let path_lower = path.to_ascii_lowercase();
    PE_EXTENSIONS.iter().any(|ext| path_lower.ends_with(ext))
}

/// Callback that collects full paths of PE files only
unsafe extern "C" fn list_recursive_cb(dentry: *const WimlibDirEntry, user_ctx: *mut c_void) -> i32 {
    unsafe {
        let entries = &mut *(user_ctx as *mut Vec<String>);
        let attrs = (*dentry).attributes;
        if attrs & WIMLIB_FILE_ATTRIBUTE_DIRECTORY == 0 {
            let path = wide_to_string((*dentry).full_path);
            if is_pe_extension(&path) {
                entries.push(path);
            }
        }
    }
    0
}

/// List all PE files (recursively) under `dir_path` inside a WIM image
pub fn list_dir_recursive(
    wim_path: &Path,
    image_index: i32,
    dir_path: &str,
) -> Result<Vec<String>, AppError> {
    let wim = WimHandle::open(wim_path)?;

    let dir_wide = to_wide(dir_path);
    let mut entries: Vec<String> = Vec::new();

    let flags = WIMLIB_ITERATE_DIR_TREE_FLAG_CHILDREN | WIMLIB_ITERATE_DIR_TREE_FLAG_RECURSIVE;

    let ret = unsafe {
        wimlib_iterate_dir_tree(
            wim.ptr(),
            image_index,
            dir_wide.as_ptr(),
            flags,
            Some(list_recursive_cb),
            &mut entries as *mut Vec<String> as *mut c_void,
        )
    };

    if ret != 0 {
        return Err(wimlib_error(ret));
    }

    Ok(entries)
}

/// Partial repr of wimlib_wim_info (only need image_count)
#[repr(C)]
struct WimlibWimInfo {
    guid: [u8; 16],
    image_count: u32,
    // remaining fields omitted
    _pad: [u8; 240],
}

/// An image entry inside a WIM file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WimImage {
    /// 1-based index
    pub index: i32,
    /// Human-readable name
    pub name: String,
}

impl std::fmt::Display for WimImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.is_empty() {
            write!(f, "Image {}", self.index)
        } else if self.name.len() > 40 {
            write!(f, "{}...", &self.name[..37])
        } else {
            write!(f, "{}", self.name)
        }
    }
}

/// List all images inside a WIM file
pub fn list_images(wim_path: &Path) -> Result<Vec<WimImage>, AppError> {
    let wim = WimHandle::open(wim_path)?;

    let mut info: WimlibWimInfo = unsafe { std::mem::zeroed() };
    let ret = unsafe { wimlib_get_wim_info(wim.ptr(), &mut info) };
    if ret != 0 {
        return Err(wimlib_error(ret));
    }

    let mut images = Vec::new();
    for i in 1..=info.image_count as i32 {
        let name_ptr = unsafe { wimlib_get_image_name(wim.ptr(), i) };
        let name = if name_ptr.is_null() {
            String::new()
        } else {
            wide_to_string(name_ptr)
        };
        images.push(WimImage { index: i, name });
    }

    Ok(images)
}
