use anyhow::{anyhow, Result};
use std::ffi::c_void;
use windows::core::s;
use windows::Win32::Foundation::GENERIC_READ;
use windows::Win32::Storage::FileSystem::CreateFileA;
use windows::Win32::Storage::FileSystem::{
    FILE_FLAG_OVERLAPPED, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Ioctl::{FSCTL_ENUM_USN_DATA, MFT_ENUM_DATA_V0, USN_RECORD_V2};
use windows::Win32::System::IO::DeviceIoControl;

pub unsafe fn get_files() -> Result<()> {
    let handle = CreateFileA(
        s!("\\\\.\\C:"),
        GENERIC_READ.0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        None,
        OPEN_EXISTING,
        FILE_FLAG_OVERLAPPED,
        None,
    )?;

    let mut med = MFT_ENUM_DATA_V0 {
        HighUsn: i64::MAX,
        ..Default::default()
    };
    // evil pointer hacks
    // https://stackoverflow.com/a/24191977/9044183
    let med_ptr = &mut med as *mut _ as *mut c_void;

    const BUFFER_SIZE: usize = 0x10000usize + (std::mem::size_of::<u64>() * 8);
    let mut p_data: [u8; BUFFER_SIZE] = [0u8; BUFFER_SIZE];
    let pd_ptr = &mut p_data as *mut _ as *mut c_void;

    let mut bytes_returned: u32 = 0;

    while DeviceIoControl(
        handle,
        FSCTL_ENUM_USN_DATA,
        Some(med_ptr),
        std::mem::size_of::<MFT_ENUM_DATA_V0>() as u32,
        Some(pd_ptr),
        std::mem::size_of::<[u8; BUFFER_SIZE]>() as u32,
        Some(&mut bytes_returned as *mut u32),
        None,
    )
    .is_ok()
    {
        med.StartFileReferenceNumber = u64::from_ne_bytes(p_data[..8].try_into()?);
        println!("{} {}", med.StartFileReferenceNumber, bytes_returned);
        // TODO: ???
        let record: USN_RECORD_V2 = *(pd_ptr.add(8 * 8) as *mut USN_RECORD_V2);
        dbg!(record);
    }

    Ok(())
}
