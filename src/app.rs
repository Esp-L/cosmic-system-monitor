use cosmic::iced::{Alignment, Task, font};
use cosmic::iced::window::{self, Id};
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::cosmic_config::{ConfigGet, ConfigSet};
use cosmic::prelude::*;
use cosmic::widget;
use sysinfo::{System, CpuRefreshKind, RefreshKind, MemoryRefreshKind, Networks, Components, Disks};
use crate::config::Config;
use std::time::Duration;
use cosmic::iced::time;
use std::fs;
use std::sync::LazyLock;

use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
static NVML: LazyLock<Result<nvml_wrapper::Nvml, nvml_wrapper::error::NvmlError>> = LazyLock::new(nvml_wrapper::Nvml::init);

fn get_nvidia_temp() -> Option<f32> {
    if let Ok(nvml) = &*NVML {
        if let Some(pci_slot) = get_nvidia_pci_slot() {
            if let Ok(device) = nvml.device_by_pci_bus_id(pci_slot.as_str()) {
                if let Ok(temp) = device.temperature(TemperatureSensor::Gpu) {
                    return Some(temp as f32);
                }
            }
        }
        if let Ok(device) = nvml.device_by_index(0) {
            if let Ok(temp) = device.temperature(TemperatureSensor::Gpu) {
                return Some(temp as f32);
            }
        }
    }
    None
}

pub struct AppModel {
    core: cosmic::Core,
    popup: Option<Id>,
    config: Config,
    #[allow(dead_code)]
    config_handler: cosmic::cosmic_config::Config,
    system: System,
    networks: Networks,
    components: Components,
    disks: Disks,
    cpu_usage: f32,
    ram_usage: f32,
    gpu_usage: f32,
    cpu_temp: f32,
    gpu_temp: f32,
    gpu_vram_used: u64,
    gpu_vram_total: u64,
    download_speed: String,
    upload_speed: String,
    net_total_down: u64,
    net_total_up: u64,
}

#[derive(Clone, Debug)]
pub enum Message {
    Tick,
    TogglePopup,
    PopupClosed(Id),
    ToggleCpuPct,
    ToggleCpuTemp,
    ToggleRamPct,
    ToggleRamUsed,
    ToggleGpuPct,
    ToggleGpuTemp,
    ToggleGpuVram,
    ToggleDiskPct,
    ToggleDiskUsed,
    ToggleNetSpeed,
    ToggleNetTotal,
    OpenSystemMonitor,
    SetSystemMonitorCmd(String),
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "io.github.marcos.SysMonitor";

    fn core(&self) -> &cosmic::Core { &self.core }
    fn core_mut(&mut self) -> &mut cosmic::Core { &mut self.core }

    fn init(core: cosmic::Core, _flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let config_handler = cosmic::cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .unwrap_or_else(|_| cosmic::cosmic_config::Config::new(Self::APP_ID, 0).unwrap());
        let config = config_handler.get("config").unwrap_or_default();

        let mut system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
        );
        system.refresh_all();

        let networks = Networks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();

