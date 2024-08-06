use cmake::Config;
use std::env;
use std::path::{Path, PathBuf};

macro_rules! debug_log {
    ($($arg:tt)*) => {
        if std::env::var("BUILD_DEBUG").is_ok() {
            println!("cargo:warning=[DEBUG] {}", format!($($arg)*));
        }
    };
}

fn copy_folder(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).expect("Failed to create dst directory");
    if cfg!(unix) {
        std::process::Command::new("cp")
            .arg("-rf")
            .arg(src)
            .arg(dst.parent().unwrap())
            .status()
            .expect("Failed to execute cp command");
    }

    if cfg!(windows) {
        std::process::Command::new("robocopy.exe")
            .arg("/e")
            .arg(src)
            .arg(dst)
            .status()
            .expect("Failed to execute robocopy command");
    }
}

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");
    let knf_src = Path::new(&manifest_dir).join("knf");
    let knf_dst = out_dir.join("knf");
    let knfc_src = Path::new(&manifest_dir).join("knfc");
    let knfc_dst = out_dir.join("knfc");

    let profile = if cfg!(debug_assertions) {
        "Debug"
    } else {
        "Release"
    };

    debug_log!("TARGET: {}", target);
    debug_log!("CARGO_MANIFEST_DIR: {}", manifest_dir);
    debug_log!("OUT_DIR: {}", out_dir.display());

    if !knf_dst.exists() {
        debug_log!("Copy {} to {}", knf_src.display(), knf_dst.display());
        copy_folder(&knf_src, &knf_dst);
    }

    if !knfc_dst.exists() {
        debug_log!("Copy {} to {}", knfc_src.display(), knfc_dst.display());
        copy_folder(&knfc_src, &knfc_dst);
    }

    // Bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.hpp")
        .clang_arg(format!("-I{}", knfc_dst.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Failed to generate bindings");

    // Write the generated bindings to an output file
    let bindings_path = out_dir.join("bindings.rs");
    bindings
        .write_to_file(bindings_path)
        .expect("Failed to write bindings");

    println!("cargo:rerun-if-changed=./knf");
    println!("cargo:rerun-if-changed=./knfc");
    println!("cargo:rerun-if-changed=wrapper.h");
    

    debug_log!("Bindings Created");

    let mut config = Config::new(&knfc_dst);

    config
        .profile(profile)
        .very_verbose(std::env::var("CMAKE_VERBOSE").is_ok()) // Not verbose by default
        .always_configure(false)
        .build();

    println!("cargo:rustc-link-search={}", out_dir.join("lib").display());

    // Link
    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=static={}", "knfc");
    println!("cargo:rustc-link-lib=static={}", "kaldi-native-fbank-core");
    

    
}