# slisko-fw — ESP32 firmware (bong69 / WT32-ETH01)

Targets `xtensa-esp32-espidf`. **Excluded from the host workspace** (it can't
compile for the host); build it from this directory with the esp toolchain.

## One-time toolchain setup (cargo-native)

```sh
cargo install espup espflash ldproxy
espup install                       # installs the Xtensa Rust fork + LLVM
. $HOME/export-esp.sh               # adds the toolchain to PATH (per shell)
cargo install esp-generate          # project scaffolder
```

The first `slisko-fw` build downloads ESP-IDF (~GB) via `esp-idf-sys`.

## Deps (resolve latest with `cargo add` — never pin from memory)

```sh
cargo add esp-idf-svc esp-idf-hal esp-idf-sys
cargo add ws2812-esp32-rmt-driver   # or smart-leds + RMT adapter
cargo add slisko-core --path ../slisko-core
```

Prefer regenerating the cargo/sdkconfig wiring with `esp-generate` /
`esp-idf-template` and grafting these crates in.

## Board facts (source: github.com/bobko69/8PortLEDDistro)

- **LED data outputs (clockless WS281x):** GPIO 1, 2, 3, 4, 5, 12, 14, 15.
  Classic ESP32 has 8 RMT channels → one per output. GPIO1/3 are also UART0
  TX/RX — flash over USB-C, log over the network (no WiFi fallback).
- **Ethernet is the only network path (LAN8720 RMII):** MDC=GPIO23, MDIO=GPIO18,
  RMII 50 MHz clock=GPIO0 (input), PHY power-enable=GPIO16, phy_addr=1.
  RMII data on GPIO 13/19/21/22/25/26/27. Disjoint from the LED pins.
- **Button/sensor headers:** H1=GPIO17/32/33, H2=GPIO34, H3=GPIO35, H4=GPIO36
  (34/35/36 are input-only).

## Flash & monitor

```sh
cargo espflash flash --monitor
```
