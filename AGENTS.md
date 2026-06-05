# AGENTS.md — jdw-osc-lib

## Source Structure

```
src/
  lib.rs       # Module exports
  model.rs     # OscArgHandler, TaggedBundle, TimedOSCPacket
  osc_stack.rs # OSCStack polling framework
```

## Key Types

- `OscArgHandler` — trait with `get_float(i)`, `get_int(i)`, `get_string(i)`, `get_bigdecimal(i)` returning `Result<T, OscError>`
- `TaggedBundle` — wraps `rosc::OscBundle`, validates `/bundle_info` header presence
- `TimedOSCPacket` — `{ beat: BigDecimal, packet: OscPacket }`
- `OSCStack` — builder-pattern handler registration + blocking poll loop

## OSCStack API

```rust
OSCStack::new(port: u16)
  .on_message("/addr", |msg| -> Result<(), OscError> { })
  .on_tbundle("/addr", |tb| -> Result<(), OscError> { })
  .funnel_tbundle("/addr", |tb| -> Result<(), OscError> { })  // also catches sub-addresses
  .set_name("service-name")  // for logging
  .begin()                   // blocks forever
```

## Protocol Convention

- All JDW bundles carry a `/bundle_info` as their first message
- Timed packets use `BigDecimal` beats (not milliseconds) for beat-synchronous timing
- String arguments use `rosc::OscType::String`

## Test

No tests currently — this is a shared protocol library used across 3 services.
