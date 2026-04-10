# Wave 2: Scaffold — Files, Types, Stubs, Callsites, Wiring

## Goal
Create the entire new effect domain structure, populate all types, stub all functions and systems, migrate callsites, and wire plugins. At the end of this wave, the build compiles with everything stubbed.

## Steps (in order)

### 2a. Create folder structure and empty files
- Create every directory and `.rs` file from the folder structure doc
- Add `mod effect;` back to `src/lib.rs`
- All `mod.rs` files get `pub(crate) mod` declarations

### 2b. Write all types
- All enums, structs, traits, components, resources, messages (see wave 3 type list from previous version)
- All config structs with derives and OrderedFloat fields
- All trait impls with `todo!()` bodies
- Death pipeline types in `src/shared/` (GameEntity, Hp, KilledBy, Dead, DamageDealt, KillYourself, Destroyed, DespawnEntity)

### 2c. Migrate callsites
- Fix all `use crate::effect::*` imports to new module paths
- Replace old type names with new ones (see type swaps doc)
- Systems that called old effect functions get `todo!()` bodies

### 2d. Stub all functions
- Walking algorithm, dispatch, command extensions, condition evaluators, EffectStack methods — all `todo!()` bodies

### 2e. Stub all systems
- All bridge systems, game systems, tick systems, condition system, reset systems, death pipeline systems — empty bodies with `_` params

### 2f. Wire plugins
- EffectPlugin: system sets, Config::register(), trigger register(), evaluate_conditions, resets, resources
- DeathPipelinePlugin: system sets, apply_damage, detect_deaths, process_despawn_requests

## Verification
`cargo dcheck` passes. Use `#[expect(dead_code)]`, `#[expect(unused_imports)]`, `#[expect(clippy::todo)]` as needed.

## Docs to read
- `effect-refactor/migration/folder-structure.md` — target directory tree
- `effect-refactor/rust-types/` — all type definitions (enums, configs, components, resources, messages, traits)
- `effect-refactor/storing-effects/` — BoundEffects, StagedEffects, SpawnStampRegistry
- `effect-refactor/migration/rust-type-swaps.md` — old → new type mapping
- `effect-refactor/migration/new-dependencies.md` — ordered-float
- `effect-refactor/walking-effects/walking-algorithm.md` — walker signature
- `effect-refactor/command-extensions/` — all command extension structs
- `effect-refactor/creating-effects/effect-api/` — trait contracts
- `effect-refactor/evaluating-conditions/` — condition evaluator signatures
- `effect-refactor/migration/plugin-wiring/` — EffectPlugin registration
- `effect-refactor/migration/new-trigger-implementations/` — system signatures
- `effect-refactor/migration/new-effect-implementations/` — tick system signatures
- `unified-death-pipeline/rust-types/` — death pipeline types
- `unified-death-pipeline/migration/plugin-wiring/` — DeathPipelinePlugin
- `unified-death-pipeline/migration/systems-to-create/` — death system signatures
