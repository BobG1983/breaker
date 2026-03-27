---
name: Enum dispatch over trait objects for effect system
description: Confirmed decision — use plain enum variants (not Box<dyn Trait>) for Effect and all discriminated types in the effect domain. typetag/dyn-clone pattern evaluated and rejected.
type: project
---

# Enum Dispatch Over Trait Objects — Effect System

## Decision

Use plain `enum Effect` with `#[derive(Deserialize, Clone, Debug, PartialEq)]` for all effect variants. Do NOT introduce `Box<dyn Effect>` with typetag/dyn-clone.

**Why:**
- `Effect` is a closed set (all variants known at compile time, defined by game design). Closed sets = enum, not trait objects.
- `typetag` generates impls for `dyn MyTrait`, not `dyn MyTrait + Send + Sync`. Bevy requires `Send + Sync` on resources/components. This is a load-bearing incompatibility with no clean workaround.
- `typetag` uses `inventory`/`ctor` (static constructors before `main`) — hidden complexity, WASM-incompatible.
- RON syntax for typetag requires `{"TypeName": {...}}` map-key tagging, which is noisier than the plain variant syntax the codebase already uses.
- `PartialEq` on `dyn Trait` requires manual `as_any()` + downcasting on every impl — 2 boilerplate methods × 25 variants vs. one `#[derive]`.
- `dyn-clone` correctly solves the Clone-for-trait-objects problem in isolation, but there is no benefit here since plain enums derive `Clone` for free.

**How to apply:**
- When a new effect type is needed: add a variant to `Effect` in `types.rs` and add a match arm in the appropriate handler system.
- If `Effect` grows unwieldy: split into sub-enums (`BoltEffect`, `BreakerEffect`, etc.) wrapped by a top-level `Effect` — still all plain enums, all RON-compatible.
- Do not reach for `Box<dyn Effect>` for extensibility; the set is closed and enum exhaustiveness checking is a feature, not a limitation.

## Codebase Evidence

- `breaker-game/src/effect/definition/types.rs:122` — `Effect` enum, 25 variants, all four derives.
- `breaker-game/src/effect/definition/types.rs:269` — `EffectNode` wraps `Effect` in `Do(Effect)`, proving trees work with plain enums.
- `breaker-game/src/breaker/definition.rs:14` — `BreakerDefinition` holds `Vec<RootEffect>`, loads from RON via `bevy_common_assets` with zero custom deserializer code.

## Rejected Alternatives

- `typetag` + `Box<dyn Effect>`: Send+Sync incompatibility, RON format friction, ctor complexity.
- `dyn-clone` alone: solves Clone but leaves deserialization unsolved; no benefit without typetag.
- `serde_flexitos`: Same ctor/WASM limitations as typetag; no advantage for a closed set.
- `enum_dispatch` crate: Useful when variants share a self-executing method (e.g., `fn fire(&self)`). This codebase dispatches via match in systems (heterogeneous resource access per variant) — enum_dispatch doesn't help that shape.
