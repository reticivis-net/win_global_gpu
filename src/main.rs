// #![windows_subsystem = "windows"] // this prevents the gui?

mod charging_events;
mod exe_scan_2;
mod full_win_scan;
mod hstring_utils;
mod notification;
mod registry;
mod winapp_scan;

use anyhow::{anyhow, Result};
use std::sync::OnceLock;
use windows::core::HSTRING;

fn unplug() {
    notification::toast("🔌 Unplugged");
}
fn plug() {
    notification::toast("🔌 Plugged in");
}

static PROGRAMS: OnceLock<Vec<HSTRING>> = OnceLock::new();

fn main() -> Result<()> {
    // TODO: CreateMutexW to detect multiple instances
    println!("Hello, world!");
    if PROGRAMS.set(full_win_scan::get_all_programs()?).is_err() {
        return Err(anyhow!("Failed to store program list."));
    }
    // unsafe {
    //     let files = exe_scan_2::get_all_files();
    // }
    // let files = exe_scan::get_files()?;
    // println!("{}", files.len());
    // notification::register()?;
    // unsafe { charging_events::register_events(unplug, plug)? }
    Ok(())
}
