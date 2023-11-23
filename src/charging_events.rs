use std::sync::OnceLock;
use windows::core::s;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Power::{
    PoAc, PoDc, PoHot, RegisterPowerSettingNotification, POWERBROADCAST_SETTING,
};
use windows::Win32::System::SystemServices::GUID_ACDC_POWER_SOURCE;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExA, DefWindowProcA, DispatchMessageA, GetMessageA, RegisterClassA, CW_USEDEFAULT,
    DEVICE_NOTIFY_WINDOW_HANDLE, HWND_MESSAGE, MSG, PBT_APMPOWERSTATUSCHANGE,
    PBT_POWERSETTINGCHANGE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_POWERBROADCAST, WNDCLASSA,
};
static UNPLUG_HANDLE: OnceLock<fn()> = OnceLock::new();
static PLUG_HANDLE: OnceLock<fn()> = OnceLock::new();
pub unsafe fn register_events(unplug:fn(), plug:fn()) {
    UNPLUG_HANDLE.set(unplug).expect("Unable to set unplug callback");
    PLUG_HANDLE.set(plug).expect("Unable to set plug callback");
    // register the
    let instance = GetModuleHandleA(None).unwrap();
    let window_class = s!("window");

    let wc = WNDCLASSA {
        hInstance: instance.into(),
        lpszClassName: window_class,
        lpfnWndProc: Some(wndproc),
        ..Default::default()
    };
    let atom = RegisterClassA(&wc);
    debug_assert!(atom != 0);
    let window = CreateWindowExA(
        WINDOW_EX_STYLE::default(),
        window_class,
        s!("test"),
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
    RegisterPowerSettingNotification(
        window,
        &GUID_ACDC_POWER_SOURCE,
        DEVICE_NOTIFY_WINDOW_HANDLE.0,
    )
    .expect("oopsie woopsie :3");
    let mut message = MSG::default();

    while GetMessageA(&mut message, window, 0, 0).into() {
        DispatchMessageA(&message);
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message {
        WM_POWERBROADCAST => {
            if wparam.0 == PBT_APMPOWERSTATUSCHANGE as usize
                || wparam.0 == PBT_POWERSETTINGCHANGE as usize
            {
                let pbs: POWERBROADCAST_SETTING =
                    unsafe { *(lparam.0 as *const POWERBROADCAST_SETTING) };
                const AC: u8 = PoAc.0 as u8;
                const DC: u8 = PoDc.0 as u8;
                const HOT: u8 = PoHot.0 as u8;
                match pbs.Data[0] {
                    AC => {
                        (PLUG_HANDLE.get().expect("No plug callback set"))();
                    }
                    DC | HOT => {
                        (UNPLUG_HANDLE.get().expect("No unplug callback set"))();
                    }
                    u => {
                        panic!("Unknown POWERBROADCAST_SETTING {u}")
                    }
                }
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcA(window, message, wparam, lparam) },
    }
}
