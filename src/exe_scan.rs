use super::sector_reader::SectorReader;
use anyhow::{anyhow, Result};
use ntfs::Ntfs;
use std::ffi::CStr;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
// use windows::Win32::Foundation::GetLastError;
use windows::Win32::Storage::FileSystem::GetLogicalDriveStringsA;

pub fn get_files() -> Result<Vec<String>> {
    let mut files: Vec<String> = vec![];
    dbg!(get_drives()?);
    for drive in get_drives()? {
        files.append(&mut scan_drive(&drive)?);
    }
    Ok(files)
}

pub fn scan_drive(drive: &String) -> Result<Vec<String>> {
    let f = File::open(&drive)?;
    let sr = SectorReader::new(f, 4096)?;
    let mut fs = BufReader::new(sr);
    let mut ntfs = Ntfs::new(&mut fs)?;
    ntfs.read_upcase_table(&mut fs)?;
    let root = ntfs.root_directory(&mut fs)?;
    scan_dir(root, fs)
}

fn scan_dir<T>(dir: ntfs::NtfsFile, mut fs: T) -> Result<Vec<String>>
where
    T: Read + Seek,
{
    let files: Vec<String> = vec![];
    let index = dir.directory_index(&mut fs)?;
    let mut iter = index.entries();
    while let Some(file) = iter.next(&mut fs) {
        let file = file?;
        let key = file.key().unwrap()?;
        if key.is_directory() {}
        dbg!(key.name().to_string()?);
    }
    Ok(files)
}

fn get_drives() -> Result<Vec<String>> {
    let mut drives: Vec<String> = vec![];

    const BUFSIZE: usize = 512;
    let mut lpbuffer: [u8; BUFSIZE] = [0u8; BUFSIZE];
    let rval;
    unsafe {
        rval = GetLogicalDriveStringsA(Some(&mut lpbuffer));
    }
    // https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getlogicaldrivestringsa#return-value
    if rval == 0 {
        // fail
        // let err = unsafe { GetLastError()? };
        Err(anyhow!("Reading drive letters failed!"))
    } else if rval as usize > BUFSIZE {
        // if not fail, rval is buffer size
        Err(anyhow!(
            "Reading drive letters failed! Buffer needs to be {rval} long but is only {BUFSIZE}."
        ))
    } else {
        for drive in lpbuffer[..rval as usize].split_inclusive(|c| *c == 0u8) {
            drives.push(CStr::from_bytes_with_nul(drive)?.to_str()?.to_string());
        }
        Ok(drives)
    }
}
