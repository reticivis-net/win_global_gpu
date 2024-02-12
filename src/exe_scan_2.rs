use crate::error::Result;
use crate::hstring_utils::*;
use rustc_hash::FxHashMap;
use std::ffi::c_void;
use windows::core::HSTRING;
use windows::Win32::Foundation::{GENERIC_READ, HANDLE, MAX_PATH};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FileIdType, FindFirstVolumeW, FindNextVolumeW, GetFinalPathNameByHandleW,
    GetVolumeInformationByHandleW, OpenFileById, FILE_ATTRIBUTE_DIRECTORY, FILE_FLAG_OVERLAPPED,
    FILE_ID_DESCRIPTOR, FILE_ID_DESCRIPTOR_0, FILE_NAME_NORMALIZED, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Ioctl::{FSCTL_ENUM_USN_DATA, MFT_ENUM_DATA_V0, USN_RECORD_V2};
use windows::Win32::System::IO::DeviceIoControl;

const BACKSLASH_UTF16: u16 = 92;

#[derive(Debug)]
struct MiniFile {
    // store only what we need
    name: HSTRING,
    parent: u64,
}

#[derive(Debug, Clone)]
struct MiniDir {
    name: HSTRING,
    full_name: Option<HSTRING>,
    parent: u64,
}

