// HKEY_CURRENT_USER\Software\Microsoft\DirectX\UserGpuPreferences

use anyhow::Result;
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{
    CommitTransaction, CreateTransaction, RollbackTransaction,
};

pub unsafe fn write_reg(programs: Vec<HSTRING>) -> Result<()> {
    let transaction = CreateTransaction(
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
        0,
        0,
        0,
        PCWSTR::null(),
    )?;
    match write_reg_transaction(transaction, programs) {
        Ok(_) => {
            CommitTransaction(transaction)?;
        }
        Err(_) => {
            RollbackTransaction(transaction)?;
        }
    }
    Ok(())
}

pub unsafe fn write_reg_transaction(transaction: HANDLE, programs: Vec<HSTRING>) -> Result<()> {
    Ok(())
}
