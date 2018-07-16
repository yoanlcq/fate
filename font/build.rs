extern crate hyper;

use std::path::PathBuf;
use std::env;
use std::fs;

fn fetch_freetype_msvc(arch: &str) {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dir = PathBuf::from(manifest_dir).join("target").join("downloads").join(arch);
    fs::DirBuilder::new().recursive(true).create(&dir).unwrap();

    let url_base = "https://github.com/PistonDevelopers/binaries/raw/master";
    let file_names = ["freetype.dll", "freetype.lib"];

    let client = hyper::Client::new();
    for file_name in file_names {
        let data = client.get(format!("{}/{}/{}", url_base, arch, file_name)).send().unwrap();
        fs::write(format!("{}/{}", dir, file_name), &data).unwrap();
    }
    Ok(())
}

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.contains("pc-windows-msvc") {
        let arch = if target.contains("i686") {
            "i686"
        } else if target.contains("x86_64") {
            "x86_64"
        } else { 
            panic!("No arch")
        };
        fetch_freetype_msvc(arch);
    }
}