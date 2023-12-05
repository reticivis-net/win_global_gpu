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
use std::env;
use std::sync::OnceLock;
use windows::core::HSTRING;

fn unplug() {
    let res =
        unsafe { registry::write_reg(PROGRAMS.get().unwrap(), registry::GpuMode::Integrated) };
    match res {
        Ok(_) => {
            println!("Wrote to registry!");
            notification::toast("Set global GPU to integrated GPU.").unwrap();
        }
        Err(e) => {
            notification::toast("Error writing to registry.").unwrap();
            dbg!(e);
        }
    }
}
fn plug() {
    let res = unsafe { registry::write_reg(PROGRAMS.get().unwrap(), registry::GpuMode::Dedicated) };
    match res {
        Ok(_) => {
            println!("Wrote to registry!");
            notification::toast("Set global GPU to dedicated GPU.").unwrap();
        }
        Err(e) => {
            notification::toast("Error writing to registry.").unwrap();
            dbg!(e);
        }
    }
}

static PROGRAMS: OnceLock<Vec<HSTRING>> = OnceLock::new();

fn kill_duplicate() -> Result<bool> {
    unsafe { prevent_duplicate::kill_older_process() }
}
fn set_programs() -> Result<()> {
    if PROGRAMS.set(full_win_scan::get_all_programs()?).is_err() {
        return Err(anyhow!("Failed to store program list."));
    }
    Ok(())
}
fn core() -> Result<()> {
    kill_duplicate()?;
    set_programs()?;
    notification::register()?;
    #[cfg(not(debug_assertions))]
    unsafe {
        hide_console::hide_console()?
    }
    unsafe { charging_events::register_events(unplug, plug)? }
    Ok(())
}

fn main() -> Result<()> {
    println!("Hello, world!");
    elevate::elevate_if_needed()?;
    match env::args().collect::<Vec<String>>().get(1) {
        None => {
            core()?;
        }
        Some(arg) => {
            match arg.as_str() {
                "shutdown" => {
                    let older_proc = kill_duplicate()?;
                    if !older_proc {
                        println!("No other instance of Win Global GPU found.")
                    }
                }
                "dedicated" => {
                    set_programs()?;
                    unsafe {
                        registry::write_reg(PROGRAMS.get().unwrap(), registry::GpuMode::Dedicated)?
                    };
                    println!("Set global GPU to dedicated GPU.");
                }
                "integrated" => {
                    set_programs()?;
                    unsafe {
                        registry::write_reg(PROGRAMS.get().unwrap(), registry::GpuMode::Integrated)?
                    };
                    println!("Set global GPU to integrated GPU.");
                }
                "reset" => {
                    unsafe { registry::write_reg(&vec![], registry::GpuMode::None)? };
                    println!("Reset!");
                }
                err => {
                    eprintln!("Invalid argument {err}. Run `win_global_gpu.exe help` to see all commands.")
                }
            }
        }
    }

    // unsafe { registry::write_reg(&vec!(), registry::GpuMode::None)?;}
    // it doesnt return an error type it returns the stuff already in the var so ? doesn't work
    Ok(())
}
