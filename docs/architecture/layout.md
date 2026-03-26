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
- **`systems/`** — one `.rs` file per system function, or per tightly-coupled group (e.g., a system + its helper). Files are named after the system. `systems/mod.rs` only re-exports.
- Any canonical file (e.g., `components.rs`) may be promoted to a **directory** with `mod.rs` + subfiles when the single file grows too large. The `mod.rs` follows the same routing-only rule.
- **Shared math modules** live in `shared/math.rs` when multiple domains need the same pure functions (e.g., `ray_vs_aabb` for CCD). These should contain only pure functions and data types — no systems, no Bevy resources.
- No `utils.rs`, `helpers.rs`, `common.rs`, or `types.rs`. If it doesn't fit the categories above, it probably belongs in an existing file or a different domain.

## Per-Effect Layout (Effect Domain)

The `effect/` domain evaluates `EffectNode` trees and dispatches leaf effects using a **per-effect file** layout instead of the canonical category-based layout. Each effect gets its own file containing the relevant typed events, observers, and helper systems. This keeps each effect self-contained and scales cleanly as new effects are added.

```
src/effect/
├── mod.rs                 # Re-exports + pub mod declarations
├── plugin.rs              # EffectPlugin — calls each effect's register(), wires bridges and Until timers
├── sets.rs                # EffectSystems set (Bridge variant for cross-domain ordering)
├── definition.rs          # EffectNode, Trigger, Effect, EffectTarget, EffectChains, Target, ImpactTarget
├── evaluate.rs            # evaluate_node() pure function + NodeEvalResult enum
├── active.rs              # ActiveEffects resource (Vec<(Option<String>, EffectNode)>)
├── armed.rs               # ArmedEffects component (Vec<(Option<String>, EffectNode)> on bolt entities)
├── typed_events.rs        # Re-exports all typed events; fire_typed_event / fire_passive_event dispatch
├── helpers.rs             # Shared bridge helpers (evaluate_active_chains, evaluate_entity_chains, arm_bolt)
├── registry.rs            # BreakerRegistry re-export (canonical: breaker/registry.rs)
├── effect_nodes/          # EffectNode tree logic (NOT a sub-domain — no plugin.rs)
│   └── until.rs           # UntilTimers, UntilTriggers, tick_until_timers, check_until_triggers
├── effects/               # Per-effect handlers (NOT a sub-domain — no plugin.rs)
│   ├── mod.rs             # Routing only + shared stack_u32/stack_f32 helpers
│   ├── <effect_a>.rs      # Typed event + observer + systems + register() for effect A
│   └── <effect_b>.rs      # Typed event + observer + register() for effect B
└── triggers/              # Bridge systems (one file per trigger group — NOT a sub-domain)
    ├── mod.rs             # Routing + re-exports
    ├── on_bolt_lost.rs    # bridge_bolt_lost
    ├── on_bump.rs         # bridge_bump, bridge_bump_whiff
    ├── on_no_bump.rs      # bridge_no_bump
    ├── on_impact.rs       # bridge_cell_impact, bridge_wall_impact, bridge_breaker_impact
    ├── on_death.rs        # bridge_cell_death, bridge_bolt_death, cleanup systems, apply_once_nodes
    └── on_timer.rs        # bridge_timer_threshold
```

**Rules:**
- One file per effect type. The file owns any typed events, `Component`s, and observers the effect needs, plus a `register(app: &mut App)` function.
- `effects/` and `triggers/` and `effect_nodes/` are **directory groupings**, not sub-domains — none have a `plugin.rs`. `EffectPlugin` registers all observers, bridges, and systems.
- `definition.rs` holds the `EffectNode` enum (`When`, `Do`, `Until`, `Once`), the `Trigger` enum, the `Effect` enum, `EffectChains` component, `EffectEntity` marker, and `EffectTarget` runtime enum.
- `evaluate.rs` holds the pure `evaluate_node(Trigger, &EffectNode) -> Vec<NodeEvalResult>` function. Bridge systems call this to determine whether a chain fires, arms, or does not match.
- `active.rs` holds `ActiveEffects` — the global resource holding all breaker-definition and triggered-chip chains. Bridge helpers sweep it for global triggers.
- `armed.rs` holds `ArmedEffects` — a component on bolt entities holding partially-resolved chains. Subsequent triggers re-evaluate these via bridge helpers.
- `typed_events.rs` holds `fire_typed_event` (triggered effects) and `fire_passive_event` (passive/OnSelected effects) dispatch helpers. Each per-effect typed event struct lives in its effect file and is re-exported from `typed_events.rs`.
- `helpers.rs` holds shared bridge helpers consumed by multiple trigger files.
- `BreakerDefinition` lives in `breaker/definition.rs`. `BreakerRegistry` lives in `breaker/registry.rs`. Both are re-exported from `effect/` for historical reasons (`init_breaker` moved to `breaker/systems/init_breaker.rs`).
- Adding a new leaf effect = new file in `effects/` + `mod.rs` entry + `Effect` variant in `definition.rs` + `register()` call in `plugin.rs`.
- Adding a new trigger = new (or updated) file in `triggers/` + bridge system registered in `plugin.rs`.
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
