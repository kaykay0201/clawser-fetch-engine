use std::env;
use std::path::PathBuf;

fn main() {
    // Allow override via environment variable for local dev / CI.
    let lib_dir = if let Ok(dir) = env::var("CLAWSER_LIB_DIR") {
        PathBuf::from(dir)
    } else {
        // Default: look next to the crate root.
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("native")
    };

    if !lib_dir.exists() {
        panic!(
            "clawser-sys: native library directory not found at {:?}. \
             Set CLAWSER_LIB_DIR to the directory containing clawser_fetch.dll / \
             libclawser_fetch.so, or place them in clawser-sys/native/.",
            lib_dir
        );
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=clawser_fetch");

    // On non-Windows, set rpath for development.
    let target = env::var("TARGET").unwrap_or_default();
    if !target.contains("windows") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
    }
}
