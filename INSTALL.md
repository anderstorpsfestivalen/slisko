# Slisko Installation Guide

This guide covers installing Slisko as a systemd service on Raspberry Pi OS.

## Prerequisites

On Raspberry Pi OS, you may need to enable hardware interfaces:

### Enable SPI (for LED strip output)
```bash
sudo raspi-config
# Navigate to: Interface Options → SPI → Enable
```

### Enable I2C (if using I2C devices)
```bash
sudo raspi-config
# Navigate to: Interface Options → I2C → Enable
```

**Note:** A reboot is recommended after enabling hardware interfaces, but you can reboot at your convenience:
```bash
sudo reboot
```

## Quick Installation

1. Clone the repository (if you haven't already):
```bash
git clone https://github.com/anderstorpsfestivalen/slisko.git
cd slisko
```

2. Run the installation script:
```bash
sudo ./rpi/install.sh
```

The script will:
- Build the binary
- Create a `slisko` user with GPIO/SPI access
- Install to `/opt/slisko/`
- Verify hardware device access
- Set up systemd service
- Start the service automatically

## What Gets Installed

### Files and Directories

- **Binary**: `/opt/slisko/slisko`
- **Configurations**: `/opt/slisko/configurations/`
- **Service**: `/etc/systemd/system/slisko.service`
- **Environment**: `/etc/default/slisko`

### User and Groups

The `slisko` user is created and added to:
- `spi` group (for SPI LED output)
- `gpio` group (for GPIO button input)

## Configuration

### Changing Settings

Edit `/etc/default/slisko` to change settings:

```bash
sudo nano /etc/default/slisko
```

Example settings:
```bash
# Which configuration file to use
CONFIG_FILE=configurations/7609.toml

# Frame rate
FPS=60

# Additional arguments (e.g., enable SPI output)
SLISKO_ARGS="--spi"
```

After changing settings:
```bash
sudo systemctl restart slisko
```

### Editing Configurations

Configuration files are in `/opt/slisko/configurations/`:

```bash
sudo nano /opt/slisko/configurations/9010.toml
```

After editing configs:
```bash
sudo systemctl restart slisko
```

## Service Management

### View Status
```bash
sudo systemctl status slisko
```

### View Logs
```bash
# Follow logs in real-time
sudo journalctl -u slisko -f

# View last 100 lines
sudo journalctl -u slisko -n 100

# View logs since boot
sudo journalctl -u slisko -b
```

### Start/Stop/Restart
```bash
sudo systemctl start slisko
sudo systemctl stop slisko
sudo systemctl restart slisko
```

### Enable/Disable Auto-start
```bash
# Enable (start on boot)
sudo systemctl enable slisko

# Disable (don't start on boot)
sudo systemctl disable slisko
```

## Updating

To update Slisko after pulling new code:

```bash
cd /path/to/slisko/repo
git pull
sudo ./rpi/install.sh
```

The install script will rebuild and replace the binary while keeping your configurations.

## Uninstallation

To completely remove Slisko:

```bash
cd /path/to/slisko/repo
sudo ./rpi/uninstall.sh
```

The script will prompt before removing the environment file and user account.

## Troubleshooting

### Service won't start

Check logs for errors:
```bash
sudo journalctl -u slisko -n 50
```

Common issues:
- **Permission denied**: Check that slisko user is in `spi` and `gpio` groups
- **Config file not found**: Check `CONFIG_FILE` in `/etc/default/slisko`
- **DDP connection failed**: Check network and DDP host configuration

### GPIO access denied

If you see GPIO permission errors in logs:

1. **Check user group membership:**
```bash
groups slisko
# Should show: slisko spi gpio
```

2. **Manually add to gpio group if missing:**
```bash
sudo usermod -a -G gpio slisko
sudo systemctl restart slisko
```

3. **Check GPIO device permissions:**
```bash
ls -l /dev/gpiomem
# Should show: crw-rw---- 1 root gpio
```

4. **Verify systemd device access:**
```bash
sudo systemctl show slisko | grep DeviceAllow
# Should show: DeviceAllow=/dev/gpiomem rw
```

### LED/SPI output not working

For SPI output, ensure:
1. **SPI is enabled:** `sudo raspi-config` → Interface Options → SPI
2. **SPI device exists:**
```bash
ls -l /dev/spidev0.0
# Should show: crw-rw---- 1 root spi
```
3. **Config has SPI enabled:** `/etc/default/slisko` has `SLISKO_ARGS="--spi"`
4. **Service has been restarted:** `sudo systemctl restart slisko`

### Button input not working

For GPIO button input:
1. **Check GPIO is enabled** (usually enabled by default)
2. **Verify slisko is in gpio group:** `groups slisko`
3. **Check logs for GPIO errors:** `sudo journalctl -u slisko | grep -i gpio`
4. **Ensure config has buttons defined** in your `.toml` config file

### Check service permissions
```bash
# View effective user
sudo systemctl show slisko | grep User

# Check group membership
groups slisko

# Check device access
sudo systemctl show slisko | grep -E '(DeviceAllow|SupplementaryGroups)'
```

## Manual Installation

If you prefer not to use the install script:

1. Build: `go build`
2. Create user: `sudo useradd --system slisko`
3. Install binary: `sudo cp slisko /opt/slisko/`
4. Copy configs: `sudo cp -r configurations /opt/slisko/`
5. Install service: `sudo cp rpi/slisko.service /etc/systemd/system/`
6. Reload systemd: `sudo systemctl daemon-reload`
7. Enable: `sudo systemctl enable --now slisko`

## Development Mode

For development, you can still run directly without the service:

```bash
# Stop the service first
sudo systemctl stop slisko

# Run directly
go run . --config configurations/9010.toml --simulator
```

Remember to start the service again when done:
```bash
sudo systemctl start slisko
```
