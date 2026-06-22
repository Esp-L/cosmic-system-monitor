use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub update_interval_ms: u64,
    pub show_cpu_pct: bool,
    pub show_cpu_temp: bool,
    pub show_ram_pct: bool,
    pub show_ram_used: bool,
    pub show_gpu_pct: bool,
    pub show_gpu_temp: bool,
    pub show_gpu_vram: bool,
    pub show_disk_pct: bool,
    pub show_disk_used: bool,
    pub show_net_speed: bool,
    pub show_net_total: bool,
    pub system_monitor_cmd: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            update_interval_ms: 2000,
            show_cpu_pct: true,
            show_cpu_temp: true,
            show_ram_pct: true,
            show_ram_used: false,
            show_gpu_pct: true,
            show_gpu_temp: false,
            show_gpu_vram: false,
            show_disk_pct: true,
            show_disk_used: true,
            show_net_speed: true,
            show_net_total: false,
            system_monitor_cmd: "gnome-system-monitor".to_string(),
        }
    }
}

impl Config {
    pub const VERSION: u64 = 1;
}
