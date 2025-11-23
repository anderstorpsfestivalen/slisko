#!/bin/bash
set -e

# Slisko LED Controller Installation Script
# This script installs slisko as a systemd service on Raspberry Pi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Installation paths
INSTALL_DIR="/opt/slisko"
SERVICE_FILE="slisko.service"
ENV_FILE="/etc/default/slisko"

echo -e "${GREEN}Slisko LED Controller - Installation Script${NC}"
echo "=============================================="
echo

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
    exit 1
fi

# Get the repository root directory (parent of rpi/)
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Check if we're in the right place
if [ ! -f "$REPO_ROOT/go.mod" ]; then
    echo -e "${RED}Error: Cannot find slisko repository root${NC}"
    echo "Please run this script from: sudo ./rpi/install.sh"
    exit 1
fi

cd "$REPO_ROOT"

echo -e "${YELLOW}Step 1:${NC} Checking dependencies..."
# Check for Go
if ! command -v go &> /dev/null; then
    echo -e "${RED}Error: Go is not installed. Please install Go first.${NC}"
    exit 1
fi
echo "  ✓ Go found: $(go version)"

# Check for git
if ! command -v git &> /dev/null; then
    echo -e "${YELLOW}Warning: git not found. Updates will need to be done manually.${NC}"
fi

echo
echo -e "${YELLOW}Step 2:${NC} Building binary..."
# Build the binary
if ! go build -o slisko .; then
    echo -e "${RED}Error: Build failed${NC}"
    exit 1
fi
echo "  ✓ Binary built successfully"

echo
echo -e "${YELLOW}Step 3:${NC} Creating slisko user and group..."
# Create user if it doesn't exist
if ! id -u slisko &> /dev/null; then
    useradd --system --no-create-home --shell /usr/sbin/nologin slisko
    echo "  ✓ Created slisko user"
else
    echo "  ✓ User slisko already exists"
fi

# Add slisko user to spi and gpio groups for hardware access
if getent group spi > /dev/null 2>&1; then
    usermod -a -G spi slisko
    echo "  ✓ Added slisko to spi group"
else
    echo -e "  ${YELLOW}⚠ Warning: spi group not found (SPI output may not work)${NC}"
fi

if getent group gpio > /dev/null 2>&1; then
    usermod -a -G gpio slisko
    echo "  ✓ Added slisko to gpio group"
else
    echo -e "  ${RED}⚠ Warning: gpio group not found${NC}"
    echo "    GPIO access may not work. You may need to add slisko to gpio manually:"
    echo "    sudo usermod -a -G gpio slisko"
fi

# Verify GPIO access
if [ -e /dev/gpiomem ]; then
    # Set proper permissions on gpiomem device
    chmod g+rw /dev/gpiomem 2>/dev/null || true
    echo "  ✓ GPIO device found: /dev/gpiomem"
else
    echo -e "  ${YELLOW}⚠ Warning: /dev/gpiomem not found (not critical)${NC}"
fi

# Check if SPI device exists
if [ -e /dev/spidev0.0 ]; then
    echo "  ✓ SPI device found: /dev/spidev0.0"
else
    echo -e "  ${YELLOW}⚠ Warning: /dev/spidev0.0 not found${NC}"
    echo "    SPI output will not work. Enable SPI with: sudo raspi-config"
fi

echo
echo -e "${YELLOW}Step 4:${NC} Installing to ${INSTALL_DIR}..."
# Create installation directory
mkdir -p "${INSTALL_DIR}"
mkdir -p "${INSTALL_DIR}/configurations"

# Stop service if it's running
if systemctl is-active --quiet slisko; then
    echo "  ⚠ Stopping existing slisko service..."
    systemctl stop slisko
fi

# Copy binary
cp slisko "${INSTALL_DIR}/"
chmod 755 "${INSTALL_DIR}/slisko"

# Copy configurations
cp -r configurations/* "${INSTALL_DIR}/configurations/"

# Set ownership
chown -R slisko:slisko "${INSTALL_DIR}"
echo "  ✓ Installed to ${INSTALL_DIR}"

echo
echo -e "${YELLOW}Step 5:${NC} Setting up environment file..."
# Create environment file if it doesn't exist
if [ ! -f "${ENV_FILE}" ]; then
    cat > "${ENV_FILE}" << 'EOF'
# Slisko LED Controller Configuration
# Override these settings as needed

# Which configuration file to use
CONFIG_FILE=configurations/9010.toml

# Frame rate (default: 60)
FPS=60

# Uncomment to enable specific outputs
# (DDP is configured in the config file)
#SLISKO_ARGS="--spi"
EOF
    echo "  ✓ Created ${ENV_FILE}"
else
    echo "  ✓ Environment file already exists at ${ENV_FILE}"
fi

echo
echo -e "${YELLOW}Step 6:${NC} Installing systemd service..."
# Copy service file from rpi/ directory
cp rpi/slisko.service /etc/systemd/system/
chmod 644 /etc/systemd/system/slisko.service

# Reload systemd
systemctl daemon-reload
echo "  ✓ Service file installed"

echo
echo -e "${YELLOW}Step 7:${NC} Enabling and starting service..."
# Enable service
systemctl enable slisko

# Start service
if systemctl start slisko; then
    echo "  ✓ Service started successfully"
else
    echo -e "${RED}Error: Failed to start service${NC}"
    echo "Check logs with: journalctl -u slisko -f"
    exit 1
fi

echo
echo -e "${GREEN}=============================================="
echo "Installation completed successfully!"
echo "==============================================${NC}"
echo
echo "Useful commands:"
echo "  View status:     sudo systemctl status slisko"
echo "  View logs:       sudo journalctl -u slisko -f"
echo "  Restart:         sudo systemctl restart slisko"
echo "  Stop:            sudo systemctl stop slisko"
echo "  Disable:         sudo systemctl disable slisko"
echo
echo "Configuration:"
echo "  Service file:    /etc/systemd/system/slisko.service"
echo "  Environment:     ${ENV_FILE}"
echo "  Installation:    ${INSTALL_DIR}"
echo "  Active config:   ${INSTALL_DIR}/$(grep CONFIG_FILE ${ENV_FILE} | cut -d= -f2)"
echo
echo -e "${YELLOW}Note:${NC} To change settings, edit ${ENV_FILE} then run:"
echo "  sudo systemctl restart slisko"
echo
