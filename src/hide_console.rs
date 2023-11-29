use anyhow::Result;
use windows::Win32::System::Console::{FreeConsole, GetConsoleProcessList};
pub unsafe fn hide_console() -> Result<()> {
    let mut processes = [0; 2];
    if GetConsoleProcessList(&mut processes) == 1 {
        // free my boy console he aint do nothin wrong
        FreeConsole()?;
    }
    Ok(())
}
