use anyhow::{anyhow, Result};
use std::env;
use std::ffi::{c_void, OsStr, OsString};
use windows::core::HSTRING;
use windows::Win32::Foundation::{GetLastError, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_NORMAL;

unsafe fn is_admin() -> Result<bool> {
    // get process
    let proc = GetCurrentProcess();
    // default value just to have something
    let mut handle: HANDLE = INVALID_HANDLE_VALUE;
    // open handle thing
    OpenProcessToken(proc, TOKEN_QUERY, &mut handle as *mut HANDLE)?;
    // set up vars
    let mut elevation = TOKEN_ELEVATION::default();
    let size: u32 = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
    let mut ret_size: u32 = size;
    // get info
    GetTokenInformation(
        handle,
        TokenElevation,
        Some(&mut elevation as *mut _ as *mut c_void),
        size,
        &mut ret_size,
    )?;
    // seems to be 0 = not admin, 1 = admin
    Ok(elevation.TokenIsElevated != 0)
}

unsafe fn elevate() -> Result<()> {
    // get path to self
    let self_path: String = env::current_exe()?
        .into_os_string()
        .into_string()
        .map_err(|_| anyhow!("Failed to convert self path to string."))?;
    // run it as admin
    ShellExecuteW(
        None,
        // https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew#runas
        &HSTRING::from("runas"), // run as admin
        &HSTRING::from(self_path),
        // pass args
        &HSTRING::from(
            env::args_os()
                .skip(1)
                .collect::<Vec<OsString>>()
                .join(OsStr::new(" ")),
        ),
        None,
        SW_NORMAL,
    );
    // error handling isnt automatic for this function for some reason
    GetLastError()?;
    Ok(())
}

pub fn elevate_if_needed() -> Result<()> {
    // if program is not running as admin, spawn new process as admin and exit
    // "reincarnate" as admin-privileged process
    let admin = unsafe { is_admin()? };
    if !admin {
        println!("Not running as admin! Requesting elevation...");
        unsafe {
            elevate()?;
            println!("Elevated new process. Goodbye!");
        }
        std::process::exit(0);
    }
    Ok(())
}
