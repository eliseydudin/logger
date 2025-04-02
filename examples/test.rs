use logger::Logger;
use std::{error::Error, io::stdout, thread};

fn main() -> Result<(), Box<dyn Error>> {
    Logger::init(stdout());
    log::set_max_level(log::LevelFilter::Trace);
    log::info!("info");
    log::error!("error");
    log::debug!("silly");

    thread::Builder::new()
        .name("other".to_owned())
        .spawn(|| log::info!("hello from the other thread!"))?;

    let handle = thread::spawn(|| log::info!("mrrp"));
    handle.join().map_err(|err| format!("{err:?}"))?;

    Ok(())
}
