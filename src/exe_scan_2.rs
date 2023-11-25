use anyhow::Result;
use rustc_hash::FxHashMap;
use std::ffi::c_void;
use windows::core::s;
use windows::Win32::Foundation::GENERIC_READ;
use windows::Win32::Storage::FileSystem::{
    CreateFileA, FileIdBothDirectoryInfo, GetFileInformationByHandleEx, FILE_ATTRIBUTE_DIRECTORY,
    FILE_FLAG_OVERLAPPED, FILE_ID_BOTH_DIR_INFO, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Ioctl::{FSCTL_ENUM_USN_DATA, MFT_ENUM_DATA_V0, USN_RECORD_V2};
use windows::Win32::System::IO::DeviceIoControl;

#[derive(Debug)]
struct MiniFile {
    // store only what we need
    name: String,
    parent: u64,
}

#[derive(Debug)]
struct MiniDir {
    name: String,
    full_name: Option<String>,
    parent: u64,
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

    // let mut root_info = FILE_ID_BOTH_DIR_INFO::default();
    //
    // GetFileInformationByHandleEx(
    //     handle,
    //     FileIdBothDirectoryInfo,
    //     &mut root_info as *mut _ as *mut c_void,
    //     std::mem::size_of::<FILE_ID_BOTH_DIR_INFO>() as u32
    // )?;
    //
    // dbg!(root_info);
    // return Ok(());

    let mut med = MFT_ENUM_DATA_V0 {
        HighUsn: i64::MAX,
        LowUsn: i64::MIN,
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
    let mut dirs: FxHashMap<u64, MiniDir> = FxHashMap::default();
    let mut files: u64 = 0;

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
                    // this never happens
                    if record.FileReferenceNumber == 1407374883553285 {
                        dbg!(record, n);
                        panic!();
                    }
                    // is not directory, we'll traverse those later
                    if record.FileAttributes & FILE_ATTRIBUTE_DIRECTORY.0 != 0 {
                        let ret = dirs.insert(
                            record.FileReferenceNumber,
                            MiniDir {
                                name: n,
                                full_name: None,
                                parent: record.ParentFileReferenceNumber,
                            },
                        );
                    } else {
                        files += 1;
                        if n.ends_with(".exe") {
                            // is exe file
                            // println!("DIR {}", n)
                            exes.push(MiniFile {
                                name: n,
                                parent: record.ParentFileReferenceNumber,
                            })
                        }
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
    println!(
        "Found {} EXEs and {} dirs and {} files.\nBuilding tree...",
        exes.len(),
        dirs.len(),
        files
    );
    for exe in exes {
        let mut e = dirs.get(&exe.parent);
        loop {
            match e {
                Some(p) => {
                    e = dirs.get(&p.parent);
                    dbg!(p);
                }
                None => {
                    dbg!(e);
                    break;
                }
            }
        }
        break;
    }
    Ok(())
}
