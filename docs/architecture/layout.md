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

## Per-Effect Layout (Behaviors Domain)

The `behaviors/` domain evaluates unified `TriggerChain` trees and dispatches leaf effects using a **per-effect file** layout instead of the canonical category-based layout. Each effect gets its own file containing the relevant components, observer, and helper systems. This keeps each effect self-contained and scales cleanly as new effects are added.

```
src/behaviors/
├── mod.rs                 # Re-exports + pub mod declarations
├── plugin.rs              # BehaviorsPlugin — wires init, bridges, observers
├── sets.rs                # BehaviorSystems set (Bridge variant for cross-domain ordering)
├── definition.rs          # Asset type (ArchetypeDefinition with root trigger fields + chains vec)
├── registry.rs            # Registry resource (name → definition lookup)
├── active.rs              # ActiveChains resource (Vec<TriggerChain> populated at run start and by overclocks)
├── armed.rs               # ArmedTriggers component (partially-resolved chains attached to a bolt entity)
├── evaluate.rs            # TriggerKind enum + evaluate() pure function for chain resolution
├── events.rs              # EffectFired { effect: TriggerChain, bolt: Option<Entity> } observer event
├── init.rs                # Init systems (config overrides, component stamping)
├── bridges.rs             # Per-trigger bridge systems (message → EffectFired event or ArmedTriggers)
└── effects/               # Per-effect handlers (NOT a sub-domain — no plugin.rs)
    ├── mod.rs             # Routing only
    ├── <effect_a>.rs      # Components + observer + HUD for effect A
    └── <effect_b>.rs      # Observer handler for effect B
```

**Rules:**
- One file per effect type. The file owns any `Component`s the effect needs and the observer that handles `EffectFired`.
- `effects/` is a **directory grouping**, not a sub-domain — it has no `plugin.rs`. `BehaviorsPlugin` registers all observers and systems from effect files.
- `definition.rs` holds the RON-deserialized `ArchetypeDefinition` asset type. `ArchetypeDefinition` has named root trigger fields (`on_bolt_lost`, `on_perfect_bump`, `on_early_bump`, `on_late_bump`) plus a `chains: Vec<TriggerChain>` for additional multi-step chains. These are content data types, not Bevy components or resources.
- `events.rs` holds the `EffectFired` observer event. Bridge systems fire it via `commands.trigger()` when a `TriggerChain` resolves to a leaf. Each effect handler self-selects via pattern matching on `effect` — adding a new leaf effect never touches `bridges.rs`.
- `armed.rs` holds the `ArmedTriggers` component. When a bridge resolves a trigger node but the inner chain is not a leaf, it pushes the remaining chain onto the bolt's `ArmedTriggers` for evaluation on the next matching trigger event.
- `evaluate.rs` holds the pure `evaluate(TriggerKind, &TriggerChain) -> EvalResult` function and the `TriggerKind` enum. Bridge systems call this to determine whether a chain fires, arms, or does not match.
- `bridges.rs` holds per-trigger bridge systems. Each reads ONE message type, evaluates all active chains and any armed triggers, and fires `EffectFired` or updates `ArmedTriggers`. Adding a new trigger = new bridge system + `BehaviorSystems::Bridge` entry in `plugin.rs`.
- `init.rs` holds systems that run at archetype init time (config overrides, component stamping). `init_archetype` populates `ActiveChains` from the archetype definition's root trigger fields and `chains` vec.
- Adding a new leaf effect = new file in `effects/` + `mod.rs` entry + `TriggerChain` leaf variant in `chips/definition.rs` + observer registered in `plugin.rs`.
- Adding a new archetype = new RON file only (if using existing trigger fields and effects).
- This layout applies **only** to the `behaviors/` domain. Standard domains use the canonical category-based layout.
- `behaviors/` is a **top-level domain** registered directly in `game.rs`. It is not nested under `breaker/`.

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
