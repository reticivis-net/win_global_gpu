use crate::exe_scan_2;
use crate::winapp_scan;
use anyhow::Result;
use windows::core::HSTRING;

pub fn get_all_programs() -> Result<Vec<HSTRING>> {
    println!("Scanning for Windows apps...");
    let mut programs = winapp_scan::find_windows_apps()?;
    println!("Found {} windows apps.", programs.len());
    let mut files = unsafe { exe_scan_2::get_all_files()? };
    programs.append(&mut files);
    println!("Found {} programs.", programs.len());
    Ok(programs)
}
