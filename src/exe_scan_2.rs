use anyhow::{anyhow, Result};
use std::ffi::c_void;
use std::string::FromUtf8Error;
use windows::core::s;
use windows::Win32::Foundation::GENERIC_READ;
use windows::Win32::Storage::FileSystem::CreateFileA;
use windows::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_DIRECTORY, FILE_FLAG_OVERLAPPED, FILE_SHARE_READ, FILE_SHARE_WRITE,
    OPEN_EXISTING,
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

    const BUFFER_SIZE: usize = 0x10000;
    const FULL_BUFFER_SIZE: usize = BUFFER_SIZE + std::mem::size_of::<u64>();
    let mut p_data: [u8; FULL_BUFFER_SIZE] = [0u8; FULL_BUFFER_SIZE];
    let pd_ptr = &mut p_data as *mut _ as *mut c_void;

    let mut bytes_returned: u32 = 0;

    while DeviceIoControl(
        handle,
        FSCTL_ENUM_USN_DATA,
        Some(med_ptr),
        std::mem::size_of::<MFT_ENUM_DATA_V0>() as u32,
        Some(pd_ptr),
        std::mem::size_of::<[u8; FULL_BUFFER_SIZE]>() as u32,
        Some(&mut bytes_returned as *mut u32),
        None,
    )
    .is_ok()
    {
        med.StartFileReferenceNumber = u64::from_ne_bytes(p_data[..8].try_into()?);
        println!("{} {}", med.StartFileReferenceNumber, bytes_returned);
        // TODO: ???
        let mut offset: usize = 8;
        while offset < bytes_returned as usize {
            let record: USN_RECORD_V2 =
                std::mem::transmute::<[u8; std::mem::size_of::<USN_RECORD_V2>()], USN_RECORD_V2>(
                    p_data[offset..offset + std::mem::size_of::<USN_RECORD_V2>()].try_into()?,
                );
            let name_bytes = p_data[offset + record.FileNameOffset as usize
                ..offset + record.FileNameOffset as usize + record.FileNameLength as usize]
                .to_vec();
            let name_words: Vec<u16> = name_bytes
                .chunks(2)
                .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]])) // Use from_le_bytes or from_be_bytes for little/big endian
                .collect();
            match String::from_utf16(&name_words) {
                Ok(n) => {
                    if record.FileAttributes & FILE_ATTRIBUTE_DIRECTORY.0 != 0 {
                        println!("DIR {}", n)
                    }
                    if n.ends_with(".exe") {
                        println!("EXE {}", n);
                    }
                    // println!("{}", n)
                }
                Err(e) => {
                    dbg!(e);
                    dbg!(p_data[offset + record.FileNameOffset as usize
                        ..offset
                            + record.FileNameOffset as usize
                            + record.FileNameLength as usize]
                        .to_vec());
                    panic!();
                }
            }
            offset += record.RecordLength as usize;
        }
    }

    Ok(())
}
