use anyhow::{anyhow, Result};
use rustc_hash::FxHashMap;
use std::ffi::c_void;
use std::string::FromUtf16Error;
use windows::core::s;
use windows::Win32::Foundation::{GENERIC_READ, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileA, FileIdInfo, FileIdType, GetFileInformationByHandleEx, GetFinalPathNameByHandleA,
    OpenFileById, FILE_ATTRIBUTE_DIRECTORY, FILE_FLAG_OVERLAPPED, FILE_ID_DESCRIPTOR,
    FILE_ID_DESCRIPTOR_0, FILE_ID_INFO, FILE_NAME_NORMALIZED, FILE_SHARE_READ, FILE_SHARE_WRITE,
    OPEN_EXISTING,
};
use windows::Win32::System::Ioctl::{FSCTL_ENUM_USN_DATA, MFT_ENUM_DATA_V0, USN_RECORD_V2};
use windows::Win32::System::IO::DeviceIoControl;

#[derive(Debug)]
struct MiniFile {
    // store only what we need
    name: String,
    parent: u64,
}

#[derive(Debug, Clone)]
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

    // let mut root_info = FILE_ID_INFO::default();
    //
    // GetFileInformationByHandleEx(
    //     handle,
    //     FileIdInfo,
    //     &mut root_info as *mut _ as *mut c_void,
    //     std::mem::size_of::<FILE_ID_INFO>() as u32
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
            // convert the u8 bytes of UTF-16 into a rust string
            match string_from_utf16(
                &p_data[offset + record.FileNameOffset as usize
                    ..offset + record.FileNameOffset as usize + record.FileNameLength as usize],
            ) {
                Ok(n) => {
                    // this never happens
                    if record.Usn == 5
                    /*1407374883553285*/
                    {
                        dbg!(record, n);
                        panic!();
                    }
                    // is not directory, we'll traverse those later
                    if record.FileAttributes & FILE_ATTRIBUTE_DIRECTORY.0 != 0 {
                        if
                        /*n == "C" ||*/
                        n == "C:" || n == "C:\\" {
                            dbg!(&record, &n);
                            // panic!();
                        }
                        let ret = dirs.insert(
                            record.FileReferenceNumber,
                            MiniDir {
                                name: n,
                                full_name: None,
                                parent: record.ParentFileReferenceNumber,
                            },
                        );
                        if let Some(r) = ret {
                            dbg!(record, r);
                        }
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
    println!("{}", minifile_to_path(&exes[0], &mut dirs, &handle)?);
    Ok(())
}

fn minifile_to_path(
    file: &MiniFile,
    dirs: &mut FxHashMap<u64, MiniDir>,
    handle: &HANDLE,
) -> Result<String> {
    let parent: String;
    // brain-melting reference management because dirs needs to be unreferenced to be passed to the
    // recursions
    let parentmaybe = dirs.get(&file.parent);
    if parentmaybe.is_some() {
        let cloned_parent = parentmaybe.unwrap().clone();
        if let Some(full) = &cloned_parent.full_name {
            parent = full.clone();
        } else {
            let full_name = minifile_to_path(
                &MiniFile {
                    name: cloned_parent.name.clone(),
                    parent: cloned_parent.parent,
                },
                dirs,
                handle,
            )?;
            dirs.get_mut(&file.parent).unwrap().full_name = Some(full_name.clone());
            parent = full_name;
        }
    } else {
        let unknown_path = unsafe { path_from_id(handle, &(file.parent as i64))? };
        dirs.insert(
            file.parent,
            MiniDir {
                full_name: Some(unknown_path.clone()),
                // doesnt matter
                name: String::new(),
                parent: 0,
            },
        );
        parent = unknown_path;
    }
    Ok(format!("{}\\{}", parent, file.name))
}

unsafe fn path_from_id(handle: &HANDLE, id: &i64) -> Result<String> {
    // get the path from an unknown ID, most notably used for the root folder

    // weird struct to hold the ID
    let id = FILE_ID_DESCRIPTOR {
        dwSize: std::mem::size_of::<FILE_ID_DESCRIPTOR>() as u32,
        Type: FileIdType,
        Anonymous: FILE_ID_DESCRIPTOR_0 { FileId: *id },
    };
    // open the file
    let file = OpenFileById(
        *handle,
        &id as *const FILE_ID_DESCRIPTOR,
        GENERIC_READ.0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        None,
        FILE_FLAG_OVERLAPPED,
    )?;
    // get the path from the handle we opened
    const BSIZE: usize = 0x8000;
    let mut lpszFilePath: [u8; BSIZE] = [0; BSIZE];
    let len = GetFinalPathNameByHandleA(file, &mut lpszFilePath, FILE_NAME_NORMALIZED);
    if len == 0 {
        // err case
        Err(anyhow!("GetFinalPathNameByHandleA failed."))
    } else {
        // convert to string and return
        Ok(String::from_utf8(lpszFilePath[..len as usize].to_vec())?)
    }
}

fn string_from_utf16(utf16: &[u8]) -> Result<String, FromUtf16Error> {
    // thanks chatgpt for this btw
    let name_words: Vec<u16> = utf16
        // group by 2 bytes
        .chunks(2)
        // map bytes to words
        // yes [chunk[0], chunk[1]] is necessary because 🤓 size cant be known at compile time
        .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]]))
        // collect
        .collect();
    // vec of words to utf16
    String::from_utf16(&name_words)
}