        let app = AppModel {
            core,
            popup: None,
            config,
            config_handler,
            system,
            networks,
            components,
            disks,
            cpu_usage: 0.0,
            ram_usage: 0.0,
            gpu_usage: 0.0,
            cpu_temp: 0.0,
            gpu_temp: 0.0,
            gpu_vram_used: 0,
            gpu_vram_total: 0,
            download_speed: "0 B/s".to_string(),
            upload_speed: "0 B/s".to_string(),
            net_total_down: 0,
            net_total_up: 0,
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        time::every(Duration::from_millis(self.config.update_interval_ms)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Tick => {
                self.system.refresh_cpu_all();
                self.system.refresh_memory();
                self.networks.refresh(true);
                self.components.refresh(true);
                self.disks.refresh(true);

                self.cpu_usage = self.system.global_cpu_usage();
                self.ram_usage = (self.system.used_memory() as f32 / self.system.total_memory() as f32) * 100.0;

                for component in &self.components {
                    let label = component.label();
                    let temp = component.temperature().unwrap_or(0.0);
                    if label == "Tctl" || label.contains("CPU") || label.contains("Package id 0") {
                        self.cpu_temp = temp;
                    } else if label == "edge" || label.contains("nvidia") {
                        self.gpu_temp = temp;
                    }
                }

                self.gpu_usage = read_gpu_usage().unwrap_or(0.0);

                if let Some((used, total)) = read_gpu_vram() {
                    self.gpu_vram_used = used;
                    self.gpu_vram_total = total;
                }

                if self.gpu_temp == 0.0 {
                    if let Some(temp) = get_nvidia_temp() {
                        self.gpu_temp = temp;
                    }
                }

                let mut total_down = 0u64;
                let mut total_up = 0u64;
                for (_, data) in &self.networks {
                    total_down += data.received();
                    total_up += data.transmitted();
                }

                self.download_speed = format_speed(total_down);
                self.upload_speed = format_speed(total_up);
                self.net_total_down += total_down;
                self.net_total_up += total_up;

                Task::none()
            }
            Message::TogglePopup => {
                if let Some(p) = self.popup.take() {
                    return destroy_popup(p);
                } else {
                    let new_id = window::Id::unique();
                    self.popup.replace(new_id);
                    let popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        Some((1, 1)),
                        None,
                        None,
                    );
                    return get_popup(popup_settings);
                }
            }
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
                Task::none()
            }
            Message::ToggleCpuPct => { self.config.show_cpu_pct = !self.config.show_cpu_pct; let _ = self.config_handler.set("config", &self.config); Task::none() }
            Message::ToggleCpuTemp => { self.config.show_cpu_temp = !self.config.show_cpu_temp; let _ = self.config_handler.set("config", &self.config); Task::none() }
            Message::ToggleRamPct => { self.config.show_ram_pct = !self.config.show_ram_pct; let _ = self.config_handler.set("config", &self.config); Task::none() }
            Message::ToggleRamUsed => { self.config.show_ram_used = !self.config.show_ram_used; let _ = self.config_handler.set("config", &self.config); Task::none() }
            Message::ToggleGpuPct => { self.config.show_gpu_pct = !self.config.show_gpu_pct; let _ = self.config_handler.set("config", &self.config); Task::none() }
            Message::ToggleGpuTemp => { self.config.show_gpu_temp = !self.config.show_gpu_temp; let _ = self.config_handler.set("config", &self.config); Task::none() }
            Message::ToggleGpuVram => { self.config.show_gpu_vram = !self.config.show_gpu_vram; let _ = self.config_handler.set("config", &self.config); Task::none() }
            Message::ToggleDiskPct => { self.config.show_disk_pct = !self.config.show_disk_pct; let _ = self.config_handler.set("config", &self.config); Task::none() }
            Message::ToggleDiskUsed => { self.config.show_disk_used = !self.config.show_disk_used; let _ = self.config_handler.set("config", &self.config); Task::none() }
            Message::ToggleNetSpeed => { self.config.show_net_speed = !self.config.show_net_speed; let _ = self.config_handler.set("config", &self.config); Task::none() }
            Message::ToggleNetTotal => { self.config.show_net_total = !self.config.show_net_total; let _ = self.config_handler.set("config", &self.config); Task::none() }
            Message::OpenSystemMonitor => {
                let cmd = self.config.system_monitor_cmd.clone();
                if !cmd.trim().is_empty() {
                    let _ = std::process::Command::new(cmd).spawn();
                }
                Task::none()
            }
            Message::SetSystemMonitorCmd(value) => {
                self.config.system_monitor_cmd = value;
                let _ = self.config_handler.set("config", &self.config);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let text_size = 12;
        let bold_font = font::Font { weight: font::Weight::Bold, ..Default::default() };
        let config = &self.config;

        let mut segments: Vec<Element<'_, Message>> = Vec::new();

        // CPU
        let cpu_on = config.show_cpu_pct || config.show_cpu_temp;
        if cpu_on {
            let mut parts: Vec<String> = Vec::new();
            if config.show_cpu_pct { parts.push(format!("{:.0}%", self.cpu_usage)); }
            if config.show_cpu_temp { parts.push(format!("{:.0}°C", self.cpu_temp)); }
            segments.push(metric_widget("CPU", &parts.join(" "), text_size, bold_font));
        }

        // RAM
        let ram_on = config.show_ram_pct || config.show_ram_used;
        if ram_on {
            let mut parts: Vec<String> = Vec::new();
            if config.show_ram_pct { parts.push(format!("{:.0}%", self.ram_usage)); }
            if config.show_ram_used {
                let used_gb = self.system.used_memory() as f64 / 1_073_741_824.0;
                let total_gb = self.system.total_memory() as f64 / 1_073_741_824.0;
                parts.push(format!("{:.1}/{:.1} GB", used_gb, total_gb));
            }
            segments.push(metric_widget("RAM", &parts.join(" "), text_size, bold_font));
        }

        // GPU
        let gpu_on = config.show_gpu_pct || config.show_gpu_temp || config.show_gpu_vram;
        if gpu_on {
            let mut parts: Vec<String> = Vec::new();
            if config.show_gpu_pct { parts.push(format!("{:.0}%", self.gpu_usage)); }
            if config.show_gpu_temp && self.gpu_temp > 0.0 { parts.push(format!("{:.0}°C", self.gpu_temp)); }
            if config.show_gpu_vram && self.gpu_vram_total > 0 {
                let used = self.gpu_vram_used / 1024 / 1024;
                let total = self.gpu_vram_total / 1024 / 1024;
                parts.push(format!("{}/{} MB", used, total));
            }
            segments.push(metric_widget("GPU", &parts.join(" "), text_size, bold_font));
        }

        // DISK
        let disk_on = config.show_disk_pct || config.show_disk_used;
        if disk_on {
            let mut parts: Vec<String> = Vec::new();
            for d in &self.disks {
                let mount = d.mount_point().to_string_lossy().to_string();
                if mount == "/" || mount.starts_with("/home") {
                    let total = d.total_space() / 1024 / 1024 / 1024;
                    let available = d.available_space() / 1024 / 1024 / 1024;
                    let used = total - available;
                    let usage = if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 };
                    if config.show_disk_pct { parts.push(format!("{:.0}%", usage)); }
                    if config.show_disk_used { parts.push(format!("{}/{} GB", used, total)); }
                    break;
                }
            }
            if parts.is_empty() { parts.push("N/A".to_string()); }
            segments.push(metric_widget("DISK", &parts.join(" "), text_size, bold_font));
        }

