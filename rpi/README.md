# Raspberry Pi Installation Files

This directory contains all the files needed to install and run Slisko as a systemd service on Raspberry Pi OS.

## Files

- **install.sh** - Installation script that sets up systemd service
- **uninstall.sh** - Removal script to cleanly uninstall Slisko
- **slisko.service** - Systemd unit file for running Slisko as a service

## Usage

See [../INSTALL.md](../INSTALL.md) for complete installation instructions.

Quick start:
```bash
sudo ./rpi/install.sh
```

## Requirements

- Raspberry Pi OS (Raspbian)
- Go (for building)
- Root access (via sudo)
