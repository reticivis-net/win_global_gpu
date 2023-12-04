// HKEY_CURRENT_USER\Software\Microsoft\DirectX\UserGpuPreferences

use anyhow::Result;
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CommitTransaction, CreateTransaction, RollbackTransaction,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyTransactedW, RegDeleteTreeW, RegOpenKeyTransactedW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_ALL_ACCESS, REG_OPTION_NON_VOLATILE, REG_SZ,
};

#[derive(PartialEq)]
pub enum GpuMode {
    Dedicated,
    Integrated,
    None,
}

pub unsafe fn write_reg(programs: &Vec<HSTRING>, mode: GpuMode) -> Result<()> {
    // transactions either are for error-handling or bulk writing, seems like something i want
    // wrap registry writes in a error-handling transaction manager

    // for some reason this function doesnt take "None"s, so i have to pass null pointers and 0s
    let transaction = CreateTransaction(
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
        0,
        0,
        0,
        PCWSTR::null(),
    )?;
    match write_reg_transaction(transaction, programs, mode) {
        Ok(_) => {
            CommitTransaction(transaction)?;
        }
        Err(_) => {
            RollbackTransaction(transaction)?;
        }
    }
    Ok(())
}

pub unsafe fn write_reg_transaction(
    transaction: HANDLE,
    programs: &Vec<HSTRING>,
    mode: GpuMode,
) -> Result<()> {
    let mut key = HKEY::default();
    match RegOpenKeyTransactedW(
        HKEY_CURRENT_USER,
        &HSTRING::from(r"Software\Microsoft\DirectX\UserGpuPreferences"),
        0,
        KEY_ALL_ACCESS,
        &mut key as *mut HKEY,
        transaction,
        None,
    ) {
        Ok(_) => {
            println!("Registry key exists, clearing.");
            RegDeleteTreeW(key, None)?;
        }
        Err(e) => {
            if e == ERROR_FILE_NOT_FOUND.into() {
                println!("Registry key doesn't exist, creating.");
                RegCreateKeyTransactedW(
                    HKEY_CURRENT_USER,
                    &HSTRING::from(r"Software\Microsoft\DirectX\UserGpuPreferences"),
                    0,
                    None,
                    REG_OPTION_NON_VOLATILE,
                    KEY_ALL_ACCESS,
                    None,
                    &mut key as *mut HKEY,
                    None,
                    transaction,
                    None,
                )?;
            } else {
                // some other issue D:
                return Err(e.into());
            }
        }
    }

    // if mode is None, user just wants to clear values, which we just did, so we're done.
    if mode != GpuMode::None {
        println!("Writing to registry...");
        let string = HSTRING::from(match mode {
            GpuMode::Dedicated => "GpuPreference=2;",
            GpuMode::Integrated => "GpuPreference=1;",
            GpuMode::None => {
                panic!("what")
            }
        });
        let mut stringu8: Vec<u8> = vec![];
        for word in string.as_wide() {
            for byte in word.to_ne_bytes() {
                stringu8.push(byte);
            }
        }
        // needs to be null terminated
        // i guess its 2 0s cause its utf-16? discord of all things crashed when there was just 1
        stringu8.push(0);
        stringu8.push(0);
        for program in programs {
            RegSetValueExW(key, program, 0, REG_SZ, Some(stringu8.as_slice()))?;
        }
    }
    RegCloseKey(key)?;
    Ok(())
}
