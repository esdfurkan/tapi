use tauri::{AppHandle, Emitter, Runtime};
use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind};
use std::time::Duration;

#[derive(Clone, serde::Serialize)]
struct SystemStats {
    cpu_usage: f32,
    ram_used: u64,
    ram_total: u64,
    ram_percentage: f32,
}

pub fn start_stats_thread<R: Runtime>(app: &AppHandle<R>) {
    let app_handle = app.clone();
    
    std::thread::spawn(move || {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        loop {
            // Sleep first to allow CPU reading to be accurate (first reading is always 0)
            std::thread::sleep(Duration::from_millis(1500));
            sys.refresh_all();
            
            let cpu_usage = sys.global_cpu_usage();
            let ram_used = sys.used_memory();
            let ram_total = sys.total_memory();
            let ram_percentage = if ram_total > 0 {
                (ram_used as f32 / ram_total as f32) * 100.0
            } else { 0.0 };

            let stats = SystemStats {
                cpu_usage,
                ram_used,
                ram_total,
                ram_percentage,
            };

            let _ = app_handle.emit("system-stats", stats);
        }
    });
}
