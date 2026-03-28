---
name: BreakerDestroyedAt and RequestBreakerDestroyed impact map
description: Complete reference map for the two stub destruction messages in the breaker domain — all five reference sites are in one file (messages.rs) and one other file (plugin.rs). No consumers exist anywhere.
type: project
---

Both types are pure stubs labeled "FUTURE" in their comments. All fields are commented out.

## Definition site
- `breaker-game/src/breaker/messages.rs:67` — `RequestBreakerDestroyed` struct definition
- `breaker-game/src/breaker/messages.rs:77` — `BreakerDestroyedAt` struct definition

## Registration site
- `breaker-game/src/breaker/plugin.rs:31-38` — imported and registered via `add_message::<RequestBreakerDestroyed>()` and `add_message::<BreakerDestroyedAt>()` in `BreakerPlugin::build`

## Tests (in messages.rs)
- `breaker-game/src/breaker/messages.rs:116` — `request_breaker_destroyed_debug_format` — constructs `RequestBreakerDestroyed {}` and asserts debug string
- `breaker-game/src/breaker/messages.rs:122` — `breaker_destroyed_at_debug_format` — constructs `BreakerDestroyedAt {}` and asserts debug string

## Producers
None — no system sends either message.

## Consumers
None — no system reads either message.

## Scenarios / RON / Docs
None.

**Why:** Both were created as Wave 2a stubs for two-phase destruction (C7). The implementation was never wired in.

**How to apply:** To delete both types, remove the two struct definitions from `messages.rs`, remove the two tests immediately below them, remove the import + two `add_message` calls from `plugin.rs`. No other files are affected.
