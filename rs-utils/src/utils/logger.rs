use dotenvy::dotenv_override;
use std::error::Error;

extern crate dotenvy;
extern crate log;

pub fn initialize_logger() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    log_panics::init();
    dotenv_override()?;

    Ok(())
}
