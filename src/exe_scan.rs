use super::sector_reader::SectorReader;
use anyhow::{anyhow, Result};
use ntfs::Ntfs;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use windows::Win32::Foundation::GetLastError;
use windows::Win32::Storage::FileSystem::GetLogicalDrives;

pub fn get_files() -> Result<Vec<String>> {
    let mut files: Vec<String> = vec![];
    for drive in get_drives()? {
        match scan_drive(&drive) {
            Ok(mut drive_files) => files.append(&mut drive_files),
            Err(e) => {
                eprintln!("Scanning drive {drive} failed due to {e}!")
            }
        }
    }
    Ok(files)
}

pub fn scan_drive(drive: &String) -> Result<Vec<String>> {
    let f = File::open(drive)?;
    let sr = SectorReader::new(f, 4096)?;
    let mut fs = BufReader::new(sr);
    let mut ntfs = Ntfs::new(&mut fs)?;
    // let mut i = 0;
    // while let Ok(file) = ntfs.file(&mut fs, i) {
    //     file;
    //     i+=1;
    // }
    // println!("{}", i);
    // Ok(vec!())
    // ntfs.read_upcase_table(&mut fs)?;
    let root = ntfs.root_directory(&mut fs)?;
    scan_dir(&root, &mut fs, &ntfs, drive)
}

fn scan_dir<T>(
    dir: &ntfs::NtfsFile,
    fs: &mut T,
    ntfs: &Ntfs,
    parent: &String,
) -> Result<Vec<String>>
where
    T: Read + Seek,
{
    let mut files: Vec<String> = vec![];
    let index = dir.directory_index(fs)?;
    let mut iter = index.entries();
    while let Some(file) = iter.next(fs) {
        let file = file?;
        let key = file.key().unwrap()?;
        let name = key.name().to_string()?;
        if name.starts_with("$") || name == "." {
            continue;
        }
        let full_name = format!("{parent}\\{name}");
        if key.is_directory() {
            let mut child = scan_dir(&file.to_file(ntfs, fs)?, fs, ntfs, &full_name)?;
            files.append(&mut child)
        } else if name.ends_with(".exe") {
            files.push(full_name)
        }
    }
    Ok(files)
}

fn get_drives() -> Result<Vec<String>> {
    let mut drives: Vec<String> = vec![];
    let drive_bitmask;
    unsafe {
        drive_bitmask = GetLogicalDrives();
    }
    // https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getlogicaldrivestringsa#return-value
    if drive_bitmask == 0 {
        // fail
        let err = unsafe { GetLastError().err() };
        match err {
            Some(e) => Err(anyhow!("Reading drive letters failed! {e}")),
            None => Err(anyhow!(
                "Reading drive letters failed due to an unknown error!"
            )),
        }
    } else {
        const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        for i in 0..26 {
            if (1 << i) & drive_bitmask != 0 {
                let letter = LETTERS.as_bytes()[i] as char;
                drives.push(format!("\\\\.\\{letter}:"));
            }
        }
        Ok(drives)
    }
}
