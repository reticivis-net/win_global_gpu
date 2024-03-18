use std::backtrace::Backtrace;
use std::panic;
use std::panic::PanicInfo;
use windows::core::{w, HSTRING, PCWSTR};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR};
pub fn setup_panic_hook() {
    panic::set_hook(Box::new(panic_hook));
}

pub fn panic_hook(panic_info: &PanicInfo) {
    // let msg = match panic_info.payload().downcast_ref::<&str>() {
    //     Some(s) => *s,
    //     None => match panic_info.payload().downcast_ref::<String>() {
    //         Some(s) => &s[..],
    //         None => "Panic occurred but the message is not a string.",
    //     },
    // };
    //
    // let location = panic_info.location().unwrap(); // Note: This might panic if location is None, you might want to handle this case differently.
    //
    let backtrace = Backtrace::force_capture();
    //
    // // Construct the full error message
    // let error_message = format!(
    //     "Panic occurred in file '{}' at line {}:\n{}\nBacktrace:\n{:?}",
    //     location.file(),
    //     location.line(),
    //     msg,
    //     backtrace
    // );

    let pester_message = if cfg!(debug_assertions) {
        ""
    } else {
        "\nYou're running in release mode. You may not see the full backtrace if you don't compile with debug symbols or run in debug mode."
    };

    let error_message = format!("Please screenshot this and file an issue on the GitHub.\n{panic_info}\nstack backtrace:\n{backtrace}{pester_message}");
    eprintln!("{}", error_message);
    let err_hstring = HSTRING::from(error_message);
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(err_hstring.as_ptr()),
            w!("Win Global GPU"),
            MB_ICONERROR,
        );
    }
}
