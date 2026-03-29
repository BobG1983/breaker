---
name: Chain Lightning Rework — Intentional Patterns
description: Patterns from the chain_lightning rework (feature/runtime-effects) that look like violations but are intentional
type: project
---

## `ChainState` missing `#[derive(Debug)]` on the enum itself — intentional (struct literal in fields)

`ChainState` is used only internally within `ChainLightningChain`. The absence of `Debug` is not a compliance gap but the review flagged it as a missing derive. If `ChainLightningChain` gets `Debug` added, `ChainState` needs it too.

## `chain_lightning_test_app` vs `chain_lightning_damage_test_app` split

Two test app factories exist side-by-side in helpers.rs. The split is intentional: `chain_lightning_test_app` is minimal (no collector, no tick system) and used in fire_tests that drive the system directly via `&mut World`. `chain_lightning_damage_test_app` adds the tick system and collector for tick_tests. This is NOT duplication — the two have different lifecycles.

## `ChainLightningWorld` SystemParam name

`ChainLightningWorld` is the `SystemParam` struct that bundles world queries for `tick_chain_lightning`. The "World" suffix is unusual (most SystemParam bundles in this codebase use `Params` suffix), but it does not conflict with Bevy's `World` type because SystemParam is used in system signatures, not as a Bevy ECS type. Accept as intentional naming for this module.

## `hit_set: HashSet<Entity>` as public field on `ChainLightningChain`

All fields on `ChainLightningChain` are `pub`. This is consistent with other stateful chain components in this codebase (`TetherBeamComponent`, etc.) where test access requires pub fields. Not a visibility violation given the module's test coverage needs.
