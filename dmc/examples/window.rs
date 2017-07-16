extern crate dmc;
use dmc::display::Display;

use std::time::Duration;
use std::thread::sleep;

fn main() {
    let display = Display::open().expect("Could not open display!");
    let window = display.create_window(&Default::default()).expect("Couldn't create window!");
    window.set_title("Three");
    window.show();
    sleep(Duration::from_secs(1));
    window.set_title("Two");
    sleep(Duration::from_secs(1));
    window.set_title("One");
    sleep(Duration::from_secs(1));
}
