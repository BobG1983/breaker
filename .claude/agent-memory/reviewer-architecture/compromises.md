---
name: Architectural Compromises
description: Accepted and resolved architectural compromises with rationale
type: reference
---

## Accepted Architectural Compromises
- Physics domain mutates bolt Transform + BoltVelocity for collision response (minimum necessary)
- Screen domain seeds ALL domain configs during loading (centralized boot sequence)
- bolt/hover_bolt reads breaker Transform (read-only cross-domain query)
- bolt/spawn_bolt reads BreakerConfig and RunState (read-only, config access for spawn positioning)
- physics/bolt_lost reads breaker Transform (read-only, for respawn position)
- UI domain reads run::node::NodeTimer (read-only, for timer display)
- screen/run_end reads run::resources::RunState/RunOutcome (read-only)
- screen/run_setup reads behaviors::ArchetypeRegistry (read-only)
- screen/upgrade_select reads upgrades::UpgradeRegistry (read-only)
- All screen sub-domains read input::InputConfig (read-only, for key bindings)
- bolt/spawn_additional_bolt reads breaker Transform and ActiveNodeLayout (read-only, same pattern as spawn_bolt)
- breaker/apply_entity_scale_to_breaker reads run::node::ActiveNodeLayout (read-only, extracts entity_scale to stamp EntityScale component)
- bolt/apply_entity_scale_to_bolt reads run::node::ActiveNodeLayout (read-only, same pattern as breaker)
- Other domains attach interpolate components at spawn (opt-in cross-domain composition)
- behaviors/init.rs writes ResMut<BreakerConfig> and inserts breaker-owned components at init time — accepted for archetype config composition
- behaviors/plugin.rs orders against BreakerSystems::InitParams and UiSystems::SpawnTimerHud
- behaviors/consequences/life_lost.rs reads ui::StatusPanel (read-only, for HUD parenting)
- **Debug domain cross-domain exception**: debug/ is the ONLY domain permitted to read AND write other domains' resources and components directly. All gated behind `#[cfg(feature = "dev")]`. Does NOT set precedent for production domains.
- **Scenario runner cross-crate exception**: breaker-scenario-runner reads entity components from bolt, breaker, chips, input, run domains directly. Five domain modules widened to `pub mod` in lib.rs (`chips` added 2026-03-20 for `TriggerChain`/`ImpactTarget` in `initial_overclocks`). Dev-only crate, never shipped.
- **Chip effect cross-domain reads**: physics reads Piercing, PiercingRemaining (mut), DamageBoost from bolt; TiltControlBoost, WidthBoost from breaker. cells reads DamageBoost from bolt. breaker reads BreakerSpeedBoost, WidthBoost, BumpForceBoost from breaker entity (same entity). bolt reads BoltSpeedBoost (Amp chip component in chips/components.rs) from bolt entity. All justified per plugins.md "Chip Effect" section. PiercingRemaining mutation is collision-response (same class as BoltVelocity mutation).

## Active Violations (pending resolution)
(none)

## Resolved Compromises (2026-03-16)
- ~~bolt/apply_bump_velocity reads breaker entity components~~ → first resolved by including multiplier in BumpPerformed message (2026-03-16); then apply_bump_velocity itself DELETED in refactor/unify-behaviors (2026-03-21) — velocity scaling now via TriggerChain::SpeedBoost leaf
- ~~bolt/behaviors/effects/shockwave.rs cross-domain mutation~~ → FIXED 2026-03-20 (feature/overclock-trigger-chain): shockwave now writes `DamageCell` messages (consumer-owns pattern); cells/handle_cell_hit processes damage. No direct CellHealth mutation. NOTE: shockwave.rs now lives at behaviors/effects/shockwave.rs (bolt/behaviors/ deleted in refactor/unify-behaviors).
- ~~physics/ccd.rs exists outside canonical layout~~ → moved to shared/math.rs
- ~~run/node/ lacks its own plugin.rs~~ → NodePlugin extracted
- ~~handle_life_lost writes ResMut<RunState>~~ → sends RunLost message instead
- ~~UI domain owns animate_fade_out~~ → moved to new fx domain
- ~~behaviors/plugin.rs uses bare fn refs for cross-domain OnEnter ordering~~ → BreakerSystems::InitParams + UiSystems::SpawnTimerHud extracted
