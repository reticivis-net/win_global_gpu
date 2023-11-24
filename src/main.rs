mod charging_events;
mod exe_scan;
mod notification;
mod registry;
mod sector_reader;
mod winapp_scan;

use anyhow::Result;
use std::fs::File;

fn unplug() {
    notification::toast("Unplugged");
}
fn plug() {
    notification::toast("Plugged in");
}

fn main() -> Result<()> {
    println!("Hello, world!");
    let f = File::open("C:\\$MFT")?;
    let files = exe_scan::get_files()?;
    println!("{}", files.len());
    unsafe { charging_events::register_events(unplug, plug)? }
    Ok(())
}
