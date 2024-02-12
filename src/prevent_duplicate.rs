use windows::core::HSTRING;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, INVALID_HANDLE_VALUE};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
    PAGE_READWRITE,
};
use windows::Win32::System::Threading::{
    OpenProcess, TerminateProcess, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE,
};

use crate::error::Result;

// size of 32 bit integer, which is what windows uses for PIDs
const BUF_SIZE: u32 = std::mem::size_of::<u32>() as u32; // should just be 4 but why not

unsafe fn write_pid_to_shared_mem() -> Result<()> {
    let name = HSTRING::from("Global\\net.reticivis.win_global_gpu");
    // create shared memory
    let map_file = CreateFileMappingW(
        INVALID_HANDLE_VALUE, // use paging file
        None,
        PAGE_READWRITE,
        0,        // upper byte, always 0
        BUF_SIZE, // lower byte
        &name,
    )?;
    // get pointer to shared memory
    let p_buf = MapViewOfFile(map_file, FILE_MAP_ALL_ACCESS, 0, 0, BUF_SIZE as usize);
    // get PID
    let pid = std::process::id();
    // write PID to memory
    std::ptr::write_volatile(p_buf.Value as *mut u32, pid);
    // free pointer-like thing
    UnmapViewOfFile(p_buf)?;
    Ok(())
}

pub unsafe fn kill_older_process() -> Result<bool> {
    let name = HSTRING::from("Global\\net.reticivis.win_global_gpu");

    // https://learn.microsoft.com/en-us/windows/win32/memory/creating-named-shared-memory?redirectedfrom=MSDN
    // try to open the shared memory
    match OpenFileMappingW(FILE_MAP_ALL_ACCESS.0, false, &name) {
        Ok(file_mapping) => {
            // another process is open!
            // grab the pointer to shared memory
            let p_buf = MapViewOfFile(file_mapping, FILE_MAP_ALL_ACCESS, 0, 0, BUF_SIZE as usize);
            // read PID
            let pid = std::ptr::read_volatile(p_buf.Value as *mut u32);
            println!(
                "Found older Win Global GPU instance running with PID {}. Killing...",
                pid
            );
            // grab handle to process with proper perms
            let process_handle =
                OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_TERMINATE, false, pid)?;
            // kill process
            TerminateProcess(process_handle, 0)?;
            println!("Killed!");
            // release handles, might not be necessary but is good practice
            UnmapViewOfFile(p_buf)?;
            CloseHandle(process_handle)?;
            // now that the old process is killed, the memory is still open, we need to write our new PID to it
            write_pid_to_shared_mem()?;
            Ok(true)
        }
        Err(e) => {
            if e == ERROR_FILE_NOT_FOUND.into() {
                // nothing open!
                // signal that we exist now
                write_pid_to_shared_mem()?;
                Ok(false)
            } else {
                // some other issue D:
                Err(e.into())
            }
        }
    }
}
