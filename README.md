# jdw-osc-lib

Shared Rust library for OSC (Open Sound Control) handling across the JackDAW project. Provides typed models, safe argument extraction, and a polling framework for building OSC-driven services.

## Features

- **`OscArgHandler`** — trait for safely extracting typed OSC arguments (strings, floats, ints, BigDecimals) from message bundles
- **`TaggedBundle`** — OSC bundle wrapper that expects a `/bundle_info` header in every bundle
- **`TimedOSCPacket`** — time-stamped packet with beat timing for sequencer use
- **`OSCStack`** — declarative polling framework with chained handlers: `.on_message()`, `.on_tbundle()`, `.funnel_tbundle()`, and a blocking `.begin()` loop

## Usage

All JackDAW Rust services (`jdw-sc`, `jdw-sequencer`, `jdw-keys-backend`) depend on this crate.

```rust
use jdw_osc_lib::{OSCStack, OscArgHandler};

let mut stack = OSCStack::new(port);
stack.on_message("/some/address", |msg| {
    let val: f32 = msg.args.get_float(0)?;
    Ok(())
});
stack.begin();
```

## Dependencies

- `rosc` — low-level OSC encode/decode
- `bigdecimal` — arbitrary precision decimals for musical timing
- `log` — logging facade
