---
name: Project State
description: Bevy version, workspace structure, phase audit history, debug domain structure
type: reference
---

## Project Configuration
- Bevy 0.18.1, bevy_egui 0.39, edition 2024
- Workspace with three crates (game, derive, scenario-runner), plugin-per-domain, message-driven decoupling
- Also depends on: bevy_asset_loader 0.25, bevy_common_assets 0.15, iyes_progress 0.16
- Architecture docs in `docs/architecture/` (README, layout, messages, ordering, plugins, state, physics, content, standards, data)

## Audit History
- Phase 0 scaffolding complete, reviewed 2025-03-10
- Phase 1 core mechanics implemented, reviewed 2025-03-10
- Main menu screen implemented, reviewed 2026-03-11
- Full audit completed 2026-03-11: clean, no critical violations
- Post-Phase1 additions audit 2026-03-11: BoltServing, hover_bolt, launch_bolt, BumpVisual, RunState, cleanup_entities<T> — all clean
- Config-to-entity extraction refactor audited 2026-03-12: PASS, no violations
- Full doc-vs-code audit 2026-03-13: 22 mismatches found and FIXED in docs
- wall/ domain extracted from physics (2026-03-13 branch: refactor/extract-wall-domain)
- screen/ refactored into sub-domains (2026-03-13): loading/ and main_menu/ as direct children, PASS audit
- **Phase 2b audit 2026-03-13**: PASS with 1 structural issue (run/node/mod.rs routing-only violation)
- screen/run_end/ sub-domain added (Phase 2b): follows loading/main_menu pattern, PASS
- **Post-refactor audit 2026-03-13**: PASS — run/node/mod.rs routing-only violation RESOLVED (types extracted to resources.rs + sets.rs). No critical violations. One observation: run/node/ lacks its own plugin.rs (systems registered in parent run/plugin.rs).
- **Phase 2c audit 2026-03-13**: PASS — BehaviorPlugin added as breaker sub-domain (later extracted to top-level behaviors/ domain 2026-03-16). Per-consequence file layout in `consequences/` directory. Bridge systems consume BoltLost + BumpPerformed messages. Internal dispatch via Bevy observers (Events), not messages.
- **Full codebase audit 2026-03-16**: PASS — 0 critical violations, 2 minor observations (SelectedArchetype placement in shared.rs, double init_resource in tests). All 11 review categories clean.
- **fix/review-findings audit 2026-03-16**: PASS — animate_fade_out moved bolt→UI (correct), FadeOut shared type correct, multiplier insert_if_new precedence correct.
- **Phase 2d audit 2026-03-16**: PASS with 3 structural issues — RunSetupPlugin, PauseMenuPlugin, UpgradeSelectPlugin added as screen sub-domains.
- **Architecture compromise cleanup 2026-03-16**: 5 compromises resolved — bump multiplier in message, shared math module, NodePlugin extracted, RunLost message, fx domain.
- **Upgrade infrastructure audit 2026-03-16**: PASS with 3 structural issues. upgrades/ domain renamed to chips/.
- **Compromise cleanup verification audit 2026-03-16**: PASS — all 5 compromises confirmed resolved. 3 doc drift items.
- **Phase 2e audit 2026-03-16**: PASS — 0 critical violations. Chrono archetype, Prism archetype, interpolate/ domain.
- **behaviors/ domain extraction audit 2026-03-16**: PASS with 2 moderate issues (resolved).
- **Scenario coverage expansion audit 2026-03-17**: PASS — 0 critical violations, 1 observation. 10 new scenario RON files, 6 new invariant checkers (BoltSpeedInRange, BoltCountReasonable, ValidBreakerState, TimerMonotonicallyDecreasing, BreakerPositionClamped, PhysicsFrozenDuringPause). New bolt/queries.rs follows canonical layout. Visibility narrowing (pub→pub(crate)) across 12 domains, correctly preserving pub on types needed by scenario runner. InvariantParams resource + ScenarioStats resource added. DebugSetup.breaker_position field added.
- **Phase 4b.2 effect consumption audit 2026-03-19**: PASS with 1 medium observation. chips/ components read by physics, cells, breaker, bolt. width_boost_visual in Update. Stacking infra in chips/effects/mod.rs.
- **Full codebase audit 2026-03-19 (earlier)**: PASS — 0 critical violations, 6 structural issues, 4 doc drift items, 5 observations. Key issues: NodeSequence in wrong file, CellTypeRegistry/ArchetypeRegistry pub fields violate encapsulation pattern, BreakerSystems::GradeBump + BoltSystems::InitParams/Reset undocumented in ordering.md.
- **Full codebase audit 2026-03-19 (latest)**: PASS — 0 BLOCKING, 2 IMPORTANT (run/difficulty.rs non-canonical file, BoltSystems::InitParams/Reset still missing from ordering.md table), 3 MINOR, 1 doc drift. Registry encapsulation issues RESOLVED (CellTypeRegistry and ArchetypeRegistry now use private fields). NodeSequence placement in run/resources.rs is correct (not a violation).
- **Overclock trigger chain audit 2026-03-20 (session 6)**: BLOCKING — 1 cross-domain mutation in shockwave.rs.
- **Overclock trigger chain audit 2026-03-20 (session 7)**: PASS — 0 BLOCKING, 1 IMPORTANT (#[expect(clippy::cast_precision_loss)] precedent), 4 doc drift. Shockwave BLOCKING violation fully resolved: writes DamageCell messages, no direct CellHealth mutation. ChipKind removed, ChipDefinition.effects: Vec<ChipEffect>. chips/ widened to pub mod for scenario runner. OverclockEffectFired.bolt now Option<Entity> (NOTE: OverclockEffectFired was renamed to EffectFired in refactor/unify-behaviors). TriggerChain leaves have stacking fields. handle_cell_hit re-exported pub(crate) for E2E tests.
- **EntityScale feature audit 2026-03-20**: PASS — 0 BLOCKING, 2 IMPORTANT (ordering.md missing new OnEnter systems, cross-domain ActiveNodeLayout reads undocumented), 2 MINOR, 1 doc drift. EntityScale in shared/ correct. ActiveNodeLayout read-only reads by breaker/bolt follow established compromise patterns. Physics Option<&EntityScale> follows chip-effect query pattern. spawn_additional_bolt mid-node scale stamping correct.
- **TriggerChain unification Step 1 audit 2026-03-21**: PASS — 0 BLOCKING, 2 IMPORTANT (evaluate.rs missing new trigger variant handling; semantic overlap between behaviors/Trigger+Consequence and TriggerChain leaves), 3 MINOR (content.md stale, new leaves lack stacking fields, bridge_overclock_bump needs Early/Late mapping). 7 new TriggerChain variants (4 leaves: LoseLife, TimePenalty, SpawnBolt, BoltSpeedBoost; 3 triggers: OnEarlyBump, OnLateBump, OnBumpWhiff). Pure type definitions, no runtime wiring yet.
- **Behaviors unification audit 2026-03-21 (refactor/unify-behaviors)**: bolt/behaviors/ sub-domain deleted, BoltBehaviorsPlugin removed. ActiveOverclocks→ActiveChains. OverclockEffectFired→EffectFired. OverclockTriggerKind→TriggerKind. behaviors/consequences/ deleted; replaced by behaviors/effects/. ConsequenceFired removed; EffectFired is unified dispatch event. All bridge + effect systems now in BehaviorsPlugin. lib.rs visibility: behaviors module now pub.

## Debug Domain Structure (Phase 2f — audited 2026-03-17)
- debug/ restructured into three sub-domains: overlays/, telemetry/, hot_reload/
- Hot-reload pipeline: Bevy file_watcher → AssetEvent::Modified → propagate_*_defaults → *Config updated → propagate_*_config → entity components updated
- All hot-reload systems centralized in debug/hot_reload/
- 13 system files: 8 defaults propagators, 2 config propagators, 3 content/registry propagators
- HotReloadSystems::PropagateDefaults → PropagateConfig ordering, both in Update, gated on GameState::Playing
- Node layout changes mid-play: despawn + re-spawn cells immediately
- debug/recording/ sub-domain added: captures InputActions for scripted playback
- Workspace restructured: single crate now lives in game/ directory
- **Spawn signal system audit 2026-03-18**: PASS with 1 minor issue. 5 new messages (BoltSpawned, BreakerSpawned, WallsSpawned, CellsSpawned, SpawnNodeComplete), all sender-owns. check_spawn_complete coordinator in run/node/ reads cross-domain messages correctly via MessageReader. Minor: ScenarioLifecycle redundantly registers SpawnNodeComplete (NodePlugin already does).
