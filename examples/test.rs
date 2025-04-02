use std::{error::Error, fs::File, io::stdout};

fn main() -> Result<(), Box<dyn Error>> {
    let out = File::create("test.txt")?;
    logger::init(out);

    log::info!("this writes to file!!");

    let out = stdout();
    logger::Logger::replace_logger(out);

    log::info!("info");
    log::error!("error");
    log::debug!("debug");
    log::trace!("trace");
    log::warn!("warn");

    Ok(())
}
