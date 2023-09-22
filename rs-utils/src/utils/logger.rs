use dotenvy::dotenv;
use std::{env, error::Error};

extern crate dotenvy;

pub fn initialize_logger() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_BACKTRACE", "1");

    // env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    env_logger::init();
    log_panics::init();
    dotenv()?;

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
