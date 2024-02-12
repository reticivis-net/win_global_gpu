use crate::error::Result;
use windows::Win32::System::Console::{FreeConsole, GetConsoleProcessList};

pub unsafe fn hide_console() -> Result<()> {
    // by @interacsion on the Rust discord, not 100% how it works but it works
    // side effect is that RustRover's debug also gets detached, so this is only run in release mode
    println!("Releasing console...");
    let mut processes = [0; 2];
    if GetConsoleProcessList(&mut processes) == 1 {
        // free my boy console he aint do nothin wrong
        FreeConsole()?;
    }
    Ok(())
}
