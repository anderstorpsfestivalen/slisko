# slisko firmware

Bakes the slisko render engine down onto the bong69 ESP32 board so it computes
patterns itself — no Raspberry Pi. See the design plan at
`.claude/plans/i-am-interested-in-goofy-canyon.md`.

## Crates

| crate | target | what |
|-------|--------|------|
| `slisko-core` | host + esp32 | `no_std + alloc` portable render engine: pixel, chassi, pattern trait, fakers, traffic shaper, utils. Ported from the Go `pkg/*` + `patterns/*`. |
| `slisko-sim`  | host (std) | desktop simulator — runs `slisko-core` and draws pixels at their baked positions. Same logic as the firmware. |
| `slisko-fw`   | xtensa-esp32-espidf | the ESP32 firmware. **Excluded from the workspace** (needs the esp toolchain). See `slisko-fw/README.md`. |

The config + active patterns are emitted as `generated_config.rs` by the Go
`cmd/baker` tool (reuses `pkg/configuration` + `pkg/chassi`).

## Host build (stable toolchain)

```sh
cargo test           # core unit tests
cargo run -p slisko-sim
```

## Conventions

- `edition = "2024"` everywhere.
- Dependency versions are resolved with `cargo add` (latest) — never pinned from memory.
- Lean on cargo-native esp tooling (`espup`, `esp-generate`, `cargo espflash`).
