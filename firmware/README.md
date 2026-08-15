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

Before the first firmware build, create the local WiFi credential list:

```sh
cp src/credentials.example.rs src/credentials.rs
```

Edit `WIFI_NETWORKS` in `credentials.rs`; entries are tried in declaration
order and an empty password selects an open network. The file is ignored by
Git so network passwords are not committed.

Each selected configuration must provide a top-level `Name`. The baker keeps
that value as the human-readable mDNS service instance and derives a lowercase,
hyphenated `.local` hostname from it. For example, `Name = "Cisco 7609"`
advertises `Cisco 7609` at `cisco-7609.local` through `_http._tcp` and
`_slisko._tcp` on port 80, plus `_ddp._udp` on port 4048.

## Board facts (source: github.com/bobko69/8PortLEDDistro)

- **LED data outputs (clockless WS281x):** GPIO 1, 2, 3, 4, 5, 12, 14, 15.
  Classic ESP32 has 8 RMT channels → one per output. GPIO1/3 are also UART0
  TX/RX — flash over USB-C, then log over the network.
- **APA102 outputs:** configured clock/data pairs use SPI2 and then SPI3, with
  no chip-select. SPI0/1 remain reserved for flash/cache access, so the
  firmware supports at most two independent APA102 chains.
- **Ethernet is preferred (LAN8720 RMII):** MDC=GPIO23, MDIO=GPIO18,
  RMII 50 MHz clock=GPIO0 (input), PHY power-enable=GPIO16, phy_addr=1.
  RMII data on GPIO 13/19/21/22/25/26/27. Disjoint from the LED pins.
- **WiFi is the fallback path:** after Ethernet has no DHCP lease for 30
  seconds, station mode cycles the local credential list. Ethernet has the
  higher route priority; once its DHCP lease returns, WiFi is stopped. The same
  30-second preference window applies after every later Ethernet loss.
- **Button/sensor headers:** H1=GPIO17/32/33, H2=GPIO34, H3=GPIO35, H4=GPIO36
  (34/35/36 are input-only).

The Cisco 7609 configuration uses GPIO32 and GPIO33 as active-low redundant
PSU connection inputs. Both low shows a green SUP720 management LED, either
high shows it red, and both high black out internally rendered patterns. DDP
frames intentionally remain unmodified. Firmware logs prefixed `GPIO DEBUG`
show both raw pin levels at boot, every raw edge, and each debounced combined
state transition.

After both PSUs have been off, reconnecting either supply starts the chassis
POST: all 131 physical LEDs hold amber for 10 seconds, switch off one at a time
at 90 ms per LED from the leftmost slot to the rightmost, and remain black for
8 seconds. Each link LED then appears after an independent random 0–5 second
negotiation delay, holds its steady link color for 100–500 ms, and begins its
own traffic flicker while activity ramps from zero to its time-of-day target
over 30 seconds. Status and panel LEDs appear immediately. Returning both PSUs
to off cancels and re-arms the sequence.

The `traffic_shaper.timezone` setting is a POSIX timezone. Both chassis use
Central European time with automatic CET/CEST changes. Per-port traffic burst
frequency continuously follows the configured low and peak windows after SNTP
has synchronized, while visible flicker cadence remains wall-clock fast. The
firmware uses peak intensity as its pre-sync fallback.

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

For the 7609 PSU GPIO trace, select its configuration instead:

```sh
source ~/export-esp.sh
SLISKO_CONFIG=configurations/7609.toml cargo run --features uart-logs
```

`GET /health` reports the selected transport and IP under `network`, detailed
Ethernet state under `ethernet`, and WiFi state, SSID, IP, and attempt count
under `wifi`.
