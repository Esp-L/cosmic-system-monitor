# COSMIC System Monitor Applet

<p align="center">
<img src="resources/preview.png" width="400" alt="Preview">
</p>

Monitor de sistema limpo e poderoso para o **COSMIC Desktop Environment**.

## O que é o COSMIC System Monitor?

Um applet leve de monitoramento de sistema que se integra perfeitamente ao COSMIC Desktop, mostrando métricas do sistema em tempo real no painel. Perfeito para usuários que desejam acompanhar o desempenho do sistema sem poluir a área de trabalho.

## Distribuições Suportadas

| Distribuição | Status |
|--------------|--------|
| Pop!_OS 22.04+ |ok |
| Ubuntu 22.04+ |ok |
| Fedora 38+ |ok |
| Arch Linux | ok |

> **Nota**: Este applet foi projetado especificamente para o COSMIC Desktop.

## Pré-requisitos

### 1. Rust Toolchain

Este projeto usa Rust, então você precisará do toolchain Rust que inclui o `cargo`.

**Debian/Ubuntu:**
```bash
sudo apt update && sudo apt install rustc cargo
```

**Fedora/CentOS/RHEL:**
```bash
sudo dnf install rust cargo
```

**Arch Linux:**
```bash
sudo pacman -S rust cargo
```

### 2. Just (Executor de Comandos)

**Debian/Ubuntu:**
```bash
sudo apt update && sudo apt install just
```

**Fedora/CentOS/RHEL:**
```bash
sudo dnf install just
```

**Arch Linux:**
```bash
sudo pacman -S just
```

### 3. Bibliotecas de Desenvolvimento do Sistema

**Debian/Ubuntu:**
```bash
sudo apt update && sudo apt install build-essential libsensors-dev libgtk-3-dev libdbus-1-dev pkg-config
```

**Fedora/CentOS/RHEL:**
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install lm_sensors-devel gtk3-devel dbus-devel pkg-config
```

**Arch Linux:**
```bash
sudo pacman -S base-devel lm_sensors gtk3 dbus pkgconf
```

### 4. Dependências do COSMIC

Como este applet usa `libcosmic`, você pode precisar de dependências adicionais:

**Pop!_OS:**
```bash
sudo apt update && sudo apt install libcosmic-dev
```

**Outras Distribuições:**
Pode ser necessário compilar `libcosmic` a partir do código-fonte. Verifique o [repositório libcosmic](https://github.com/pop-os/libcosmic) para mais informações.

## Instalação

### Passo 1: Clonar o Repositório

```bash
git clone https://github.com/marcossl10/cosmic-system-monitor.git
cd cosmic-system-monitor
```

### Passo 2: Compilar e Instalar

```bash
sudo just install
```

Isso irá:
- Compilar a aplicação em modo release
- Instalar o binário em `/usr/bin/cosmic-sys-monitor`
- Instalar entrada desktop em `/usr/share/applications/`
- Instalar ícone do app em `/usr/share/icons/hicolor/symbolic/apps/`
- Instalar metainfo em `/usr/share/metainfo/`

### Passo 3: Reiniciar o Painel COSMIC

Faça logout e login novamente, ou reinicie o painel COSMIC:

```bash
killall -9 cosmic-panel
```

O applet agora deve aparecer na configuração do seu painel.

## Uso

### Adicionando ao Painel

1. Abra as **Configurações**
2. Navegue até **Área de Trabalho** → **Painel**
3. Clique no botão **+** para adicionar um applet
4. Selecione **System Monitor**


## Funcionalidades

- 📊 **Uso de CPU** - Monitoramento de uso do processador em tempo real
- 💾 **Uso de Memória** - RAM usada (porcentagem e GB)
- 🎮 **Uso de GPU** - Uso, Temperatura e VRAM (porcentagem e GB)
- 💿 **Uso de Disco** - Espaço em disco usado (porcentagem e GB)
- 🌡️ **Temperatura** - Temperaturas de CPU e GPU
- 🌐 **Rede** - Velocidades de download/upload em tempo real (B/s, KB/s, MB/s)
- ⚙️ **Configurável** - Ative/desative métricas via menu popup
- 🎨 **Aparência Nativa** - Integração perfeita com o COSMIC Desktop
- ⚡ **Baixo Recurso** - Consumo mínimo de memória e CPU

## Solução de Problemas

### Applet Não Aparece

1. Verifique a instalação: `ls -la /usr/bin/cosmic-sys-monitor`
2. Verifique os logs: `journalctl -u cosmic-sys-monitor` (se executando como serviço)
3. Tente executar manualmente: `cosmic-sys-monitor`

### Compilação Falha

1. Certifique-se de que todas as dependências estão instaladas
2. Atualize o Rust: `rustup update`
3. Limpe a compilação: `just clean && cargo build --release`

### Sensores Não Detectados

```bash
sudo sensors-detect
sudo systemctl enable --now lm_sensors
```

## Compilando a Partir do Código-Fonte

### Compilação Debug
```bash
just build-debug
```

### Compilação Release
```bash
just build-release
```

### Executar Sem Instalar
```bash
just run
```

### Verificar Qualidade do Código
```bash
just check
```

## Desinstalação

```bash
sudo just uninstall
```

## Licença

Este projeto está licenciado sob a Licença MIT - veja o arquivo [LICENSE](LICENSE) para detalhes.

## Agradecimentos

- [pop-os/libcosmic](https://github.com/pop-os/libcosmic) - Biblioteca do COSMIC Desktop
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - Biblioteca de informações do sistema
