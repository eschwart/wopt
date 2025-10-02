use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs"); // rerun if build.rs itself changes
    println!("cargo:rerun-if-changed="); // empty means always rerun

    let path = Path::new("target/wopt/counter");

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    // Reset counter to "0" every rebuild
    fs::write(path, b"0").unwrap();
}
