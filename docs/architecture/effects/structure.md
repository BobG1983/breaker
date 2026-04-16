# Domain Structure

The effect system lives entirely in `breaker-game/src/effect_v3/`. The directory is organized by concern (types, walking, dispatch, storage, conditions, triggers, effects) rather than by entity type.

```
effect_v3/
  mod.rs                     — public re-exports (EffectV3Plugin, EffectV3Systems)
  plugin.rs                  — EffectV3Plugin: configures sets, registers all 30
                               effect configs via Fireable::register, registers all
                               trigger categories via triggers::*::register::register
  sets.rs                    — EffectV3Systems { Bridge, Tick, Conditions, Reset }

  types/                     — flat directory of one type per file
    mod.rs
    effect_type.rs           — EffectType enum (30 variants, all wrap configs)
    reversible_effect_type.rs — ReversibleEffectType (16-variant subset) + From/TryFrom
    tree.rs                  — Tree enum (Fire, When, Once, During, Until, Sequence, On)
    scoped_tree.rs           — ScopedTree (restricted Tree for During/Until)
    terminal.rs              — Terminal (leaf for Sequence/On)
    scoped_terminal.rs       — ScopedTerminal (restricted leaf)
    root_node.rs             — RootNode { Stamp(StampTarget, Tree), Spawn(EntityKind, Tree) }
    stamp_target.rs          — StampTarget enum (Bolt, Breaker, ActiveBolts, Every*, ...)
    trigger.rs               — Trigger enum (uses OrderedFloat<f32> for Hash)
    condition.rs             — Condition { NodeActive, ShieldActive, ComboActive(u32) }
    entity_kind.rs           — EntityKind { Cell, Bolt, Wall, Breaker, Any }
    participants.rs          — BumpTarget/ImpactTarget/DeathTarget/BoltLostTarget
                               + ParticipantTarget wrapper enum
    trigger_context.rs       — TriggerContext (Bump/Impact/Death/BoltLost/None)
    route_type.rs            — RouteType { Bound, Staged }
    bump_status.rs           — BumpStatus type
    attraction_type.rs       — AttractionType (used by AttractionConfig)

  traits/
    mod.rs
    fireable.rs              — Fireable trait (fire + register)
    reversible.rs            — Reversible trait (reverse + reverse_all_by_source)
    passive_effect.rs        — PassiveEffect helper trait

  commands/                  — EffectCommandsExt + concrete Command structs
    mod.rs                   — re-exports
    ext/system.rs            — EffectCommandsExt trait + impl on Commands
    ext/mod.rs
    fire.rs                  — FireEffectCommand (calls fire_dispatch)
    reverse.rs               — ReverseEffectCommand (calls reverse_dispatch)
    route.rs                 — RouteEffectCommand
    stamp.rs                 — StampEffectCommand (sugar for Route(Bound, ...))
    stage.rs                 — StageEffectCommand (sugar for Route(Staged, ...))
    remove.rs                — RemoveEffectCommand (name-sweep across both stores)
    remove_staged.rs         — RemoveStagedEffectCommand (entry-specific consume)
    track_armed_fire.rs      — TrackArmedFireCommand (Shape D bookkeeping)

  dispatch/
    mod.rs
    fire_dispatch.rs         — fire_dispatch: match EffectType -> config.fire()
    reverse_dispatch/
      mod.rs
      system.rs              — reverse_dispatch / fire_reversible_dispatch /
                               reverse_all_by_source_dispatch
      tests.rs

  storage/
    mod.rs
    bound_effects.rs         — BoundEffects(pub Vec<(String, Tree)>) component
    staged_effects.rs        — StagedEffects(pub Vec<(String, Tree)>) component
    armed_fired_participants.rs — ArmedFiredParticipants component (Shape D)
    spawn_stamp_registry/
      mod.rs
      resource.rs            — SpawnStampRegistry resource
      watchers/
        stamp_spawned_bolts.rs   — Added<Bolt> -> commands.stamp_effect
        stamp_spawned_cells.rs
        stamp_spawned_walls.rs
        stamp_spawned_breakers.rs

  walking/                   — tree walker entry points and per-node evaluators
    mod.rs
    walk_effects/
      mod.rs
      system.rs              — walk_bound_effects, walk_staged_effects, evaluate_tree
      tests.rs
    fire.rs                  — evaluate_fire (queues FireEffectCommand)
    sequence.rs              — evaluate_sequence + evaluate_terminal
    when/system.rs           — evaluate_when (gate + arm-on-nested-gate)
    once/system.rs           — evaluate_once (gate + remove-after-match)
    during/system.rs         — evaluate_during (queues DuringInstallCommand)
    until/system.rs          — evaluate_until + UntilApplied + UntilEvaluateCommand
    on/system.rs              — evaluate_on (resolve participant + track armed fires)

  conditions/
    mod.rs
    evaluate_conditions/
      mod.rs
      system.rs              — evaluate_conditions exclusive system + DuringActive
                               component + condition transition state machine
    node_active.rs           — is_node_active(world)
    shield_active.rs         — is_shield_active(world) — checks ShieldWall archetypes
    combo_active.rs          — is_combo_active(world, threshold)
    armed_source.rs          — is_armed_source(source) — armed-key naming convention
    node_active.rs

  triggers/                  — one subdirectory per trigger category
    mod.rs                   — pub mod bolt_lost; bump; death; impact; node; time
    bump/
      mod.rs
      register.rs            — register(app): adds all bridges to EffectV3Systems::Bridge
      bridges/system.rs      — on_bumped, on_perfect_bumped, on_bump_occurred, ...
    impact/  ...
    death/   ...
    bolt_lost/ ...
    node/    ...             — also owns NodeTimerThreshold scan + reset systems
    time/    ...             — also owns tick_timers + Time-based triggers

  effects/                   — one module per effect (30 total)
    mod.rs                   — pub use of all *Config types
    speed_boost/
      mod.rs                 — re-exports SpeedBoostConfig
      config.rs              — SpeedBoostConfig + impl Fireable + impl Reversible
    shockwave/
      mod.rs
      config.rs              — ShockwaveConfig + impl Fireable
      components.rs          — ShockwaveRing component
      systems/
        mod.rs
        system.rs            — tick_shockwave, expand_radius, apply_damage
        tests.rs
    chain_lightning/
      ...                    — same shape: config + components + systems
    ...

  stacking/
    mod.rs
    effect_stack/
      mod.rs
      component.rs           — EffectStack<C> generic component for stack-based passives
      tests/                 — aggregate, push_pop, multi_source, mod
```

