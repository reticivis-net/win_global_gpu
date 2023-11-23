use winrt_notification::Toast;

pub(crate) fn toast(content: &str) {
    Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Win Global GPU")
        .text1(content)
        .show()
        .expect("unable to toast");
}