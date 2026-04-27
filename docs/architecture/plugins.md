# Plugin Architecture

## Workspace Layout

```
brickbreaker/
├── breaker-game/             # Main game (binary + library)
├── rantzsoft_spatial2d/      # 2D spatial: Position2D, propagation, interpolation
├── rantzsoft_physics2d/      # 2D physics: Aabb2D, CollisionLayers, DistanceConstraint, quadtree, CCD
├── rantzsoft_defaults/       # GameConfig derive + RON loader + SeedableRegistry
├── rantzsoft_defaults_derive/ # Proc-macro for #[derive(GameConfig)]
├── rantzsoft_stateflow/      # State routing, screen transitions, lifecycle messages
├── rantzsoft_dmg/            # Damage/kill/heal pipeline: Hp, Dead, Invulnerable, DmgSystems
└── breaker-scenario-runner/  # Headless gameplay testing (dev-only)
```

**Naming.** Game-specific crates use `breaker-<name>`. Game-agnostic reusable crates use `rantzsoft_<name>` and contain ZERO game-specific code (see `.claude/rules/rantzsoft-crates.md`). They may be extracted to standalone repos when reuse is needed.

## Domain Layout (`breaker-game/src/`)

One Bevy plugin per **domain**, not per system function.

```
src/
├── lib.rs            # Library root — declares domain modules
├── main.rs           # Binary entry — calls lib
├── app.rs            # App constructor: DefaultPlugins + Game group
├── game.rs           # PluginGroup wiring all domain plugins
├── prelude/          # Cross-domain re-exports (no types)
├── shared/           # PlayfieldConfig, BaseWidth, BaseHeight, etc.
├── state/            # Lifecycle, routing, menus, pause, run/node, HUD
├── input/            # Keyboard → GameAction translation
├── breaker/          # Breaker mechanics, bump system
├── effect_v3/        # Effect tree dispatch (top-level domain)
├── bolt/             # Bolt physics, CCD, chain bolts
├── cells/            # Cell types, grid, destruction
├── walls/            # Wall builder, boundary entities
├── chips/            # Chip catalog + recipes + dispatch
├── mutators/         # Protocols (positive upgrades) + hazards (stacking challenges)
├── fx/               # Cross-cutting visuals
├── audio/            # Stub (Phase 6)
└── debug/            # Dev tooling (cfg = "dev")
```

`lib.rs` declares most modules `pub(crate)` to enforce plugin boundaries. The exceptions — `bolt`, `breaker`, `cells`, `chips`, `effect_v3`, `input`, `state`, `walls`, `debug` — are `pub` because `breaker-scenario-runner` needs cross-crate access for invariant checking and entity tagging.

## Plugin Philosophy

Each domain plugin is a `bevy::app::Plugin` that:

- Owns its components, resources, messages, and systems.
- **Writes** to other domains only through messages.
- **Reads** other domains' types freely — that's normal ECS, not a violation.
- May contain nested sub-domain plugins for cohesive subsets. Only top-level plugins go in `game.rs`.

If a domain exposes ordering anchors for cross-domain consumers, they live in a `pub enum {Domain}Systems` in `sets.rs`. See `ordering.md`.

### Read vs write boundaries

The architectural line is on **mutations**, not reads. The bolt collision system reads `Hp` from cells, `BaseWidth` from the breaker, `DamageBoostStack` on bolts — that's fine. It writes `DamageDealt<Cell>` (a message), not `Hp.current` (a component). Routing every read through messages would be paranoia, not architecture.

The `debug/` domain is the one accepted exception — it reads AND writes across domains because hot-reload, telemetry, and recording inherently cut across everything. All debug code is compiled out of release builds.

## Cross-Domain Write Exception Rubric

Sometimes a non-owning domain needs to mutate another domain's component. The default answer is "use a message"; exceptions exist when message indirection would cost more than it saves.

A write exception is acceptable only when ALL of these hold:

1. **The writing domain has unique contextual data** the owning domain does not. (Hazard knows the heal cap; bolt domain doesn't.)
2. **The mutation is a simple, scoped numeric/marker update** — not a state-machine transition.
3. **The write is monotonic OR idempotent OR run-condition-gated** so it can't fight other writers.
4. **A message-based design would require a new pipeline, enum variant, or consumer system that exists solely for this one path** — i.e. the indirection would not generalize.

Existing exceptions (search `cross-domain` in source comments at the call sites for the live list):
- `Velocity2D` / `Position2D` on bolts — written by `effect_v3`, `cells/magnetic`, and `mutators/protocols/afterimage` for steering and reflection.
- `PiercingRemaining` on bolts — written by `cells/armored` to debit armor cost.
- `Hp.max` on cells — lifted by `mutators/hazards/{volatility,momentum}` to enable the mechanic's required ceiling.
- `NodeSequence` / `NodeOutcome` resources — rewritten by `mutators/protocols/tier_regression` on `OnEnter(RunState::Node)`.

Adding a new exception requires a comment at the call site (in the writing system) explaining which rubric items justify it. Don't update this doc — the source comment is the canonical record.

## The Mutators Domain

`mutators/` consolidates protocols (positive selectable upgrades) and hazards (stacking challenges). Both share structural patterns and several mechanics participate in the shared `DmgSystems::MutateDamage` / `PostApplyDamage` chains. That cross-mechanic ordering can't live cleanly in any one mechanic's `wire(app)`, which is why `MutatorsPlugin::wire_damage_chain` is the single source of truth for chain assembly.

`MutatorsPlugin::build` calls three internal wiring steps:

1. `wire_protocols(app)` — protocol registry/messages + per-mechanic `protocols::wire(app)` fan-out.
2. `wire_hazards(app)` — hazard registry/messages + per-mechanic `hazards::wire(app)` fan-out.
3. `wire_damage_chain(app)` — central ordering for diffusion / tether / echo_strike chain participants.

Run-end cleanup is each mechanic's own responsibility (`OnExit(NodeState::Playing)` registered inside its `wire(app)`).

**Game-side `DamageDealt<T>` writers MUST be tagged `.in_set(DmgSystems::EmitDamage)`** unless they're transitively ordered there via `EffectV3Systems::Tick`. Tagging a tick-set system with `EmitDamage` directly creates a scheduling cycle.

How to add a new mutator: see `creating-a-mutator.md`.

## The Effect Domain

`effect_v3/` stores effect trees in `BoundEffects` / `StagedEffects` (per-entity components), walks them through per-node evaluators, and dispatches via the `Fireable` / `Reversible` traits. The dispatch shape is enum-as-data: each variant of `EffectType` wraps a per-effect config struct that carries the parameters and implements the trait.

`EffectV3Plugin::build` does four things:

1. Configures `EffectV3Systems` ordering (`Bridge → Tick → Conditions`).
2. Registers `evaluate_conditions` into `Conditions`.
3. Inserts `SpawnStampRegistry` and registers spawn-watcher systems.
4. Calls each trigger category's `register::register(app)` and each effect config's `Fireable::register(app)`.

Free functions in `effect_v3/dispatch/` (`fire_dispatch`, `reverse_dispatch`, etc.) match on the enum once and call `config.fire(...)`. The walker queues `FireEffectCommand` / `ReverseEffectCommand`; commands run inside `Command::apply` where `&mut World` is available.

The full architecture lives in `architecture/effects/` (structure, type system, walker, conditions, commands, dispatch). This file documents only the plugin-level shape.
