# Domain Folder Layout

Every domain folder follows this canonical internal structure. See [plugins.md](plugins.md) for the plugin architecture and crate organization these folders live inside.

```
src/<domain>/
├── mod.rs           # Re-exports ONLY — pub mod declarations, pub use re-exports. No logic, no types.
├── plugin.rs        # The Plugin impl. Registers systems, messages, states. One per domain.
├── components.rs    # All #[derive(Component)] types for this domain.
├── messages.rs      # All #[derive(Message)] types for this domain.
├── resources.rs     # All #[derive(Resource)] types for this domain.
├── sets.rs          # SystemSet enums for cross-domain ordering (optional).
├── definition.rs    # RON-deserialized content types and observer dispatch events (optional).
├── queries.rs       # All Query type aliases (optional — for clippy type_complexity).
├── filters.rs       # All Query filter type aliases (optional — for clippy type_complexity).
└── systems/
    ├── mod.rs       # Re-exports ONLY — pub mod + pub use for each system.
    └── <name>.rs    # One file per system function (or tightly related group).
```

**Rules:**
- **`mod.rs`** is a routing file. It contains `pub mod` and `pub use` statements only. No `fn`, `struct`, `enum`, or `impl`.
- **`plugin.rs`** is the only file that wires things to the Bevy `App` — system registration, message registration, state registration all happen here.
- **`components.rs`**, **`messages.rs`**, **`resources.rs`** — one file each per category. Omit the file if the domain has none of that category (e.g., no `messages.rs` if the domain sends no messages).
- **`sets.rs`** — optional file for `#[derive(SystemSet)]` enums that the domain exports for cross-domain ordering. Omit if the domain has no ordering points that other domains depend on. `mod.rs` must NOT contain type definitions — SystemSet enums go here, not in `mod.rs`.
- **`definition.rs`** — optional file for RON-deserialized content data types and observer dispatch events. Use it for: `#[derive(Asset, TypePath, Deserialize)]` content types loaded from RON (e.g., `ChipDefinition`, `ChipTemplate`, `CellTypeDefinition`); content enums like `TriggerChain`, `Target`, `ImpactTarget`, `Rarity`; `#[derive(Event)]` types used with `commands.trigger()` for observer dispatch. Do NOT put in `definition.rs`: `#[derive(Component)]` types (go in `components.rs`), `#[derive(Resource)]` types (go in `resources.rs`), `#[derive(Message)]` types (go in `messages.rs`), config defaults, or registries.
- **`queries.rs`**, **`filters.rs`** — optional files for query and filter type aliases to satisfy clippy's `type_complexity` lint. Omit if not needed. **Naming convention:** `<Purpose><Query|Filter>[<Entity>]`. Include the entity suffix when the alias queries/filters entities from a *different* domain than where the alias is defined (e.g., `CollisionQueryBolt` in the physics domain). Omit the suffix when querying entities from the *same* domain (e.g., `DashQuery` in the breaker domain) — the module path provides context.
- **`systems/`** — one `.rs` file per system function, or per tightly-coupled group (e.g., a system + its helper). Files are named after the system. `systems/mod.rs` only re-exports. When a system file grows too large, it is split into a directory module — see [System File Split Convention](#system-file-split-convention) below.
- Any canonical file (e.g., `components.rs`) may be promoted to a **directory** with `mod.rs` + subfiles when the single file grows too large. The `mod.rs` follows the same routing-only rule.
- **Shared math modules** live in `shared/math.rs` when multiple domains need the same pure functions (e.g., `ray_vs_aabb` for CCD). These should contain only pure functions and data types — no systems, no Bevy resources.
- No `utils.rs`, `helpers.rs`, `common.rs`, or `types.rs`. If it doesn't fit the categories above, it probably belongs in an existing file or a different domain.

## System File Split Convention

When a system file grows too large — typically because of a large `#[cfg(test)]` block — it should be split into a directory module. The threshold is ~400 lines total, or ~800 lines of test code.

### Standard split (single `tests.rs`)

```
systems/my_system/
├── mod.rs      — routing only
├── system.rs   — all production code
└── tests.rs    — all test code
```

`mod.rs` contains only:

```rust
pub(crate) use system::*;

mod system;

#[cfg(test)]
mod tests;
```

### Large test suite split (800+ lines of tests)

When `tests.rs` would itself exceed ~800 lines, promote it to a sub-directory:

```
systems/my_system/
├── mod.rs        — routing only (same as above)
├── system.rs     — all production code
└── tests/
    ├── mod.rs    — sub-module declarations only
    ├── helpers.rs  — shared test builders and utilities (pub(super))
    ├── group_a.rs  — tests grouped by concern
    └── group_b.rs  — tests grouped by concern
```

`tests/mod.rs` contains only `mod` declarations — no test code directly.

### Rules

- `mod.rs` is routing-only: `pub(crate) use system::*;` + `#[cfg(test)] mod tests;`. No logic.
- The inner production file MUST NOT share the directory's name (avoids `clippy::module_inception`). Use `system.rs`, `types.rs`, `bridge.rs`, `checker.rs`, `data.rs`, `fns.rs`, or `core.rs` as appropriate.
- Test files use `use crate::...` absolute paths — they lose the `super::*` scope of the old single-file layout.
- Test files need explicit `use bevy::prelude::*;` — it is not re-exported through `mod.rs`.
- Items re-exported through `mod.rs` must be `pub` (not `pub(crate)` or `pub(super)`) inside the private inner module — the private `mod system;` declaration in `mod.rs` caps the effective visibility at the domain boundary regardless.
- Parent `mod.rs` files need no changes — Rust resolves `mod foo;` to `foo/mod.rs` transparently.

## Per-Effect Layout (Effect Domain)

The `effect/` domain evaluates `EffectNode` trees and dispatches leaf effects using a **per-effect file** layout instead of the canonical category-based layout. Each effect gets its own file containing the relevant typed events, observers, and helper systems. This keeps each effect self-contained and scales cleanly as new effects are added.

```
src/effect/
├── mod.rs                 # Re-exports + pub mod declarations
├── plugin.rs              # EffectPlugin — calls effects::register() and triggers::register()
├── sets.rs                # EffectSystems set (Bridge variant for cross-domain ordering)
├── commands.rs            # EffectCommandsExt trait (fire_effect, reverse_effect, transfer_effect)
├── core/                  # Core types (NOT a sub-domain — no plugin.rs)
│   ├── mod.rs             # Re-exports from types.rs
│   └── types.rs           # Trigger, ImpactTarget, Target, AttractionType, RootEffect,
│                          #   EffectNode, EffectKind, BoundEffects, StagedEffects
├── effects/               # Per-effect modules (NOT a sub-domain — no plugin.rs)
│   ├── mod.rs             # pub mod declarations + register() dispatcher
│   ├── speed_boost.rs     # ActiveSpeedBoosts, fire(), reverse(), register()
│   ├── damage_boost.rs    # fire(), reverse(), register()
│   ├── shockwave.rs       # fire(), reverse(), register()
│   ├── life_lost.rs       # fire(), reverse(), register()
│   ├── chain_bolt.rs      # fire(), reverse(), register()
│   ├── ramping_damage.rs  # fire(), reverse(), register()
│   ├── explode.rs         # fire(), reverse(), register()
│   ├── quick_stop.rs      # fire(), reverse(), register()
│   ├── tether_beam.rs     # fire(), reverse(), register()
│   └── ... (~24 total — one file per EffectKind variant)
└── triggers/              # Bridge systems (one file per trigger type — NOT a sub-domain)
    ├── mod.rs             # pub mod declarations + register() dispatcher
    ├── evaluate.rs        # Shared chain evaluation helpers
    ├── bump.rs            # Global: any successful bump
    ├── perfect_bump.rs    # Global: perfect bump
    ├── early_bump.rs      # Global: early bump
    ├── late_bump.rs       # Global: late bump
    ├── bump_whiff.rs      # Global: bump timing missed
    ├── no_bump.rs         # Global: bolt hit breaker with no bump input
    ├── bumped.rs          # Targeted on bolt: any successful bump
    ├── perfect_bumped.rs  # Targeted on bolt: perfect bump
    ├── early_bumped.rs    # Targeted on bolt: early bump
    ├── late_bumped.rs     # Targeted on bolt: late bump
    ├── impact.rs          # Global impact triggers
    ├── impacted.rs        # Targeted impacted triggers on both collision participants
    ├── bolt_lost.rs       # Global: bolt was lost
    ├── death.rs           # Global: something died; cell destroyed
    ├── died.rs            # Targeted: this entity died
    ├── node_start.rs      # Global: node started
    ├── node_end.rs        # Global: node ended
    ├── timer.rs           # TimeExpires ticker system
    └── until.rs           # Until desugaring system
```

**Rules:**
- One file per effect type. The file owns any active-state `Component`s, plus `fire()`, `reverse()`, and `register(app: &mut App)` free functions.
- `effects/` and `triggers/` and `core/` are **directory groupings**, not sub-domains — none have a `plugin.rs`. `EffectPlugin` registers all systems through `effects::register(app)` and `triggers::register(app)`.
- `core/types.rs` holds all shared data types: `Trigger`, `ImpactTarget`, `Target`, `AttractionType`, `RootEffect`, `EffectNode`, `EffectKind`, `BoundEffects`, `StagedEffects`. No observers, no systems.
- `commands.rs` holds `EffectCommandsExt` — the `Commands` extension trait for queuing fire/reverse/transfer operations.
- `BreakerDefinition` lives in `breaker/definition.rs`. `BreakerRegistry` lives in `breaker/registry.rs`.
- Adding a new leaf effect = new file in `effects/` + `mod.rs` entry + variant in `EffectKind` + `fire()`/`reverse()` arms in `EffectKind` match + `register()` call in `effects/mod.rs`.
- Adding a new trigger = new file in `triggers/` + `pub mod` in `triggers/mod.rs` + `register()` call in `triggers/mod.rs::register()`.
- Adding a new breaker = new RON file only (if using existing trigger fields and effects).
- This layout applies **only** to the `effect/` domain. Standard domains use the canonical category-based layout.
- `effect/` is a **top-level domain** registered directly in `game.rs`. It is not nested under `breaker/`.

## Nested Sub-Domains

A domain may contain **nested sub-domains** when a cohesive subset of functionality deserves its own plugin, components, and systems. Each sub-domain follows the same canonical layout as a top-level domain.

```
src/<domain>/
├── mod.rs             # Re-exports shared types + sub-domain modules
├── plugin.rs          # Parent plugin — adds sub-domain plugins via app.add_plugins()
├── components.rs      # Shared components used across sub-domains
├── systems/           # Shared systems
└── <group>/           # Grouping directory (e.g., archetypes/)
    ├── <sub>/         # Sub-domain — follows full canonical layout
    │   ├── mod.rs
    │   ├── plugin.rs  # Sub-domain plugin (added by parent)
    │   ├── components.rs
    │   └── systems/
    └── <sub>/
        └── ...
```

**Rules:**
- Sub-domains follow the **same canonical layout** as top-level domains (mod.rs routing-only, plugin.rs for registration, etc.).
- The **parent plugin adds child plugins** — `game.rs` only knows about top-level plugins.
- Sub-domains may import the **parent's shared components** (e.g., `crate::breaker::components::Breaker`). This is not a boundary violation — they are part of the same domain.
- Sub-domains communicate with **other domains** through messages, same as any domain. No special privileges.
- The grouping directory (e.g., `archetypes/`) is optional — sub-domains can live directly under the parent if there's no natural grouping. The grouping directory has a routing-only `mod.rs`.
- **Don't nest deeper than one level.** If a sub-domain needs its own sub-domains, the structure is too complex — reconsider the domain boundaries.
