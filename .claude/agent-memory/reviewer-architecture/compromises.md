---
name: Architectural Compromises
description: Accepted and resolved architectural compromises with rationale
type: reference
---

## Accepted Architectural Compromises
- Physics domain mutates bolt Position2D + BoltVelocity for collision response (minimum necessary). Previously was Transform; migrated to Position2D as canonical position (2026-03-23).
- Screen domain seeds ALL domain configs during loading (centralized boot sequence)
- bolt/hover_bolt reads breaker Position2D (read-only cross-domain query). Previously Transform; migrated 2026-03-23.
- bolt/spawn_bolt reads breaker Position2D, BreakerConfig and RunState (read-only, config access for spawn positioning). Previously Transform; migrated 2026-03-23.
- bolt/bolt_lost reads breaker Position2D (read-only, for respawn position). Previously Transform; migrated 2026-03-23. System moved from physics/ to bolt/ in 2026-03-24 spatial/physics extraction.
- UI domain reads run::node::NodeTimer (read-only, for timer display)
- screen/run_end reads run::resources::RunState/RunOutcome (read-only)
- screen/run_setup reads behaviors::ArchetypeRegistry (read-only)
- screen/chip_select reads chips::ChipRegistry (read-only)
- All screen sub-domains read input::InputConfig (read-only, for key bindings)
- bolt/spawn_additional_bolt reads breaker Position2D and ActiveNodeLayout (read-only, same pattern as spawn_bolt). Previously Transform; migrated 2026-03-23.
- breaker/apply_entity_scale_to_breaker reads run::node::ActiveNodeLayout (read-only, extracts entity_scale to stamp EntityScale component)
- bolt/apply_entity_scale_to_bolt reads run::node::ActiveNodeLayout (read-only, same pattern as breaker)
- ~~Other domains attach interpolate components at spawn (opt-in cross-domain composition)~~ → DELETED 2026-03-24: interpolate/ game domain and InterpolatePlugin fully removed from game.rs (spatial/physics extraction). Position2D migration complete — rantzsoft_spatial2d components (Spatial2D, InterpolateTransform2D, Position2D, PreviousPosition) now used. PhysicsTranslation and InterpolateTransform deleted.
- behaviors/init.rs writes ResMut<BreakerConfig> and inserts breaker-owned components at init time — accepted for archetype config composition
- behaviors/plugin.rs orders against BreakerSystems::InitParams and UiSystems::SpawnTimerHud
- behaviors/effects/life_lost.rs reads ui::StatusPanel (read-only, for HUD parenting)
- **Debug domain cross-domain exception**: debug/ is the ONLY domain permitted to read AND write other domains' resources and components directly. All gated behind `#[cfg(feature = "dev")]`. Does NOT set precedent for production domains.
- **Scenario runner cross-crate exception**: breaker-scenario-runner reads entity components from bolt, breaker, chips, input, run domains directly. Five domain modules widened to `pub mod` in lib.rs (`chips` added 2026-03-20 for `TriggerChain`/`ImpactTarget` in `initial_overclocks`). Dev-only crate, never shipped.
- **Chip effect cross-domain reads**: physics reads Piercing, PiercingRemaining (mut), DamageBoost from bolt; TiltControlBoost, WidthBoost from breaker. cells reads DamageBoost from bolt. breaker reads BreakerSpeedBoost, WidthBoost, BumpForceBoost from breaker entity (same entity). bolt reads BoltSpeedBoost (Amp chip component in chips/components.rs) from bolt entity. All justified per plugins.md "Chip Effect" section. PiercingRemaining mutation is collision-response (same class as BoltVelocity mutation).

- **run/reset_run_state mutates ChipInventory**: run domain clears chips/ domain's ChipInventory resource at run start. Same class as screen/loading seeding all domain configs — centralized boot/reset sequence. No alternative consumer-owns pattern makes sense for a cross-domain reset.

- **chips/apply_chip_effect writes ResMut<ActiveEffects>**: chips domain pushes triggered chains to effect domain's `ActiveEffects` resource. Same class as effect/init.rs writing ResMut<BreakerConfig>. Pre-existed B1-B3 refactor.

- **B12c dual type system**: effect/definition.rs defines Trigger/Effect/EffectNode parallel to chips/definition.rs TriggerChain. trigger_chain_to_effect() converts between them. Target and ImpactTarget duplicated in both domains. Transitional — resolves when TriggerChain is replaced by EffectNode as canonical runtime type. The conversion layer MUST NOT become permanent.

- **B12c typed_events.rs non-canonical file**: Centralized typed event definitions violate per-effect co-location rule. Planned dissolution: move events to co-located handler files, move dispatch functions to bridges.rs.

- **B12c passive events in wrong domain**: PiercingApplied, DamageBoostApplied, etc. defined in effect/typed_events.rs but conceptually belong to chips/ domain. Planned fix: move to chips/effects/<handler>.rs.

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
