use std::{error::Error, fs::File};

fn main() -> Result<(), Box<dyn Error>> {
    logger::init_stdout();
    log::set_max_level(log::LevelFilter::Trace);

    log::info!("info");
    log::error!("error");
    log::debug!("debug");
    log::trace!("trace");
    log::warn!("warn");

    let new_file = File::create("test.txt")?;
    logger::swap(new_file);
    log::info!("hello kro...");

    Ok(())
}
