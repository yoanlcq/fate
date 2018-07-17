extern crate hyper;

use hyper::{Uri, Response, rt::{Future, Stream}};
use std::path::PathBuf;
use std::env;
use std::fs;

fn fetch_freetype_msvc(arch: &str) {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dir = PathBuf::from(manifest_dir).join("target").join("downloads").join(arch);
    fs::DirBuilder::new().recursive(true).create(&dir).unwrap();

    let url_base = "http://github.com/PistonDevelopers/binaries/raw/master";
    let file_names = ["freetype.dll", "freetype.lib"];

    for file_name in &file_names {
        let dir = dir.clone();
        let file_name = file_name.to_string();
        hyper::rt::run({
            let uri: Uri = format!("{}/{}/{}", url_base, arch, file_name).parse().unwrap();
            hyper::Client::new().get(uri).map(Response::into_body).flatten_stream().concat2().map(move |data| {
                fs::write(dir.join(file_name), &data).unwrap();
            }).map_err(|_| ())
        });
    }
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