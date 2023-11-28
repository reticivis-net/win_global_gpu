use crate::hstring_utils::*;
use anyhow::Result;
use windows::core::HSTRING;
use windows::Management::Deployment::PackageManager;

pub fn find_windows_apps() -> Result<Vec<HSTRING>> {
    let mut apps: Vec<HSTRING> = vec![];
    // annoying OOP object to get packages
    let manager = PackageManager::new()?;
    // find packages
    for package in manager.FindPackages()? {
        // a package contains 1 or more apps
        for app in package.GetAppListEntries()? {
            // push the specific app ID the registry wants
            apps.push(app.AppUserModelId()?);
        }
    }
    Ok(apps)
}