        // NET
        let net_on = config.show_net_speed || config.show_net_total;
        if net_on {
            let mut parts: Vec<String> = Vec::new();
            if config.show_net_speed {
                parts.push(format!("↓{} ↑{}", self.download_speed, self.upload_speed));
            }
            if config.show_net_total {
                parts.push(format!("↓{} ↑{}", format_bytes(self.net_total_down), format_bytes(self.net_total_up)));
            }
            segments.push(metric_widget("NET", &parts.join(" "), text_size, bold_font));
        }

        // Join with pipes only between segments
        let mut row_children: Vec<Element<'_, Message>> = Vec::new();
        for (i, seg) in segments.into_iter().enumerate() {
            if i > 0 { row_children.push(widget::text(" | ").size(text_size).into()); }
            row_children.push(seg);
        }

        let content = widget::row(row_children).spacing(0).align_y(Alignment::Center);
        let main_btn = widget::button::custom(content)
            .on_press(Message::OpenSystemMonitor)
            .class(cosmic::theme::Button::AppletIcon);
        let click_area = widget::mouse_area(main_btn)
            .on_right_press(Message::TogglePopup);

        widget::autosize::autosize(click_area, widget::Id::unique()).into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let config = &self.config;
        let bold_font = font::Font { weight: font::Weight::Bold, ..Default::default() };

        let header = widget::row(vec![
            widget::text("Settings").size(16).font(bold_font).into(),
            widget::space().into(),
        ]);

        let cpu_section = widget::settings::section()
            .title("CPU")
            .add(widget::settings::item("Percentage", widget::toggler(config.show_cpu_pct).on_toggle(|_| Message::ToggleCpuPct)))
            .add(widget::settings::item("Temperature", widget::toggler(config.show_cpu_temp).on_toggle(|_| Message::ToggleCpuTemp)));

        let ram_section = widget::settings::section()
            .title("RAM")
            .add(widget::settings::item("Percentage", widget::toggler(config.show_ram_pct).on_toggle(|_| Message::ToggleRamPct)))
            .add(widget::settings::item("Used / Total", widget::toggler(config.show_ram_used).on_toggle(|_| Message::ToggleRamUsed)));

        let gpu_section = widget::settings::section()
            .title("GPU")
            .add(widget::settings::item("Percentage", widget::toggler(config.show_gpu_pct).on_toggle(|_| Message::ToggleGpuPct)))
            .add(widget::settings::item("Temperature", widget::toggler(config.show_gpu_temp).on_toggle(|_| Message::ToggleGpuTemp)))
            .add(widget::settings::item("VRAM", widget::toggler(config.show_gpu_vram).on_toggle(|_| Message::ToggleGpuVram)));

