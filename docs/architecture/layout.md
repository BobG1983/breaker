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

The `effect_v3/` domain organizes its source by concern (`types/`, `walking/`, `dispatch/`, `commands/`, `storage/`, `conditions/`, `triggers/`, `effects/`) rather than the canonical category-based layout. Each effect lives in its own subdirectory under `effects/` containing a config struct, optional components, optional runtime systems, and trait implementations (`Fireable` and optionally `Reversible`).

```
src/effect_v3/
├── mod.rs                 # Re-exports (EffectV3Plugin, EffectV3Systems)
├── plugin.rs              # EffectV3Plugin — configures sets, registers all 30 effect
│                          #   configs via Fireable::register, registers all trigger
│                          #   categories via triggers::*::register::register
├── sets.rs                # EffectV3Systems { Bridge, Tick, Conditions, Reset }
├── types/                 # Flat directory of one type per file
│   ├── mod.rs             # pub use of all types
│   ├── effect_type.rs     # EffectType enum (30 variants, all wrap configs)
│   ├── reversible_effect_type.rs  # ReversibleEffectType (16-variant subset) + From/TryFrom
│   ├── tree.rs            # Tree enum (Fire, When, Once, During, Until, Sequence, On)
│   ├── scoped_tree.rs     # ScopedTree (restricted Tree for During/Until)
│   ├── terminal.rs        # Terminal (leaf for Sequence/On)
│   ├── scoped_terminal.rs # ScopedTerminal (restricted leaf)
│   ├── root_node.rs       # RootNode { Stamp(StampTarget, Tree), Spawn(EntityKind, Tree) }
│   ├── stamp_target.rs    # StampTarget enum (Bolt, Breaker, Active*, Every*, ...)
│   ├── trigger.rs         # Trigger enum (uses OrderedFloat<f32> for Hash)
│   ├── condition.rs       # Condition { NodeActive, ShieldActive, ComboActive(u32) }
│   ├── entity_kind.rs     # EntityKind { Cell, Bolt, Wall, Breaker, Any }
│   ├── participants.rs    # BumpTarget/ImpactTarget/DeathTarget/BoltLostTarget + ParticipantTarget
│   ├── trigger_context.rs # TriggerContext (Bump/Impact/Death/BoltLost/None)
│   ├── route_type.rs      # RouteType { Bound, Staged }
│   └── ...                # bump_status.rs, attraction_type.rs
├── traits/                # Fireable + Reversible + PassiveEffect traits
├── commands/              # EffectCommandsExt trait + concrete Command structs
│   ├── ext/system.rs      # EffectCommandsExt impl on Commands
│   ├── fire.rs            # FireEffectCommand (calls fire_dispatch)
│   ├── reverse.rs         # ReverseEffectCommand (calls reverse_dispatch)
│   ├── route.rs / stamp.rs / stage.rs / remove.rs / remove_staged.rs / track_armed_fire.rs
├── dispatch/              # fire_dispatch, reverse_dispatch, fire_reversible_dispatch,
│                          #   reverse_all_by_source_dispatch
├── storage/               # BoundEffects, StagedEffects, ArmedFiredParticipants components +
│                          #   SpawnStampRegistry resource and per-kind watchers
├── walking/               # walk_bound_effects, walk_staged_effects, evaluate_tree, plus
│                          #   per-node evaluators (when, once, during, until, on, fire, sequence)
├── conditions/            # is_node_active, is_shield_active, is_combo_active, evaluate_conditions
├── triggers/              # Bridge systems grouped by trigger category (NOT a sub-domain)
│   ├── bump/              # bump trigger bridges (on_bumped, on_perfect_bumped, ...)
│   ├── impact/            # impact trigger bridges
│   ├── death/             # death trigger bridges (on_destroyed::<T> generic)
│   ├── bolt_lost/         # bolt-lost trigger bridges
│   ├── node/              # node lifecycle bridges + check_node_timer_thresholds
│   └── time/              # tick_effect_timers + on_time_expires
├── effects/               # One subdirectory per effect (30 total)
│   ├── mod.rs             # pub use of all *Config types
│   ├── speed_boost/{mod.rs, config.rs}    # SpeedBoostConfig + impl Fireable + impl Reversible
│   ├── shockwave/{mod.rs, config.rs, components.rs, systems/}
│   ├── chain_lightning/   # config + components + systems with tests
│   └── ...                # 30 modules total — see structure.md for the per-effect shapes
└── stacking/              # EffectStack<C> generic stack component for stack-based passives
```

**Rules:**
- One module per effect type. The module owns its config struct, any per-effect components, runtime systems (if any), and `impl Fireable for Config` / `impl Reversible for Config` (if reversible).
- `effects/`, `triggers/`, `walking/`, `conditions/`, `commands/`, `storage/`, and `types/` are **directory groupings**, not sub-domains — none have a `plugin.rs`. `EffectV3Plugin` registers all systems via `Fireable::register(app)` for each config and a `register(app)` call per trigger category.
- The dispatch layer is **free functions** (`fire_dispatch`, `reverse_dispatch`, etc.) that match on `EffectType` / `ReversibleEffectType` and call the relevant `config.fire(...)` / `config.reverse(...)`. There is no `EffectKind` enum holding methods — the enum is `EffectType` and the per-effect config is the implementation.
- `commands/ext/system.rs` holds `EffectCommandsExt` — the `Commands` extension trait with eight methods (`fire_effect`, `reverse_effect`, `route_effect`, `stamp_effect`, `stage_effect`, `remove_effect`, `remove_staged_effect`, `track_armed_fire`).
- `BreakerDefinition` lives in `breaker/definition.rs`. `BreakerRegistry` lives in `breaker/registry.rs`.
- Adding a new effect = new directory in `effects/` + `EffectType` variant + `fire_dispatch` arm + (optional) `ReversibleEffectType` variant + reverse-dispatch arms + `MyConfig::register(app)` call in `plugin.rs`. See `architecture/effects/adding_effects.md`.
- Adding a new trigger = new variant in `Trigger` + bridge function + `register()` call. See `architecture/effects/adding_triggers.md`.
- Adding a new breaker = new RON file only (if using existing trigger fields and effects).
- This layout applies **only** to the `effect_v3/` domain. Standard domains use the canonical category-based layout.
- `effect_v3/` is a **top-level domain** registered directly in `game.rs`. It is not nested under `breaker/`.

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
