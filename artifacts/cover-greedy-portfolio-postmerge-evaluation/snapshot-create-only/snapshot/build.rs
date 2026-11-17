use std::{
    env, fs,
    path::{Path, PathBuf},
};
fn rust_files(path: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            rust_files(&p, files);
        } else if p.extension().is_some_and(|e| e == "rs") {
            files.push(p);
        }
    }
}
fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut files = vec![
        root.join("Cargo.toml"),
        root.join("Cargo.lock"),
        root.join("build.rs"),
    ];
    rust_files(&root.join("src"), &mut files);
    files.sort_by_key(|p| {
        p.strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/")
    });
    let mut hash = 0xcbf29ce484222325u64;
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        let name = path
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        for byte in name
            .bytes()
            .chain(std::iter::once(0))
            .chain(fs::read(path).unwrap())
            .chain(std::iter::once(0))
        {
            hash = (hash ^ byte as u64).wrapping_mul(0x100000001b3);
        }
    }
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rustc-env=MIP_SOURCE_FINGERPRINT={hash:016x}");
}
