# Slisko

Slisko is a Rust LED pattern engine for network chassis. The same portable core
runs natively for development or on an ESP32. Native rendering is sent over DDP
to an independent ggez simulator, so the simulator can stay open while the
renderer is rebuilt and restarted.

## Workspace

| crate | purpose |
| --- | --- |
| `engine` | `no_std + alloc` chassis, patterns, fakers, traffic shaping, and output mapping |
| `baker` | Validates TOML configurations and compiles the line-card catalog into Rust source |
| `config` | Build-time-selected static configuration shared by host, simulator, and ESP32 |
| `host` | Native pattern runner and DDP sender |
| `sim` | Optional persistent ggez DDP viewer |
| `firmware` | Standalone `xtensa-esp32-espidf` firmware workspace |

The old Go application, Pixel simulator, Vue UI, Raspberry Pi deployment, and
hardware experiments are preserved under [`prev/`](prev/README.md).

## Run locally

The selected configuration is compiled automatically. It defaults to
`configurations/9010.toml`; set `SLISKO_CONFIG` to a root-relative or absolute
path to use another chassis.

```sh
# Terminal 1 — leave this running across host restarts.
cargo run -p sim

# Terminal 2 — stop, rebuild, and restart freely.
cargo run -p host
```

For the 7609, use the same selection for both processes:

```sh
SLISKO_CONFIG=configurations/7609.toml cargo run -p sim
SLISKO_CONFIG=configurations/7609.toml cargo run -p host
```

Both default to UDP port 4048. The simulator keeps its last complete frame when
the sender exits and accepts a restarted sender immediately. It sleeps while
idle, preserves the chassis aspect ratio while resizing, and caps redraws to the
current monitor refresh rate. Use `cargo run -p sim -- --fps 30` to request a
lower cap. `--help` lists pattern, seed, time, address, and asset overrides.

## Configuration tooling

Normal Cargo builds invoke the baker automatically and write generated code
only to Cargo's `OUT_DIR`. The baker constructs a `quote` token stream, validates
it as a `syn::File`, and formats it with `prettyplease`; it does not assemble
Rust syntax with strings.

```sh
cargo run -p baker -- check configurations/9010.toml
cargo run -p baker -- render configurations/7609.toml --output /tmp/7609.rs
```

See [INSTALL.md](INSTALL.md) for ESP32 setup and flashing.

## Known hardware and parity gaps

- APA102 uses the ESP32's two user SPI hosts (SPI2, then SPI3), so at most two
  independent clock/data chains can be configured. Shared-clock/multi-data
  output is not implemented.
- The 9010 GPIO and button assignments do not match the hard-coded bong69 board
  pin map. Board identity needs to become configuration data before validating
  those pins generically.
- The old HTTP/WebSocket Vue UI and mDNS behavior are not fully ported.
- WS281x chip timing still needs verification per supported LED type.
