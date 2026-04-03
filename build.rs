use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let wimlib_dir = Path::new(&manifest_dir).join("wimlib");

    println!("cargo:rustc-link-search=native={}", wimlib_dir.display());
    println!("cargo:rustc-link-lib=dylib=libwim");

    let target_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&target_dir);
    let profile_dir = out_path
        .ancestors()
        .nth(3)
        .expect("Failed to resolve target profile directory");

    let dll_src = wimlib_dir.join("libwim-15.dll");
    let dll_dst = profile_dir.join("libwim-15.dll");

    if dll_src.exists() && (!dll_dst.exists() || file_modified(&dll_src) > file_modified(&dll_dst))
    {
        fs::copy(&dll_src, &dll_dst).expect("Failed to copy libwim-15.dll to target directory");
        println!("cargo:warning=Copied libwim-15.dll to {}", dll_dst.display());
    }

    println!("cargo:rerun-if-changed=wimlib/libwim-15.dll");
    println!("cargo:rerun-if-changed=wimlib/libwim.lib");
}

fn file_modified(path: &Path) -> std::time::SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
}
