use std::fs;
use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).ancestors().nth(3).unwrap(); // gets target/debug or release

    fs::copy("Config.toml", target_dir.join("Config.toml"))
        .expect("Failed to copy Config.toml to target directory");
}
