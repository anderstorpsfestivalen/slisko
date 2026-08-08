# slisko firmware

Bakes the slisko render engine down onto the bong69 ESP32 board so it computes
patterns itself — no Raspberry Pi. See the design plan at
`.claude/plans/i-am-interested-in-goofy-canyon.md`.

## Crates

| crate | target | what |
|-------|--------|------|
| `slisko-core` | host + esp32 | `no_std + alloc` portable render engine and physical-strand mapper. Ported from the Go `pkg/*` + `patterns/*`. |
| `slisko-config` | host + esp32 | Shared baker-generated chassis, mapping, active patterns, shaper, outputs, and buttons. |
| `slisko-host` | host (std) | Runs `slisko-core` natively and streams the final mapped RGB strand over DDP. |
| `slisko-sim`  | host (std) | Optional persistent ggez viewer. Receives DDP independently and overlays LEDs on the chassis artwork. |
| `slisko-fw`   | xtensa-esp32-espidf | the ESP32 firmware. **Excluded from the workspace** (needs the esp toolchain). See `slisko-fw/README.md`. |

The shared config is emitted as `slisko-config/src/generated.rs` by the Go
`cmd/baker` tool (reuses `pkg/configuration` + `pkg/chassi`). The generated file
is committed so firmware builds do not require Go.

## Host build (stable toolchain)

```sh
cargo test                         # headless core/config/host tests
cargo test -p slisko-sim           # optional DDP/viewer logic tests

# Terminal 1: leave this open across runner rebuilds/restarts.
cargo run -p slisko-sim

# Terminal 2: render the baked patterns and send them to the viewer.
cargo run -p slisko-host

# Try one pattern with deterministic timing/input.
cargo run -p slisko-host -- --pattern colorcycler --seed 1 --hour 12
```

Both commands default to UDP port 4048 on localhost. `slisko-sim` retains the
last complete frame when the sender exits and accepts a restarted sender with a
new source port immediately. Run `--help` on either command for address, FPS,
seed, hour, pattern, and asset-path overrides.

To bake a different chassis before building any Rust target:

```sh
cd ..
go run ./cmd/baker -config configurations/7609.toml
```

## Conventions

- `edition = "2024"` everywhere.
- Dependency versions are resolved with `cargo add` (latest) — never pinned from memory.
- Lean on cargo-native esp tooling (`espup`, `esp-generate`, `cargo espflash`).
