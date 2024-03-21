use anyhow::Result;
use winrt_toast::{Toast, ToastDuration, ToastManager};

const ICON: &[u8] = include_bytes!("../assets/win_global_gpu.ico");
pub fn register() -> Result<()> {
    // is this the worst fucking thing ever? absolutely. does it work? yes :tro:
    let temp = std::env::temp_dir().join("win_global_gpu.ico");
    // assume if it exists it's probably the one i want
    if !temp.exists() {
        std::fs::write(&temp, ICON)?;
    }

    Ok(winrt_toast::register(
        "net.reticivis.win_global_gpu",
        "Win Global GPU",
        Some(&temp),
    )?)
}

pub fn toast(content: &str) -> Result<()> {
    let manager = ToastManager::new("net.reticivis.win_global_gpu");
    let mut toast = Toast::new();
    toast.text1(content).duration(ToastDuration::Short);

    manager.show(&toast)?;
    Ok(())
}
