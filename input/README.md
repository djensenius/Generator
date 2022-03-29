# Inputs

Started from [rp2040-project-template](https://github.com/rp-rs/rp2040-project-template). Using the elf2uf2-rs instead of mod-probe.

To run:
```sh
cargo run --release
```

## Requirements

- The standard Rust tooling (cargo, rustup) which you can install from https://rustup.rs/

- Toolchain support for the cortex-m0+ processors in the rp2040 (thumbv6m-none-eabi)

- flip-link - this allows you to detect stack-overflows on the first core, which is the only supported target for now.

## Dependencies

```sh
rustup target install thumbv6m-none-eabi
cargo install flip-link
# This is our suggested default 'runner'
cargo install probe-run
# If you want to use elf2uf2-rs instead of probe-run, instead do...
cargo install elf2uf2-rs --locked
```