        let disk_section = widget::settings::section()
            .title("Disk")
            .add(widget::settings::item("Percentage", widget::toggler(config.show_disk_pct).on_toggle(|_| Message::ToggleDiskPct)))
            .add(widget::settings::item("Used / Total", widget::toggler(config.show_disk_used).on_toggle(|_| Message::ToggleDiskUsed)));

        let net_section = widget::settings::section()
            .title("Network")
            .add(widget::settings::item("Speed", widget::toggler(config.show_net_speed).on_toggle(|_| Message::ToggleNetSpeed)))
            .add(widget::settings::item("Total transferred", widget::toggler(config.show_net_total).on_toggle(|_| Message::ToggleNetTotal)));

        let behavior_section = widget::settings::section()
            .title("Behavior")
            .add(widget::settings::item(
                "Left-click opens",
                widget::text_input("gnome-system-monitor", &config.system_monitor_cmd)
                    .on_input(Message::SetSystemMonitorCmd)
                    .width(180),
            ));

        let content = widget::column(vec![
            header.into(),
            cpu_section.into(),
            ram_section.into(),
            gpu_section.into(),
            disk_section.into(),
            net_section.into(),
            behavior_section.into(),
        ])
            .spacing(16)
            .padding(16);

        self.core.applet.popup_container(content).into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

fn metric_widget<'a>(label: &str, value: &str, text_size: u16, bold_font: font::Font) -> Element<'a, Message> {
    widget::row(vec![
        widget::text(format!("{} ", label)).size(text_size).font(bold_font).into(),
        widget::text(value.to_string()).size(text_size).into(),
    ]).spacing(0).align_y(Alignment::Center).into()
}

fn format_speed(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B/s", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB/s", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB/s", bytes as f64 / 1024.0 / 1024.0)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0)
    } else {
        format!("{:.2} GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
    }
}

fn read_gpu_usage() -> Option<f32> {
    if let Ok(nvml) = &*NVML {
        if let Some(pci_slot) = get_nvidia_pci_slot() {
            if let Ok(device) = nvml.device_by_pci_bus_id(pci_slot.as_str()) {
                if let Ok(util) = device.utilization_rates() {
                    return Some(u64::from(util.gpu) as f32);
                }
            }
        }
        if let Ok(device) = nvml.device_by_index(0) {
            if let Ok(util) = device.utilization_rates() {
                return Some(u64::from(util.gpu) as f32);
            }
        }
    }
    for card in 0..=1 {
        let path = format!("/sys/class/drm/card{}/device/gpu_busy_percent", card);
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(usage) = content.trim().parse::<f32>() {
                return Some(usage);
            }
        }
    }
    None
}

fn read_gpu_vram() -> Option<(u64, u64)> {
    if let Ok(nvml) = &*NVML {
        if let Some(pci_slot) = get_nvidia_pci_slot() {
            if let Ok(device) = nvml.device_by_pci_bus_id(pci_slot.as_str()) {
                if let Ok(mem) = device.memory_info() {
                    return Some((mem.used, mem.total));
                }
            }
        }
        if let Ok(device) = nvml.device_by_index(0) {
            if let Ok(mem) = device.memory_info() {
                return Some((mem.used, mem.total));
            }
        }
    }
    for card in 0..=1 {
        let used_path = format!("/sys/class/drm/card{}/device/mem_info_vram_used", card);
        let total_path = format!("/sys/class/drm/card{}/device/mem_info_vram_total", card);
        if let (Ok(used_str), Ok(total_str)) = (fs::read_to_string(&used_path), fs::read_to_string(&total_path)) {
            if let (Ok(used), Ok(total)) = (used_str.trim().parse::<u64>(), total_str.trim().parse::<u64>()) {
                if total > 0 { return Some((used, total)); }
            }
        }
    }
    None
}

fn get_nvidia_pci_slot() -> Option<String> {
    for card in 0..=1 {
        let uevent_path = format!("/sys/class/drm/card{}/device/uevent", card);
        if let Ok(content) = fs::read_to_string(&uevent_path) {
            for line in content.lines() {
                if line.starts_with("PCI_SLOT_NAME=") {
                    return Some(line.strip_prefix("PCI_SLOT_NAME=")?.to_string());
                }
            }
        }
    }
    None
}
