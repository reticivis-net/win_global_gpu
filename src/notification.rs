use anyhow::Result;
use winrt_notification::Toast;

pub fn toast(content: &str) -> Result<()> {
    Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Win Global GPU")
        .text1(content)
        .duration(winrt_notification::Duration::Short)
        .show()?;
    Ok(())
}
