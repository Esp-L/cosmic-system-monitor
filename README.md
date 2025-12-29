# COSMIC System Monitor Applet

<img src="resources/preview.png" width="400" alt="Preview">

Clean and powerful system monitor for the COSMIC Desktop.

## Quick Installation

```bash
git clone https://github.com/marcossl10/cosmic-system-monitor.git
cd cosmic-system-monitor
sudo just install
```

## Functionalities
- CPU, RAM and GPU usage and temperature
- Real-time network (B/s, KB/s, MB/s)
- Native COSMIC look and feel

## Prerequisites

Before building and installing, ensure you have the following dependencies installed on your system.

### Installing Rust
The project is written in Rust. If you don't have Rust installed, you can install it using `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Installing `just`
`just` is a command runner that simplifies build and installation tasks.

**Debian/Ubuntu:**
```bash
sudo apt update
sudo apt install just
```

**Fedora/CentOS/RHEL:**
```bash
sudo dnf install just
```

**Arch Linux:**
```bash
sudo pacman -S just
```

### System Development Libraries
These libraries are often required for system monitoring and network functionalities.

**Debian/Ubuntu:**
```bash
sudo apt update
sudo apt install build-essential libsensors-dev libgtk-3-dev libdbus-1-dev
```

**Fedora/CentOS/RHEL:**
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install lm_sensors-devel gtk3-devel dbus-devel
```

**Arch Linux:**
```bash
sudo pacman -S base-devel lm_sensors gtk3 dbus
```