## What Lives Where

**Effect domain** (`effect_v3/`) owns:
- All effect types and configs
- The walker (`walking/`)
- All dispatch (`dispatch/`)
- All trigger bridges (`triggers/`)
- All condition predicates and the condition poller
- All effect storage components (`BoundEffects`, `StagedEffects`, etc.)
- `EffectCommandsExt` and its concrete commands
- The `Fireable` / `Reversible` traits

**Outside the effect domain**:
- **Chip dispatch** — `chips/systems/dispatch_chip_effects/` reads `ChipSelected`, walks the chip's `effects: Vec<RootNode>`, calls `commands.stamp_effect(entity, name, tree)` for each non-Fire root. Bare `Tree::Fire` children fire immediately via `commands.fire_effect`. See `dispatch.md`.
- **Breaker dispatch** — same pattern in `breaker/`, reading from `BreakerDefinition`.
- **Cell dispatch** — same pattern in `cells/` (cells optionally carry effects).
- **Collision detection** — lives in entity domains (`bolt/`, `breaker/`, `cells/`, `walls/`) and emits `BoltImpactCell`, `BoltImpactWall`, etc. messages. The effect domain's bridge systems read those messages and translate them into `Trigger` dispatches.

## Per-Effect Module Shapes

Effect modules come in three shapes depending on complexity. All three implement `Fireable` (and `Reversible` if listed in `ReversibleEffectType`).

**Trivial** — single `config.rs` file holding the config struct and trait impls. Used by passive stat boosts (`speed_boost/`, `damage_boost/`, `bump_force/`).

**With runtime systems** — `config.rs` + `systems.rs` (or `systems/`) + `components.rs`. Used by effects that spawn entities or have per-tick logic (`pulse/`, `gravity_well/`, `attraction/`).

**With nested config tests** — `config/config_impl.rs` + `config/tests.rs` (or a `tests/` subdir). Used by complex effects whose config logic itself warrants unit tests (`shield/`, `chain_lightning/`, `tether_beam/`, `pulse/`, `second_wind/`, `ramping_damage/`, `piercing_beam/`).

The `Fireable::register` impl is the entry point for any per-effect plugin wiring — system registration, resource init, cleanup hooks.

## Why a Flat `types/` Directory

Each type file is short (15–80 lines) but conceptually self-contained. A flat directory means imports are stable (`use crate::effect_v3::types::Tree;`) and a reader looking for "where is `StampTarget` defined?" can find it without spelunking through nested modules. The trade-off is that `types/mod.rs` has a long `pub use` list, but that's a one-time cost.
