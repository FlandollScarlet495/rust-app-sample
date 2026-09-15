use std::env;
use std::path::PathBuf;

pub fn run() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let libraries_dir = manifest_dir.join("../../dist/libraries");

    println!("cargo:rustc-link-search=native={}", libraries_dir.display());
    println!(
        "cargo:rustc-link-rerun-if-changed={}",
        libraries_dir.display()
    );
}
