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

## Debug Domain Structure (Phase 2f — audited 2026-03-17)
- debug/ restructured into three sub-domains: overlays/, telemetry/, hot_reload/
- Hot-reload pipeline: Bevy file_watcher → AssetEvent::Modified → propagate_*_defaults → *Config updated → propagate_*_config → entity components updated
- All hot-reload systems centralized in debug/hot_reload/
- 13 system files: 8 defaults propagators, 2 config propagators, 3 content/registry propagators
- HotReloadSystems::PropagateDefaults → PropagateConfig ordering, both in Update, gated on GameState::Playing
- Node layout changes mid-play: despawn + re-spawn cells immediately
- debug/recording/ sub-domain added: captures InputActions for scripted playback
- Workspace restructured: single crate now lives in game/ directory
