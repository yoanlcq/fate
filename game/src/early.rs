use std::env;
use std::fmt;
use std::panic;
use log::{Level, LevelFilter};
use pretty_env_logger;
use backtrace;

pub fn setup_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let mut msg = match info.location() {
            Some(location) => format!("Panic occurred in file '{}' at line {}:\n", location.file(), location.line()),
            None => format!("Panic occurred in unknown location:\n"),
        };

        if let Some(payload) = info.payload().downcast_ref::<&str>() {
            msg += payload;
        } else if let Some(payload) = info.payload().downcast_ref::<&String>() {
            msg += payload.as_str();
        } else if let Some(args) = info.message() {
            msg += &fmt::format(*args);
        } else {
            msg += &format!("<unknown reason>. Debug: {:?}", info);
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
    }));
}

pub fn setup_env() {
    //env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "full");
}

pub fn setup_log() {
    use ::std::io::Write;

    let mut builder = pretty_env_logger::formatted_builder().unwrap();
    builder.format(|buf, record| {
        writeln!(buf, "[{}] [{}] {}", log_level_letter(record.level()), record.target(), record.args())
    }).filter(None, LevelFilter::Debug);

    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse(&rust_log);
    }
    builder.init();
}

fn log_level_letter(level: Level) -> &'static str {
    match level {
        Level::Error => "E",
        Level::Warn  => "W",
        Level::Info  => "I",
        Level::Debug => "D",
        Level::Trace => "T",
    }
}
