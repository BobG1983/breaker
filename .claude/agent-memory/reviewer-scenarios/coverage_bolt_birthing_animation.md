---
name: Bolt Birthing Animation Coverage Map
description: Coverage gaps for the bolt-birthing-animation branch — Birthing component, tick_birthing, begin_node_birthing, AnimateIn gating, bolt-lost respawn birthing, quit teardown chain
type: project
---

# Bolt Birthing Animation Coverage Map

## What the Feature Adds

1. `Birthing` component — zeroes `CollisionLayers` and `Scale2D` while animating, restores on completion. Lives in `shared/birthing.rs`.
2. `tick_birthing` system — lerps `Scale2D` from zero to `target_scale` over 0.3s, restores layers, removes `Birthing`.
3. `begin_node_birthing` system — `OnEnter(NodeState::AnimateIn)` — inserts `Birthing` on all bolts without it.
4. `all_animate_in_complete` system — gates `AnimateIn → Playing` transition on zero remaining `Birthing` entities.
5. `Bolt::builder().birthed()` — effect spawn sites (SpawnBolts, MirrorProtocol, SpawnPhantom, ChainBolt, TetherBeam) all use this.
6. `ActiveFilter` / `LaunchFilter` — both exclude `Birthing` bolts.
7. Bolt-lost respawn path — inserts `Birthing` on the respawned primary bolt.
8. Quit teardown chain — AnimateIn is now message-triggered (not pass-through), so paused virtual time blocks the chain (Bug 3, pending fix).

## Unit Test Coverage (strong)

- `tick_birthing`: 9 tests covering lerp progress, completion snap, layer restore, multi-entity, non-square scale.
- `begin_node_birthing`: 6 tests covering bulk insert, skip-if-already-birthing, bolt-only targeting, PreviousScale zeroing.
- `all_animate_in_complete`: 6 tests covering send/no-send conditions, full AnimateIn lifecycle integration.
- `Birthing` component: 3 tests covering construction, defaults, duration constant.
- `ActiveFilter` / `LaunchFilter`: 4 tests.
- Builder `.birthed()`: 8 tests covering all typestates, roles, motion modes, collision layers, target scale.
- Bolt-lost respawn with Birthing: 1 test (`bolt_lost_respawn_inserts_birthing_component`).
- Quit teardown chain: 1 integration test (`quit_teardown_chain_reaches_app_teardown`).

## Scenario Coverage: NONE specifically for birthing

No existing scenario verifies any of these birthing-specific properties:

1. **Birthing gate**: Bolt cannot collide during the 0.3s window — no scenario checks this.
2. **AnimateIn duration**: The node does not enter Playing until all birthing animations complete — no scenario verifies the gating.
3. **Bolt-lost respawn birthing**: After a bolt is lost, the respawned bolt also has a birthing window before becoming active — no scenario covers this.
4. **Effect-spawned bolts birthing**: MirrorProtocol, SpawnBolts, etc. spawn bolts mid-play with `.birthed()` — no scenario verifies these are inactive during their birthing window.
5. **Multi-node birthing repetition**: Every node entry re-triggers `begin_node_birthing` — no scenario specifically validates this on 2nd, 3rd node transitions.

## Implicit Coverage (existing scenarios run through AnimateIn)

ALL existing scenarios go through `NodeState::AnimateIn` on startup. Because `begin_node_birthing` runs on `OnEnter(NodeState::AnimateIn)`, every scenario exercises it. However:
- None list `BoltInBounds` during the AnimateIn phase with intent to catch birthing-window collisions.
- None verify that `Playing` doesn't start before birthing completes.
- The implicit exercise only proves "it doesn't crash during birthing" — not "birthing correctly gates collision."

## Key Invariant Gap: No BoltBirthingInactive invariant

There is no invariant that checks: "A bolt with `Birthing` has zeroed `CollisionLayers`."

If `tick_birthing` were to restore layers prematurely (e.g. a bug where `just_finished()` fires one tick early), no existing scenario would catch it. `BoltInBounds` and `NoNaN` would not fire — the bolt simply becomes collidable one frame early.

## New MutationKind Needed for Self-Test

There is no `MutationKind::InjectBirthingOnBolt` to insert `Birthing` on the tagged bolt mid-run. The closest existing mutations are `MoveBolt` and `InjectMismatchedBoltAabb`. A self-test for `BoltBirthingInactive` would need either:
(a) A new `MutationKind::RestoreBirthingLayersPrematurely` that sets the bolt's CollisionLayers to BOLT_LAYER while Birthing is still present, or
(b) A scenario that verifies the bolt doesn't collide in the first 0.3s of node start (observational, not injected).

## Bug 3 Implication (paused virtual time blocks quit chain)

Bug 3 (Quit from main menu hangs) is caused by virtual time being paused after a FadeOut transition. Because AnimateIn is now message-triggered (not pass-through), `check_spawn_complete` and `all_animate_in_complete` never fire on a re-entered run. This is a cross-feature interaction between birthing's AnimateIn gating and the stateflow crate's `unpause_at_end` behavior. No scenario exercises this path (blocked by ButtonInput<KeyCode> not being injectable — same limitation as aegis_pause_stress).

## Recorded: 2026-04-08 during feature/bolt-birthing-animation audit
