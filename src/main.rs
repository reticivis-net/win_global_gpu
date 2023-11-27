// #![windows_subsystem = "windows"] // this prevents the gui?

mod charging_events;
mod exe_scan_2;
mod notification;
mod registry;
mod sector_reader;
mod winapp_scan;

use anyhow::Result;

fn unplug() {
    notification::toast("Unplugged");
}
fn plug() {
    notification::toast("Plugged in");
}

fn main() -> Result<()> {
    // TODO: CreateMutexW to detect multiple instances
    println!("Hello, world!");
    unsafe {
        let files = exe_scan_2::get_all_files();
    }
    // let files = exe_scan::get_files()?;
    // println!("{}", files.len());
    // unsafe { charging_events::register_events(unplug, plug)? }
    Ok(())
}
