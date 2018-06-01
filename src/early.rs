use std::env;
use std::panic;
use sdl2::messagebox;
use log::LevelFilter;
use env_logger;
use backtrace;

pub fn setup_panic_hook() {
    panic::set_hook(Box::new(|info| {

        let mut msg = match info.location() {
            Some(location) => format!("Panic occurred in file '{}' at line {}:\n", location.file(), location.line()),
            None => format!("Panic occurred in unknown location:\n"),
        };

        if let Some(payload) = info.payload().downcast_ref::<&str>() {
            msg += payload;
        } else {
            msg += "<unknown reason>";
        }

        error!("{}", &msg);

        info!("Backtrace:");
        backtrace::trace(|frame| {
            let ip = frame.ip();
            let _symbol_address = frame.symbol_address();

            backtrace::resolve(ip, |symbol| {
                let what = || "??".to_owned();
                let filename = if let Some(filename) = symbol.filename() { format!("{}", filename.display()) } else { what() };
                let lineno = if let Some(lineno) = symbol.lineno() { format!("{}", lineno) } else { what() };
                let addr = if let Some(addr) = symbol.addr() { format!("0x{:8x}", addr as usize) } else { what() };
                let name = if let Some(name) = symbol.name() { format!("{}", name) } else { what() };
                // ^ NOTE: Do use the Display implementation for name. It demangles the symbol.
                info!("{}:{}: ({}) {}", &filename, &lineno, &addr, name);
            });

            true // keep going to the next frame
        });

        let flags = messagebox::MESSAGEBOX_ERROR;
        let result = messagebox::show_simple_message_box(
            flags, "Fatal error", &msg, None
        );
        if let Err(e) = result {
            use sdl2::messagebox::ShowMessageError::*;
            let msg = "Couldn't display message box: ".to_owned() + &match e {
                InvalidTitle(nul_error) => format!("Invalid title: {:?}", nul_error),
                InvalidMessage(nul_error) => format!("Invalid message: {:?}", nul_error),
                InvalidButton(nul_error, i) => format!("Invalid button {}: {:?}", i, nul_error),
                SdlError(msg) => format!("SDL2 error: {}", &msg),
            };
            error!("{}", &msg);
        }
    }));
}

pub fn setup_env() {
    //env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "full");
}

pub fn setup_log() {
    use ::std::io::Write;

    let mut builder = env_logger::Builder::new();
    builder.format(|buf, record| {
        let s = format!("{}", record.level());
        let s = s.chars().next().unwrap();
        writeln!(buf, "[{}] {}", s, record.args())
    }).filter(None, LevelFilter::Debug);

    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse(&rust_log);
    }
    builder.init();
}

