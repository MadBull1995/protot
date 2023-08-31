// logger.rs

use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref FILE: Mutex<std::fs::File> = {
        let path = "application.log";
        if Path::new(path).exists() {
            fs::remove_file(path).expect("Failed to remove old log file");
        }
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        Mutex::new(file)
    };
}

#[derive(Debug)]
pub enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
}

static mut PRINT_TO_CONSOLE: bool = false;

pub fn init(print_to_console: bool) {
    unsafe {
        PRINT_TO_CONSOLE = print_to_console;
    }
}

pub fn log(level: LogLevel, msg: &str) {
    let mut file = FILE.lock().unwrap();
    let log_str = format!("[{:?}] {}", level, msg);
    writeln!(file, "{}", log_str).expect("Could not write to log file");

    unsafe {
        if PRINT_TO_CONSOLE {
            println!("{}", log_str);
        }
    }
}
