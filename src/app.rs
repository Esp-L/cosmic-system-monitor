use cosmic::iced::{Alignment, Task, font};
use cosmic::prelude::*;
use cosmic::widget::{self};
use sysinfo::{System, CpuRefreshKind, RefreshKind, MemoryRefreshKind, Networks, Components};
use crate::config::Config;
use std::time::Duration;
use cosmic::iced::time;
use std::fs;
use std::sync::LazyLock;

// NVML para NVIDIA
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
static NVML: LazyLock<Result<nvml_wrapper::Nvml, nvml_wrapper::error::NvmlError>> = LazyLock::new(nvml_wrapper::Nvml::init);

// Temperatura GPU via NVML
fn get_nvidia_temp() -> Option<f32> {
    if let Ok(nvml) = &*NVML {
        // Tenta encontrar GPU NVIDIA pelo PCI slot
        if let Some(pci_slot) = get_nvidia_pci_slot() {
            if let Ok(device) = nvml.device_by_pci_bus_id(pci_slot.as_str()) {
                if let Ok(temp) = device.temperature(TemperatureSensor::Gpu) {
                    return Some(temp as f32);
                }
            }
        }
        // Fallback: primeira GPU NVIDIA disponível
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
    config: Config,
    #[allow(dead_code)]
    config_handler: cosmic::cosmic_config::Config,
    system: System,
    networks: Networks,
    components: Components,
    cpu_usage: f32,
    ram_usage: f32,
    gpu_usage: f32,
    cpu_temp: f32,
    gpu_temp: f32,
    download_speed: String,
    upload_speed: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    Tick,
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
        let config = Config::default();

        let mut system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
        );
        system.refresh_all();

        let networks = Networks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();

        let app = AppModel {
            core,
            config,
            config_handler,
            system,
            networks,
            components,
            cpu_usage: 0.0,
            ram_usage: 0.0,
            gpu_usage: 0.0,
            cpu_temp: 0.0,
            gpu_temp: 0.0,
            download_speed: "0 B/s".to_string(),
            upload_speed: "0 B/s".to_string(),
        };

        (app, Task::none())
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
                
                self.cpu_usage = self.system.global_cpu_usage();
                self.ram_usage = (self.system.used_memory() as f32 / self.system.total_memory() as f32) * 100.0;

                for component in &self.components {
                    let label = component.label();
                    let temp = component.temperature().unwrap_or(0.0);
                    
                    if label == "Tctl" || label.contains("CPU") || label.contains("Package id 0") {
                        self.cpu_temp = temp;
                    } else if label.contains("nvidia") || label.contains("GPU") {
                        // Prioriza sensores explicitamente NVIDIA ou GPU
                        self.gpu_temp = temp;
                    } else if (label == "edge" || label == "junction" || label.contains("amdgpu")) && self.gpu_temp == 0.0 {
                        // Fallback para AMD
                        self.gpu_temp = temp;
                    }
                }

                self.gpu_usage = read_gpu_usage().unwrap_or(0.0);
                
                // Tenta obter temperatura do nvidia-smi se disponível
                if self.gpu_temp == 0.0 {
                    if let Some(temp) = get_nvidia_temp() {
                        self.gpu_temp = temp;
                    }
                }

                let mut total_down = 0;
                let mut total_up = 0;
                for (_, data) in &self.networks {
                    total_down += data.received();
                    total_up += data.transmitted();
                }
                
                self.download_speed = format_speed(total_down);
                self.upload_speed = format_speed(total_up);
                
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let text_size = 13; // Aumentado pouca coisa conforme pedido (de 12 para 13 ou 14)
        let bold_font = font::Font { weight: font::Weight::Bold, ..Default::default() };
        
        let cpu_info = format!("{:.0}% ({:.0}°C)", self.cpu_usage, self.cpu_temp);
        let ram_info = format!("{:.0}%", self.ram_usage);
        let gpu_info = if self.gpu_usage > 0.0 || self.gpu_temp > 0.0 {
            format!("{:.0}% ({:.0}°C)", self.gpu_usage, self.gpu_temp)
        } else {
            "N/A".to_string()
        };
        let net_info = format!("↓{} ↑{}", self.download_speed, self.upload_speed);
        
        let content = widget::row()
            .spacing(12)
            .align_y(Alignment::Center)
            .push(
                widget::row()
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .push(widget::text("CPU").size(text_size).font(bold_font))
                    .push(widget::text(cpu_info).size(text_size))
            )
            .push(widget::text("│").size(text_size))
            .push(
                widget::row()
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .push(widget::text("RAM").size(text_size).font(bold_font))
                    .push(widget::text(ram_info).size(text_size))
            )
            .push(widget::text("│").size(text_size))
            .push(
                widget::row()
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .push(widget::text("GPU").size(text_size).font(bold_font))
                    .push(widget::text(gpu_info).size(text_size))
            )
            .push(widget::text("│").size(text_size))
            .push(
                widget::row()
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .push(widget::text("NET").size(text_size).font(bold_font))
                    .push(widget::text(net_info).size(text_size))
            );

        let button = widget::button::custom(content)
            .class(cosmic::theme::Button::AppletIcon);

        widget::autosize::autosize(button, widget::Id::unique()).into()
    }
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

fn read_gpu_usage() -> Option<f32> {
    // Tenta NVIDIA primeiro usando NVML
    if let Ok(nvml) = &*NVML {
        // Tenta encontrar GPU NVIDIA pelo PCI slot
        if let Some(pci_slot) = get_nvidia_pci_slot() {
            if let Ok(device) = nvml.device_by_pci_bus_id(pci_slot.as_str()) {
                if let Ok(util) = device.utilization_rates() {
                    return Some(u64::from(util.gpu) as f32);
                }
            }
        }
        // Fallback: primeira GPU NVIDIA disponível
        if let Ok(device) = nvml.device_by_index(0) {
            if let Ok(util) = device.utilization_rates() {
                return Some(u64::from(util.gpu) as f32);
            }
        }
    }
    
    // Fallback para AMD/outros via sysfs
    for card in 0..=1 {
        let path = format!("/sys/class/drm/card{}/device/gpu_busy_percent", card);
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(usage) = content.trim().parse::<f32>() {
                if usage > 0.0 || card == 1 {
                     return Some(usage);
                }
            }
        }
    }
    
    None
}

fn get_nvidia_pci_slot() -> Option<String> {
    // Lê o uevent do dispositivo DRM para obter PCI_SLOT_NAME
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
