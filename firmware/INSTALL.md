# slisko firmware — install & test guide

Two separate toolchains. You can do a lot with just the first.

- **Host path** (`slisko-core` + `slisko-sim`): plain stable Rust. This is where
  the render engine + patterns get written and tested. **No special tooling.**
- **Firmware path** (`slisko-fw`): the Xtensa ESP toolchain + ESP-IDF, only
  needed to build/flash the actual board.

Platform notes below are for **macOS** (your machine); Linux differences are
called out inline.

---

## 1. Host path — works right now

You already have `cargo`/`rustc` 1.96 (rustup-managed). Nothing else to install.

```sh
cd firmware
cargo test               # slisko-core unit tests
cargo run -p slisko-sim  # runs the engine on the host
```

If `cargo` is NOT rustup-managed, install rustup first (it's also required for
the firmware path):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

That's the whole host setup. Pattern porting + the simulator happen here, fully
testable without hardware.

---

## 2. Firmware path — build & flash the bong69 board

### 2a. System prerequisites (macOS)

ESP-IDF (pulled in automatically on the first `slisko-fw` build) needs these:

```sh
brew install cmake ninja dfu-util python3 git wget
# optional but speeds up rebuilds:
brew install ccache
```

Linux (Debian/Ubuntu): `sudo apt install git wget flex bison gperf python3 \
python3-venv cmake ninja-build ccache libffi-dev libssl-dev dfu-util libusb-1.0-0`

### 2b. Rust ESP toolchain (cargo-native)

```sh
cargo install espup espflash ldproxy
espup install                 # installs the Xtensa Rust fork + LLVM (~as a toolchain)
. $HOME/export-esp.sh         # puts the toolchain on PATH — run in each new shell
```

- `espup` — manages the Xtensa Rust toolchain (ESP32 is Xtensa, not RISC-V).
- `ldproxy` — linker shim that `esp-idf-sys` needs.
- `espflash` — flashing + serial monitor (`espflash flash --monitor`).

Tip: add `. $HOME/export-esp.sh` to your `~/.zshrc` so it's always on PATH.

Optional scaffolder (recommended for getting the cargo/sdkconfig wiring right):

```sh
cargo install esp-generate
```

### 2c. USB-serial driver for the board

The bong69 programs over USB-C via an on-board USB-serial chip. Plug it in, then:

```sh
ls /dev/cu.*
```

- If you see something like `/dev/cu.usbserial-*` or `/dev/cu.wchusbserial*`,
  you're good.
- If nothing new appears, install the driver for the chip on your board:
  - **CH340 / CH9102** → WCH `CH34x`/`CH9102` macOS driver.
  - **CP2102 / CP2104** → Silicon Labs `CP210x` VCP driver.
  (Recent macOS often works driverless for CP210x/CH9102; CH340 usually needs
  the driver.)

### 2d. First firmware build

The first build downloads ESP-IDF and its tools (~1 GB) — slow once, cached
after. From `firmware/slisko-fw`:

```sh
cd firmware/slisko-fw
# add deps against the live registry (do NOT hand-pin versions):
cargo add esp-idf-svc esp-idf-hal esp-idf-sys
cargo add ws2812-esp32-rmt-driver
cargo add slisko-core --path ../slisko-core
cargo build                       # first build bootstraps ESP-IDF
```

You'll also need a `rust-toolchain.toml` selecting the esp channel and a
`.cargo/config.toml` with the target/runner. The cleanest way to get these
correct is to generate a throwaway project with `esp-generate`, copy those two
files in, then graft our `src/` + deps on top. (This is why `slisko-fw` is a
documented stub for now — that wiring is generated, not hand-written from
memory.)

### 2e. Flash & watch logs

```sh
espflash flash --monitor          # builds, flashes over USB-C, opens the monitor
```

Logs come over USB serial. Note GPIO1/3 double as UART0 **and** as LED outputs
1/2, so once those outputs are wired you'll want logs over the network instead.

---

## What needs hardware vs not

| Work | Needs board? | Toolchain |
|------|--------------|-----------|
| Render engine + patterns (`slisko-core`) | no | stable Rust |
| Simulator (`slisko-sim`) | no | stable Rust |
| The Go `baker` (config → `generated_config.rs`) | no | Go (already set up) |
| Ethernet / RMT output / DDP sink (`slisko-fw`) | yes | ESP toolchain (section 2) |

So: install section 2 when you want to flash. Everything else is buildable and
testable today with what you've got.
