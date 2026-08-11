# Firmware — ESP32 (bong69 / WT32-ETH01)

Targets `xtensa-esp32-espidf`. **Excluded from the host workspace** (it can't
compile for the host); build it from this directory with the esp toolchain.

## One-time toolchain setup (cargo-native)

```sh
cargo install espup espflash ldproxy
espup install                       # installs the Xtensa Rust fork + LLVM
. $HOME/export-esp.sh               # adds the toolchain to PATH (per shell)
cargo install esp-generate          # project scaffolder
```

The first `firmware` build downloads ESP-IDF (~GB) via `esp-idf-sys`.

The firmware dependencies and cargo/sdkconfig wiring are already checked in.
Set `SLISKO_CONFIG` before building to select a non-default configuration.

## Board facts (source: github.com/bobko69/8PortLEDDistro)

- **LED data outputs (clockless WS281x):** GPIO 1, 2, 3, 4, 5, 12, 14, 15.
  Classic ESP32 has 8 RMT channels → one per output. GPIO1/3 are also UART0
  TX/RX — flash over USB-C, log over the network (no WiFi fallback).
- **APA102 outputs:** configured clock/data pairs use SPI2 and then SPI3, with
  no chip-select. SPI0/1 remain reserved for flash/cache access, so the
  firmware supports at most two independent APA102 chains.
- **Ethernet is the only network path (LAN8720 RMII):** MDC=GPIO23, MDIO=GPIO18,
  RMII 50 MHz clock=GPIO0 (input), PHY power-enable=GPIO16, phy_addr=1.
  RMII data on GPIO 13/19/21/22/25/26/27. Disjoint from the LED pins.
- **Button/sensor headers:** H1=GPIO17/32/33, H2=GPIO34, H3=GPIO35, H4=GPIO36
  (34/35/36 are input-only).

## Flash & monitor

The checked-in Cargo runner flashes and starts a 115200-baud monitor. A normal
run drives the complete baked LED mapping, including GPIO1:

```sh
source ~/export-esp.sh
SLISKO_CONFIG=configurations/9010.toml cargo run
```

GPIO1 is also UART0 TX. For button/debug sessions, build and flash a temporary
variant that reserves GPIO1 for readable serial logs; the first LED output will
remain dark until a normal build is flashed again:

```sh
source ~/export-esp.sh
SLISKO_CONFIG=configurations/9010.toml cargo run --features uart-logs
```
