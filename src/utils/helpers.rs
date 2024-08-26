use std::env;
use std::fs;
use colored::Colorize;
use crate::utils::{logger::loggers, config::init_config};
use std::time::Duration;



pub const fn minutes_to_duration(minutes: u64) -> Duration {
    Duration::from_secs(minutes * 60)
}

pub const fn hours_to_duration(hours: u64) -> Duration {
    Duration::from_secs(hours * 60 * 60)
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


pub fn initialize () {
    print_banner();
    print_warnings();
    loggers();
    init_config();
}