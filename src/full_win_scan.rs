use crate::error::Result;
use crate::exe_scan_2;
use crate::winapp_scan;
use windows::core::HSTRING;

pub fn get_all_programs() -> Result<Vec<HSTRING>> {
    let mut programs = winapp_scan::find_windows_apps()?;
    let mut files = unsafe { exe_scan_2::get_all_files()? };
    programs.append(&mut files);
    println!("Found {} programs.", programs.len());
    Ok(programs)
}
