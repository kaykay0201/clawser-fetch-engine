use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

const VERSION: &str = "0.1.0";
const GITHUB_REPO: &str = "kaykay0201/clawser-fetch-engine";

fn main() {
    let target = env::var("TARGET").unwrap_or_default();

    // 1. Check CLAWSER_LIB_DIR override (for local dev / CI).
    if let Ok(dir) = env::var("CLAWSER_LIB_DIR") {
        let lib_dir = PathBuf::from(&dir);
        if lib_dir.exists() {
            link(&lib_dir, &target);
            return;
        }
    }

    // 2. Check clawser-sys/native/ directory.
    let native_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("native");
    if has_native_lib(&native_dir, &target) {
        link(&native_dir, &target);
        return;
    }

    // 3. Auto-download from GitHub Releases into OUT_DIR.
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lib_dir = out_dir.join("clawser_native");
    fs::create_dir_all(&lib_dir).expect("Failed to create native lib directory");

    if !has_native_lib(&lib_dir, &target) {
        download(&lib_dir, &target);
    }

    link(&lib_dir, &target);
}

fn has_native_lib(dir: &PathBuf, target: &str) -> bool {
    if target.contains("windows") {
        dir.join("clawser_fetch.dll").exists() && dir.join("clawser_fetch.dll.lib").exists()
    } else if target.contains("apple") {
        dir.join("libclawser_fetch.dylib").exists()
    } else {
        dir.join("libclawser_fetch.so").exists()
    }
}

fn download(lib_dir: &PathBuf, target: &str) {
    let (dll_name, lib_name) = if target.contains("windows") {
        ("clawser_fetch.dll", Some("clawser_fetch.dll.lib"))
    } else if target.contains("apple") {
        ("libclawser_fetch.dylib", None)
    } else {
        ("libclawser_fetch.so", None)
    };

    let platform = if target.contains("windows") {
        "windows-x64"
    } else if target.contains("apple") && target.contains("aarch64") {
        "macos-arm64"
    } else if target.contains("apple") {
        "macos-x64"
    } else {
        "linux-x64"
    };

    // Download DLL.
    let dll_url = format!(
        "https://github.com/{}/releases/download/v{}/{}-{}",
        GITHUB_REPO, VERSION, platform, dll_name
    );
    download_file(&dll_url, &lib_dir.join(dll_name));

    // Download import lib (Windows only).
    if let Some(lib) = lib_name {
        let lib_url = format!(
            "https://github.com/{}/releases/download/v{}/{}-{}",
            GITHUB_REPO, VERSION, platform, lib
        );
        download_file(&lib_url, &lib_dir.join(lib));
    }
}

fn download_file(url: &str, dest: &PathBuf) {
    eprintln!("clawser-sys: downloading {}", url);
    let resp = ureq::get(url).call().unwrap_or_else(|e| {
        panic!(
            "clawser-sys: failed to download native library from {}: {}\n\
             You can manually download it and set CLAWSER_LIB_DIR.",
            url, e
        )
    });
    let mut reader = resp.into_body().into_reader();
    let mut file = fs::File::create(dest)
        .unwrap_or_else(|e| panic!("clawser-sys: failed to create {:?}: {}", dest, e));
    io::copy(&mut reader, &mut file)
        .unwrap_or_else(|e| panic!("clawser-sys: failed to write {:?}: {}", dest, e));
    eprintln!("clawser-sys: saved to {:?}", dest);
}

fn link(lib_dir: &PathBuf, target: &str) {
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=clawser_fetch");

    if !target.contains("windows") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
    }
}
