extern crate download_sdl2;

fn main() {
    download_sdl2::download().expect("Failed to download SDL2 development libraries");
}