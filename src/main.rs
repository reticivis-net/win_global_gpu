mod charging_events;
mod exe_scan;
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
    println!("Hello, world!");
    println!("\\\\.\\C:");
    dbg!(exe_scan::scan_drive(&"\\\\.\\C:".to_string())?);
    unsafe { charging_events::register_events(unplug, plug)? }
    Ok(())
}
