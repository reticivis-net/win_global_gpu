// #![windows_subsystem = "windows"] // this prevents the gui?

mod charging_events;
mod elevate;
mod exe_scan_2;
mod full_win_scan;
mod hide_console;
mod hstring_utils;
mod notification;
mod prevent_duplicate;
mod registry;
mod winapp_scan;

use anyhow::{anyhow, Result};
use std::sync::OnceLock;
use windows::core::HSTRING;

fn unplug() {
    notification::toast("🔌 Unplugged").unwrap();
    let res =
        unsafe { registry::write_reg(PROGRAMS.get().unwrap(), registry::GpuMode::Integrated) };
    match res {
        Ok(_) => {
            println!("Wrote to registry!")
        }
        Err(e) => {
            notification::toast("Error writing to registry.").unwrap();
            dbg!(e);
        }
    }
}
fn plug() {
    notification::toast("🔌 Plugged in").unwrap();
    let res = unsafe { registry::write_reg(PROGRAMS.get().unwrap(), registry::GpuMode::Dedicated) };
    match res {
        Ok(_) => {
            println!("Wrote to registry!")
        }
        Err(e) => {
            notification::toast("Error writing to registry.").unwrap();
            dbg!(e);
        }
    }
}

static PROGRAMS: OnceLock<Vec<HSTRING>> = OnceLock::new();

fn main() -> Result<()> {
    // TODO: CreateMutexW to detect multiple instances
    println!("Hello, world!");
    elevate::elevate_if_needed()?;
    unsafe { prevent_duplicate::kill_older_process()? }
    // unsafe { registry::write_reg(&vec!(), registry::GpuMode::None)?;}
    // it doesnt return an error type it returns the stuff already in the var so ? doesn't work
    if PROGRAMS.set(full_win_scan::get_all_programs()?).is_err() {
        return Err(anyhow!("Failed to store program list."));
    }
    unsafe { hide_console::hide_console()? }
    notification::register()?;
    unsafe { charging_events::register_events(unplug, plug)? }
    Ok(())
}
