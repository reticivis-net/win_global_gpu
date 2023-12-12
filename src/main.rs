// #![windows_subsystem = "windows"] // this prevents the gui?

mod charging_events;
mod elevate;
mod exe_scan_2;
mod full_win_scan;
#[cfg(not(debug_assertions))]
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
            notification::toast(
                "Set global GPU to integrated GPU.\nRestart programs to see changes.",
            )
            .unwrap();
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
            notification::toast(
                "Set global GPU to dedicated GPU.\nRestart programs to see changes.",
            )
            .unwrap();
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
    PROGRAMS
        .set(full_win_scan::get_all_programs()?)
        .map_err(|_| anyhow!("Failed to store program list."))
}
fn core() -> Result<()> {
    kill_duplicate()?;
    set_programs()?;
    notification::register()?;
    // only hide console in release mode
    // RustRover's debug mode counts as an attached console and gets freed
    #[cfg(not(debug_assertions))]
    unsafe {
        hide_console::hide_console()?
    }
    unsafe { charging_events::register_events(unplug, plug)? }
    Ok(())
}

fn prog() -> Result<String> {
    // modified from https://stackoverflow.com/a/58113997/9044183
    env::current_exe()?
        .file_name()
        .ok_or(anyhow!("No file name"))?
        .to_os_string()
        .into_string()
        .map_err(|e| anyhow!("Failed to convert {e:?} to String"))
}

fn main() -> Result<()> {
    match env::args().collect::<Vec<String>>().get(1) {
        None => {
            elevate::elevate_if_needed()?;
            core()?;
        }
        Some(arg) => {
            match arg.as_str() {
                "shutdown" => {
                    elevate::elevate_if_needed()?;
                    let older_proc = kill_duplicate()?;
                    if !older_proc {
                        println!("No other instance of Win Global GPU found.")
                    }
                }
                "dedicated" => {
                    elevate::elevate_if_needed()?;
                    set_programs()?;
                    unsafe {
                        registry::write_reg(PROGRAMS.get().unwrap(), registry::GpuMode::Dedicated)?
                    };
                    println!("Set global GPU to dedicated GPU.");
                }
                "integrated" => {
                    elevate::elevate_if_needed()?;
                    set_programs()?;
                    unsafe {
                        registry::write_reg(PROGRAMS.get().unwrap(), registry::GpuMode::Integrated)?
                    };
                    println!("Set global GPU to integrated GPU.");
                }
                "reset" => {
                    elevate::elevate_if_needed()?;
                    unsafe { registry::write_reg(&vec![], registry::GpuMode::None)? };
                    println!("Reset!");
                }
                // env!("CARGO_PKG_VERSION") is the cargo version
                // idk why its an env variable but whatever
                "help" => {
                    println!(
                        include_str!("../help.txt"),
                        env!("CARGO_PKG_VERSION"),
                        prog()?
                    )
                }
                "about" => {
                    println!(include_str!("../about.txt"), env!("CARGO_PKG_VERSION"))
                }
                err => {
                    eprintln!(
                        "Invalid argument `{err}`. Run `{} help` to see all commands.",
                        prog()?
                    )
                }
            }
        }
    }
    Ok(())
}
