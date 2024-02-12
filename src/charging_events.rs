use crate::error::Result;
use std::sync::OnceLock;
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Power::{
    PoAc, PoDc, PoHot, RegisterPowerSettingNotification, POWERBROADCAST_SETTING,
};
use windows::Win32::System::SystemServices::GUID_ACDC_POWER_SOURCE;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW, CW_USEDEFAULT,
    DEVICE_NOTIFY_WINDOW_HANDLE, HWND_MESSAGE, MSG, PBT_APMPOWERSTATUSCHANGE,
    PBT_POWERSETTINGCHANGE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_POWERBROADCAST, WNDCLASSW,
};

static UNPLUG_HANDLE: OnceLock<fn()> = OnceLock::new();
static PLUG_HANDLE: OnceLock<fn()> = OnceLock::new();
pub unsafe fn register_events(unplug: fn(), plug: fn()) -> Result<()> {
    // set callback functions
    UNPLUG_HANDLE
        .set(unplug)
        .expect("Unable to set unplug callback");
    PLUG_HANDLE.set(plug).expect("Unable to set plug callback");
    // register stuff needed to make a window
    let instance = GetModuleHandleW(None)?;
    let window_class = HSTRING::from("win_global_gpu");
    let wc = WNDCLASSW {
        hInstance: instance.into(),
        lpszClassName: PCWSTR(window_class.as_ptr()),
        lpfnWndProc: Some(wndproc), // where we pass our event callback
        ..Default::default()
    };
    let atom = RegisterClassW(&wc);
    if atom == 0 {
        return Err("RegisterClass failed".into());
    }
    // we need a window to receive messages, make a hidden one
    let window = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        &window_class,
        &HSTRING::from("win_global_gpu"),
        WINDOW_STYLE::default(),
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        HWND_MESSAGE,
        None,
        instance,
        None,
    );
    // register for power setting broadcasts
    RegisterPowerSettingNotification(
        window,
        &GUID_ACDC_POWER_SOURCE,
        DEVICE_NOTIFY_WINDOW_HANDLE.0,
    )?;
    // enter a loop of listening for broadcasts
    let mut message = MSG::default();
    while GetMessageW(&mut message, window, 0, 0).into() {
        // actually send the message and its info to the callback
        // not 100% sure how this works but yknow it does
        DispatchMessageW(&message);
    }
    Ok(())
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // when getting any message
    match message {
        // if it's a power messsage
        WM_POWERBROADCAST => {
            // if it's a power setting change
            if wparam.0 == PBT_APMPOWERSTATUSCHANGE as usize
                || wparam.0 == PBT_POWERSETTINGCHANGE as usize
            {
                // we're given a pointer, dereference it to a struct
                let pbs: POWERBROADCAST_SETTING =
                    unsafe { *(lparam.0 as *const POWERBROADCAST_SETTING) };
                const AC: u8 = PoAc.0 as u8;
                const DC: u8 = PoDc.0 as u8;
                const HOT: u8 = PoHot.0 as u8;
                // call proper callback depending on returned setting
                match pbs.Data[0] {
                    AC => {
                        PLUG_HANDLE.get().expect("No plug callback set")();
                    }
                    DC | HOT => {
                        UNPLUG_HANDLE.get().expect("No unplug callback set")();
                    }
                    u => {
                        panic!("Unknown POWERBROADCAST_SETTING {u}")
                    }
                }
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}
