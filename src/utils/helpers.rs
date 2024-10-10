use std::env;
use std::fs;
use colored::Colorize;
use crate::utils::logger::loggers;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const fn minutes_to_duration(minutes: u64) -> Duration {
    Duration::from_secs(minutes * 60)
}

pub fn is_program_in_path(program: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(':') {
            let p_str = format!("{}/{}", p, program);
            if fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}

pub fn print_banner() {
    let s = r"
██████╗░ ░█████╗░ ██╗░░░██╗ ░█████╗░ ██╗░░░░░ ██╗ ███╗░░██╗ ██╗░░██╗
██╔══██╗ ██╔══██╗ ██║░░░██║ ██╔══██╗ ██║░░░░░ ██║ ████╗░██║ ██║░██╔╝
██████╔╝ ███████║ ╚██╗░██╔╝ ███████║ ██║░░░░░ ██║ ██╔██╗██║ █████═╝░
██╔══██╗ ██╔══██║ ░╚████╔╝░ ██╔══██║ ██║░░░░░ ██║ ██║╚████║ ██╔═██╗░
██║░░██║ ██║░░██║ ░░╚██╔╝░░ ██║░░██║ ███████╗ ██║ ██║░╚███║ ██║░╚██╗
╚═╝░░╚═╝ ╚═╝░░╚═╝ ░░░╚═╝░░░ ╚═╝░░╚═╝ ╚══════╝ ╚═╝ ╚═╝░░╚══╝ ╚═╝░░╚═╝";
    println!("{}", s.blue());
}

pub fn print_warnings() {
    if !is_program_in_path("yt-dlp") {
        log::warn!("yt-dlp is not installed! This Lantern instance will not be able to play tracks from YouTube until it is installed!")
    }
}


pub async fn initialize() {
    print_banner();
    print_warnings();
    loggers().await;
}



pub fn get_unix_timestamp() -> Duration {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
}

pub fn get_timestamp() -> u64 {
    get_unix_timestamp().as_secs()
}
