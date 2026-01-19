use tracing_subscriber;
use tauri::{Window, Emitter};
use serde::Serialize;

pub fn init_logger() {
    tracing_subscriber::fmt::init();
}

#[derive(Serialize, Clone)]
pub struct ProgressPayload {
    pub current: usize,
    pub total: usize,
    pub percentage: f64,
    pub message: String,
}

pub trait ProgressLogger {
    fn log(&self, message: String);
    fn progress(&self, current: usize, total: usize, message: String);
}

impl ProgressLogger for Window {
    fn log(&self, message: String) {
        let _ = self.emit("translation-log", message);
    }

    fn progress(&self, current: usize, total: usize, message: String) {
        let percentage = if total > 0 {
             (current as f64 / total as f64) * 100.0
        } else { 0.0 };
        
        let payload = ProgressPayload {
            current,
            total,
            percentage,
            message: message.clone(),
        };
        
        // Emit progress specific event
        let _ = self.emit("translation-progress", payload);
        // Also emit log so it shows in the log output too
        let _ = self.emit("translation-log", message);
    }
}

pub struct ConsoleLogger;

impl ProgressLogger for ConsoleLogger {
    fn log(&self, message: String) {
        println!("{}", message);
    }

    fn progress(&self, current: usize, total: usize, message: String) {
         println!("[{}/{}] {}", current, total, message);
    }
}

pub fn log_debug(msg: &str) {
    // Synchronous file append for crash debugging
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    
    let timestamp = since_the_epoch.as_millis();

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("crash_debug.log") 
    {
        let _ = writeln!(file, "[{}] {}", timestamp, msg);
    }
}

