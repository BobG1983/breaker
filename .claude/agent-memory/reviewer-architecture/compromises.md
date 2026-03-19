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
- ui/messages.rs imports chips::ChipKind (vocabulary type in message payload)
- bolt/spawn_additional_bolt reads breaker Transform (read-only, same pattern as spawn_bolt)
- Other domains attach interpolate components at spawn (opt-in cross-domain composition)
- behaviors/init.rs writes ResMut<BreakerConfig> and inserts breaker-owned components at init time — accepted for archetype config composition
- behaviors/plugin.rs orders against BreakerSystems::InitParams and UiSystems::SpawnTimerHud
- behaviors/consequences/life_lost.rs reads ui::StatusPanel (read-only, for HUD parenting)
- **Debug domain cross-domain exception**: debug/ is the ONLY domain permitted to read AND write other domains' resources and components directly. All gated behind `#[cfg(feature = "dev")]`. Does NOT set precedent for production domains.
- **Scenario runner cross-crate exception**: breaker-scenario-runner reads entity components from bolt, breaker, input, run domains directly. Four domain modules widened to `pub mod` in lib.rs. Dev-only crate, never shipped.

## Resolved Compromises (2026-03-16)
- ~~bolt/apply_bump_velocity reads breaker entity components~~ → multiplier now included in BumpPerformed message
- ~~physics/ccd.rs exists outside canonical layout~~ → moved to shared/math.rs
- ~~run/node/ lacks its own plugin.rs~~ → NodePlugin extracted
- ~~handle_life_lost writes ResMut<RunState>~~ → sends RunLost message instead
- ~~UI domain owns animate_fade_out~~ → moved to new fx domain
- ~~behaviors/plugin.rs uses bare fn refs for cross-domain OnEnter ordering~~ → BreakerSystems::InitParams + UiSystems::SpawnTimerHud extracted
