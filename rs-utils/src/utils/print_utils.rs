use chrono::Local;
use colored::*;
use safe_lock::SafeLock;

static LOCK: SafeLock = SafeLock::new();

pub enum Status {
    Ok,
    Inf,
    Err,
}

pub fn print_status(status: Status, group: &str, message: &str) {
    let _guard = LOCK.lock();

    let status = match status {
        Status::Ok => "[OK]".green(),
        Status::Inf => "[INF]".cyan(),
        Status::Err => "[ERR]".red(),
    };

    let current_time = Local::now().format("%Y-%b-%d %H:%M:%S").to_string();
    println!("{} {} {}: {}", status, current_time, group, message);
}
