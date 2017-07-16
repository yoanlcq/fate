fn main() {
    #[cfg(feature="desktop")]
    println!("On desktop!");
    #[cfg(feature="mobile")]
    println!("On mobile!");
    #[cfg(feature="web")]
    println!("On web!");
    #[cfg(feature="nothreads")]
    println!("No threads! (we're likely on Emscripten)");
    #[cfg(feature="multiple_windows")]
    println!("Can create multiple windows! (we're likely on desktop)");
    #[cfg(feature="fixed_size_window")]
    println!("Unable to change window size! (we're likely on mobile or some console)");
}
