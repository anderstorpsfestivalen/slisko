#!/bin/bash
set -e

# Slisko LED Controller Uninstallation Script

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Installation paths
INSTALL_DIR="/opt/slisko"
SERVICE_FILE="/etc/systemd/system/slisko.service"
ENV_FILE="/etc/default/slisko"

echo -e "${YELLOW}Slisko LED Controller - Uninstallation Script${NC}"
echo "================================================"
echo

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
    exit 1
fi

# Check if installed
if [ ! -f "${SERVICE_FILE}" ] && [ ! -d "${INSTALL_DIR}" ]; then
    echo -e "${YELLOW}Slisko does not appear to be installed.${NC}"
    exit 0
fi

echo -e "${YELLOW}Step 1:${NC} Stopping and disabling service..."
# Stop service if running
if systemctl is-active --quiet slisko; then
    systemctl stop slisko
    echo "  ✓ Service stopped"
fi

# Disable service
if systemctl is-enabled --quiet slisko 2>/dev/null; then
    systemctl disable slisko
    echo "  ✓ Service disabled"
fi

echo
echo -e "${YELLOW}Step 2:${NC} Removing service file..."
# Remove service file
if [ -f "${SERVICE_FILE}" ]; then
    rm "${SERVICE_FILE}"
    systemctl daemon-reload
    echo "  ✓ Service file removed"
fi

echo
echo -e "${YELLOW}Step 3:${NC} Removing installation directory..."
# Remove installation directory
if [ -d "${INSTALL_DIR}" ]; then
    rm -rf "${INSTALL_DIR}"
    echo "  ✓ Removed ${INSTALL_DIR}"
fi

echo
echo -e "${YELLOW}Step 4:${NC} Removing environment file..."
# Ask about environment file
if [ -f "${ENV_FILE}" ]; then
    read -p "Remove environment file (${ENV_FILE})? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm "${ENV_FILE}"
        echo "  ✓ Removed ${ENV_FILE}"
    else
        echo "  ⚠ Kept ${ENV_FILE}"
    fi
fi

echo
echo -e "${YELLOW}Step 5:${NC} Handling slisko user..."
# Ask about user
read -p "Remove slisko user and group? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if id -u slisko &> /dev/null; then
        userdel slisko
        echo "  ✓ Removed slisko user"
    fi
else
    echo "  ⚠ Kept slisko user"
fi

echo
echo -e "${GREEN}=============================================="
echo "Uninstallation completed successfully!"
echo "==============================================${NC}"
echo
