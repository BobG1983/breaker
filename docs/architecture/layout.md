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
- **`queries.rs`**, **`filters.rs`** — optional files for query and filter type aliases to satisfy clippy's `type_complexity` lint. Omit if not needed.
- **`systems/`** — one `.rs` file per system function, or per tightly-coupled group (e.g., a system + its helper). Files are named after the system. `systems/mod.rs` only re-exports.
- Any canonical file (e.g., `components.rs`) may be promoted to a **directory** with `mod.rs` + subfiles when the single file grows too large. The `mod.rs` follows the same routing-only rule.
- A domain may have **shared math modules** (e.g., `physics/ccd.rs`) when multiple systems need the same pure functions. These should contain only pure functions and data types — no systems, no Bevy resources.
- No `utils.rs`, `helpers.rs`, `common.rs`, or `types.rs`. If it doesn't fit the categories above, it probably belongs in an existing file or a different domain.

## Per-Consequence Layout (Behavior Sub-Domains)

Behavior sub-domains that dispatch from triggers to consequences use a **per-consequence file** layout instead of the canonical category-based layout. Each consequence gets its own file containing the event type, related components, observer, and helper systems for that consequence. This keeps each consequence self-contained and scales cleanly as new consequences are added.

```
src/<domain>/behaviors/
├── mod.rs                 # Re-exports + pub mod declarations
├── plugin.rs              # BehaviorPlugin — wires init, bridges, observers
├── definition.rs          # Asset type, trigger/consequence enums, stat overrides
├── registry.rs            # Registry resource (name → definition lookup)
├── active.rs              # ActiveBehaviors resource (trigger → consequence runtime lookup)
├── init.rs                # Init systems (config overrides, component stamping)
├── bridges.rs             # Per-trigger bridge systems (message → consequence event)
└── consequences/          # Per-consequence handlers (NOT a sub-domain — no plugin.rs)
    ├── mod.rs             # Routing only
    ├── <consequence_a>.rs # Event + components + observer + HUD for consequence A
    └── <consequence_b>.rs # Init-time apply function for consequence B
```

**Rules:**
- One file per consequence type. The file owns the consequence's `Event`, any `Component`s it needs, and the observer or helper that handles it.
- `consequences/` is a **directory grouping**, not a sub-domain — it has no `plugin.rs`. The parent `BehaviorPlugin` registers all observers and systems from consequence files.
- `definition.rs` holds the RON-deserialized data types (`Asset`, trigger/consequence enums). These are content data types, not Bevy components or resources.
- `bridges.rs` holds per-trigger bridge systems. Each reads ONE message type and fires consequence events via `commands.trigger()`.
- `init.rs` holds systems that run at archetype init time (config overrides, component stamping).
- Adding a new consequence = new file in `consequences/` + `mod.rs` entry + match arm in `bridges.rs` + `Consequence` enum variant in `definition.rs`.
- Adding a new archetype = new RON file only (if using existing triggers/consequences).
- This layout applies **only** to behavior sub-domains that use the trigger→consequence dispatch pattern. Standard sub-domains use the canonical category-based layout.

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
