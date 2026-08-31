use std::env;
use std::path::PathBuf;
use std::process::Command;

fn split_env_paths(name: &str) -> Vec<PathBuf> {
    env::var_os(name)
        .map(|value| env::split_paths(&value).collect())
        .unwrap_or_default()
}

fn main() {
    println!("cargo:rerun-if-changed=native/daemon_embedded.cpp");
    println!("cargo:rerun-if-env-changed=RTSYN_NATIVE_INCLUDE_DIRS");
    println!("cargo:rerun-if-env-changed=RTSYN_NATIVE_LIB_DIRS");
    println!("cargo:rerun-if-env-changed=RTSYN_NATIVE_LIBS");
    println!("cargo:rerun-if-env-changed=RTSYN_NATIVE_DYNAMIC_LIBS");
    println!("cargo:rerun-if-env-changed=RTSYN_STATIC_CXX_RUNTIME");
    println!("cargo:rerun-if-env-changed=RTSYN_THREAD_CORE");

    let include_dirs = split_env_paths("RTSYN_NATIVE_INCLUDE_DIRS");
    if include_dirs.is_empty() {
        panic!("RTSYN_NATIVE_INCLUDE_DIRS is unset; build rtsyn through xmake");
    }

    let mut build = cc::Build::new();
    build.cpp(true).std("c++23").file("native/daemon_embedded.cpp");
    for include_dir in include_dirs {
        build.include(include_dir);
    }
    let cxx_compiler = build.get_compiler().path().to_owned();
    build.compile("rtsyn_embedded_daemon");

    let lib_dirs = split_env_paths("RTSYN_NATIVE_LIB_DIRS");
    for lib_dir in &lib_dirs {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
    }

    let libs = env::var("RTSYN_NATIVE_LIBS").unwrap_or_default();
    for lib in libs.split(',').filter(|lib| !lib.is_empty()) {
        if let Some(archive) = find_static_archive(&lib_dirs, lib) {
            println!("cargo:rerun-if-changed={}", archive.display());
            println!("cargo:rustc-link-lib=static:+whole-archive={lib}");
        } else {
            println!("cargo:rustc-link-lib=static={lib}");
        }
    }

    let dynamic_libs = env::var("RTSYN_NATIVE_DYNAMIC_LIBS").unwrap_or_default();
    for lib in dynamic_libs.split(',').filter(|lib| !lib.is_empty()) {
        println!("cargo:rustc-link-lib=dylib={lib}");
    }

    if env::var_os("RTSYN_STATIC_CXX_RUNTIME").is_some() {
        let archive = compiler_file(&cxx_compiler, "libstdc++.a");
        let archive_dir = archive
            .parent()
            .expect("C++ runtime archive has no parent directory");
        println!("cargo:rustc-link-search=native={}", archive_dir.display());
        println!("cargo:rustc-link-lib=static=stdc++");
    } else {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
    println!("cargo:rustc-link-lib=dylib=pthread");
    println!("cargo:rustc-link-lib=dylib=dl");
    println!("cargo:rustc-link-lib=dylib=rt");
}

fn compiler_file(compiler: &std::path::Path, file_name: &str) -> PathBuf {
    let output = Command::new(compiler)
        .arg(format!("-print-file-name={file_name}"))
        .output()
        .unwrap_or_else(|error| panic!("failed to query {}: {error}", compiler.display()));
    assert!(
        output.status.success(),
        "{} could not locate {file_name}",
        compiler.display()
    );

    let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    assert!(path.is_file(), "C++ runtime archive not found: {}", path.display());
    path
}

fn find_static_archive(lib_dirs: &[PathBuf], lib: &str) -> Option<PathBuf> {
    let file_name = format!("lib{lib}.a");
    lib_dirs
        .iter()
        .map(|dir| dir.join(&file_name))
        .find(|candidate| candidate.is_file())
}
