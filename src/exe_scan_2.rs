use anyhow::Result;
use rustc_hash::FxHashMap;
use std::ffi::c_void;
use windows::core::s;
use windows::Win32::Foundation::GENERIC_READ;
use windows::Win32::Storage::FileSystem::{
    CreateFileA, OpenFileById, FILE_ATTRIBUTE_DIRECTORY, FILE_FLAG_OVERLAPPED, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING, FILE_ID_DESCRIPTOR, FILE_ID_DESCRIPTOR_0, FileIdType,
    GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION
};
use windows::Win32::System::Ioctl::{FSCTL_ENUM_USN_DATA, MFT_ENUM_DATA_V0, USN_RECORD_V2};
use windows::Win32::System::IO::DeviceIoControl;

#[derive(Debug)]
struct MiniFile {
    // store only what we need
    name: String,
    parent: u64,
}

struct MiniDir {
    name: String,
    full_name: String,
}

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

    // windows dumps pieces of the MFT into a buffer
    const BUFFER_SIZE: usize = 0x10000;
    const FULL_BUFFER_SIZE: usize = BUFFER_SIZE + std::mem::size_of::<u64>();
    let mut p_data: [u8; FULL_BUFFER_SIZE] = [0u8; FULL_BUFFER_SIZE];
    let pd_ptr = &mut p_data as *mut _ as *mut c_void;

    // it tells us how much it filled the buffer
    let mut bytes_returned: u32 = 0;

    let mut exes: Vec<MiniFile> = vec![];

    println!("Scanning for EXEs...");
    // open the MFT
    // https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-fsctl_enum_usn_data
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
        // setup the input to get the next buffer
        // the next start is the first 8 bytes
        med.StartFileReferenceNumber = u64::from_ne_bytes(p_data[..8].try_into()?);
        // println!("{} {}", med.StartFileReferenceNumber, bytes_returned);
        // start 8 bytes into the buffer
        let mut offset: usize = 8;
        // loop through every entry in the buffer
        while offset < bytes_returned as usize {
            // transmute the current record into an object
            let record: USN_RECORD_V2 =
                std::mem::transmute::<[u8; std::mem::size_of::<USN_RECORD_V2>()], USN_RECORD_V2>(
                    p_data[offset..offset + std::mem::size_of::<USN_RECORD_V2>()].try_into()?,
                );
            // the name isn't properly in the struct, we have to do pointer math to get it
            // https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2
            let name_bytes = p_data[offset + record.FileNameOffset as usize
                ..offset + record.FileNameOffset as usize + record.FileNameLength as usize]
                .to_vec();
            // convert the u8 bytes of UTF-16 into a rust string
            let name_words: Vec<u16> = name_bytes
                .chunks(2)
                .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]])) // Use from_le_bytes or from_be_bytes for little/big endian
                .collect();
            match String::from_utf16(&name_words) {
                Ok(n) => {
                    // is not directory, we'll traverse those later
                    if record.FileAttributes & FILE_ATTRIBUTE_DIRECTORY.0 == 0
                    // is exe file
                        && n.ends_with(".exe")
                    {
                        // println!("DIR {}", n)
                        exes.push(MiniFile {
                            name: n,
                            parent: record.ParentFileReferenceNumber,
                        })
                    }
                    // println!("{}", n)
                }
                Err(e) => {
                    // log error for debugging, but don't panic because while it is generally utf-16
                    // windows makes no guarantee it's valid utf-16 so just ignore it
                    dbg!(e);
                    dbg!(p_data[offset + record.FileNameOffset as usize
                        ..offset
                            + record.FileNameOffset as usize
                            + record.FileNameLength as usize]
                        .to_vec());
                    // panic!();
                }
            }
            offset += record.RecordLength as usize;
        }
    }
    println!("Found {} EXEs.\nBuilding tree...", exes.len());
    let mut dirs: FxHashMap<u64, MiniDir>;
    for exe in exes {
        let id = FILE_ID_DESCRIPTOR {
            dwSize: std::mem::size_of::<FILE_ID_DESCRIPTOR>() as u32,
            Type:FileIdType,
            Anonymous: FILE_ID_DESCRIPTOR_0 { FileId: exe.parent as i64}
        };
        let file = OpenFileById(
            handle,
            &id as *const FILE_ID_DESCRIPTOR,
            GENERIC_READ.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            FILE_FLAG_OVERLAPPED
        );
        match file {
            Ok(f) => {
                dbg!(f);
                let fileinfo = LPBY_HANDLE_FILE_INFORMATION {
                    ..Default::default()
                }
                GetFileInformationByHandle(

                )?;
                panic!();
            }
            Err(e) => {
                dbg!(exe, e);
            }
        }
    }
    Ok(())
}