pub unsafe fn get_files_in_volume(volume: HSTRING) -> Result<Vec<HSTRING>> {
    // create the handle to the volume we want
    let handle = CreateFileW(
        &volume,
        GENERIC_READ.0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        None,
        OPEN_EXISTING,
        FILE_FLAG_OVERLAPPED,
        None,
    )?;

    // make sure this is NTFS
    let mut file_system_buffer: [u16; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];
    let mut volume_name_buffer: [u16; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];
    GetVolumeInformationByHandleW(
        handle,
        Some(&mut volume_name_buffer),
        None,
        None,
        None,
        Some(&mut file_system_buffer),
    )?;
    let mut volume_name = hstring_from_utf16_buffer(&volume_name_buffer).unwrap_or_default();
    if volume_name.is_empty() {
        volume_name = volume.clone()
    }
    println!("Scanning volume \"{volume_name}\"...");
    let file_system = hstring_from_utf16_buffer(&file_system_buffer)?;
    if file_system != "NTFS" {
        return Err(format!("{volume_name} is not NTFS, it is {file_system}.").into());
    }

    // old code attempting to get the ID from the handle that errored and isnt needed anymore
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

    // struct that we pass a pointer to windows and it reads and we modify
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
    let mut entries: u64 = 0;

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
            match hstring_from_utf16(
                &p_data[offset + record.FileNameOffset as usize
                    ..offset + record.FileNameOffset as usize + record.FileNameLength as usize],
            ) {
                Ok(n) => {
                    entries += 1;
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
                        if let Some(r) = ret {
                            dbg!(record, r);
                        }
                    } else {
                        let wide = n.as_wide();
                        // if ends in .exe
                        if wide.len() >= 4
                            && &wide[wide.len() - 4..] == HSTRING::from(".exe").as_wide()
                        {
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
        "Scanned {entries} entries. Found {} EXEs and {} dirs.\nBuilding tree...",
        exes.len(),
        dirs.len(),
    );
    // let mut full_file: String = String::new();
    let mut files: Vec<HSTRING> = vec![];
    for exe in exes {
        match minifile_to_path(&exe, &mut dirs, &handle) {
            Ok(path) => {
                // full_file.push_str(&path);
                // full_file.push_str("\n");
                files.push(clean_path(path)?);
            }
            Err(e) => {
                dbg!(("Tree failure:", exe, e));
            }
        }
    }
    println!("Tree built!");
    println!(
        "Found {} valid EXEs in volume \"{volume_name}\".",
        files.len()
    );
    //\nWriting to file...");
    // let mut file = File::create("EXEs.txt")?;
    // file.write_all(full_file.as_bytes())?;
    // println!("Wrote!");
    Ok(files)
}

fn clean_path(path: HSTRING) -> Result<HSTRING> {
    replace(
        // strip double backslashes
        replace(path, HSTRING::from(r"\\?\"), HSTRING::new())?, // strip weird long path thing
        HSTRING::from(r"\\"),
        HSTRING::from(r"\"),
    )
}

fn minifile_to_path(
    file: &MiniFile,
    dirs: &mut FxHashMap<u64, MiniDir>,
    handle: &HANDLE,
) -> Result<HSTRING> {
    // convert a MiniFile to a full resolved path string
    // parent path, set later
    let parent: HSTRING;
    // if the parent is in the dirs hashmap
    // clone so we can later borrow dirs to pass to the recursive child
    if let Some(cloned_parent) = dirs.get(&file.parent).cloned() {
        if let Some(full) = &cloned_parent.full_name {
            parent = full.clone();
        } else {
            // no, a directory is not a file, but also i am not rewriting this function or doing
            // trait bullshit to get a minidir to work here, shut up
            let full_name = minifile_to_path(
                &MiniFile {
                    name: cloned_parent.name.clone(),
                    parent: cloned_parent.parent,
                },
                dirs,
                handle,
            )?;
            // another get_mut cause only 1 can mut borrow at a time and we just mut borrowed
            dirs.get_mut(&file.parent).unwrap().full_name = Some(full_name.clone());
            parent = full_name;
        }
    } else {
        // parent isnt in the hashmap, 99% chance this is just the root drive but let's look it up
        let unknown_path = unsafe { path_from_id(handle, &(file.parent as i64))? };
        dirs.insert(
            file.parent,
            MiniDir {
                full_name: Some(unknown_path.clone()),
                // doesnt matter
                name: HSTRING::new(),
                parent: 0,
            },
        );
        parent = unknown_path;
    }
    combine_hstring_paths(&parent, &HSTRING::from("\\"), &file.name)
}

unsafe fn path_from_id(handle: &HANDLE, id: &i64) -> Result<HSTRING> {
    // get the path from an unknown ID, most notably used for the root folder

    // weird struct to hold the ID
    let id_struct = FILE_ID_DESCRIPTOR {
        dwSize: std::mem::size_of::<FILE_ID_DESCRIPTOR>() as u32,
        Type: FileIdType,
        Anonymous: FILE_ID_DESCRIPTOR_0 { FileId: *id },
    };
    // open the file
    let file = OpenFileById(
        *handle,
        &id_struct as *const FILE_ID_DESCRIPTOR,
        GENERIC_READ.0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        None,
        FILE_FLAG_OVERLAPPED,
    )?;
    // get the path from the handle we opened
    const BSIZE: usize = 0x8000;
    let mut lpsz_file_path: [u16; BSIZE] = [0; BSIZE];
    let len = GetFinalPathNameByHandleW(file, &mut lpsz_file_path, FILE_NAME_NORMALIZED);
    if len == 0 {
        // err case
        Err(format!("GetFinalPathNameByHandleA failed on {id}.").into())
    } else {
        // convert to string and return
        let path = hstring_from_utf16_buffer(&lpsz_file_path[..len as usize])?;
        // println!("unknown ID {id}'s path is {path}");
        Ok(path)
    }
}

pub unsafe fn get_volumes() -> Result<Vec<HSTRING>> {
    // get all volumes, essentially filesystems, on the system
    let mut volumes: Vec<HSTRING> = vec![];
    // buffer, api docs say it cant be longer than MAX_PATH
    let mut volume_buffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    // get the first volume and weird handle object to find the rest
    let handle = FindFirstVolumeW(&mut volume_buffer)?;
    let mut valid = true;
    while valid {
        let hstring = HSTRING::from_wide(&volume_buffer)?;
        // dbg!(hstring);
        // volume paths have a trailing \ which breaks CreateFile, thanks michaelsoft binbows
        // trim trailing nulls cause it's a fixed buffer then trailing backslash
        let volume = truncate_hstring(truncate_hstring(hstring, 0)?, BACKSLASH_UTF16)?;
        volumes.push(volume);
        // weird windows way to find volume one at a time
        valid = FindNextVolumeW(handle, &mut volume_buffer).is_ok();
    }
    Ok(volumes)
}

pub unsafe fn get_all_files() -> Result<Vec<HSTRING>> {
    let mut files: Vec<HSTRING> = vec![];
    let volumes = get_volumes()?;
    println!("Found {} volumes.", volumes.len());
    for volume in volumes {
        match get_files_in_volume(volume) {
            Ok(mut vol_files) => {
                files.append(&mut vol_files);
            }
            Err(e) => {
                println!("Scanning failed due to {e}")
            }
        }
    }
    if files.is_empty() {
        Err("Found no files. Make sure you're running as admin!".into())
    } else {
        println!(
            "Finished scanning system. Found {} EXEs total.",
            files.len()
        );
        Ok(files)
    }
}
