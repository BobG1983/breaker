---
name: Known Correct Patterns
description: Code patterns confirmed correct that should not be re-flagged in reviews
type: reference
---

## Known Correct Patterns (Do Not Flag)

### Shockwave VFX (2026-03-23, feature/wave-3-offerings-transitions)
- `Option<ResMut<Assets<ColorMaterial>>>` in `animate_shockwave` — valid Bevy 0.18 system parameter; returns None in test setups that omit the resource. Confirmed correct.
- `Annulus::new(0.85, 1.0)` — parameter order is (inner_radius, outer_radius). Correct thin ring at unit scale; Scale2D expands it. Do not re-flag as wrong-order.
- `materials.get_mut(mat_handle.id())` inside query iterator — safe because Assets<ColorMaterial> is a resource, disjoint from component queries. No borrow conflict.
- `Color::with_alpha` on `Color::LinearRgba` — sets alpha directly on the LinearRgba inner value; HDR channels (values > 1.0) are preserved. Confirmed from bevy_color source.
- `animate_shockwave` scheduled in `Update`, `tick_shockwave` in `FixedUpdate` — the scheduling asymmetry is intentional (visual-only in Update, simulation in FixedUpdate). No ordering bug.

### entity_scale feature (2026-03-20, feature/overclock-trigger-chain)
- `apply_entity_scale_to_breaker` uses `Option<Res<ActiveNodeLayout>>` and early-returns if None — correct guard. Runs `.after(BreakerSystems::InitParams).after(NodeSystems::Spawn)`. `NodeSystems::Spawn` is in a `.chain()` with `set_active_layout` first, so `ActiveNodeLayout` exists before this system runs. No ordering hazard.
- `apply_entity_scale_to_bolt` runs `.after(BoltSystems::InitParams).after(NodeSystems::Spawn)` — same correct ordering as breaker.
- `width_boost_visual` formula `(base + boost) * scale` is correct Option B stacking per design.
- `bolt_breaker_collision` `half_w = (breaker_w.half_width() + width_boost.map_or(0.0, |b| b.0 / 2.0)) * breaker_scale` — WidthBoost stores full width (b.0/2 gives half), then multiplied by scale. Correct.
- `bolt_breaker_collision` `expanded_half = Vec2::new(half_w + r, half_h + r)` — r uses bolt's scaled effective radius. Correct.
- `bolt_lost` lost-detection threshold `playfield.bottom() - r` where r = radius * entity_scale — smaller bolt needs less clearance below floor. Correct behavior: scaled bolt is lost slightly sooner.
- `spawn_additional_bolt` reads `layout.as_ref().map_or(1.0, |l| l.0.entity_scale)` and always inserts `EntityScale(entity_scale)` — correct; never missing EntityScale on extra bolts.
- `NodeLayout.validate()` checks entity_scale before cols/rows — order matters only for error messages; both checks always run via `?`. Correct.
- `deserialize_entity_scale_at_minimum` test uses `f32::EPSILON` tolerance for 0.5 deserialization — float representation of 0.5 is exact (power of 2), so EPSILON is safe here.
- `clamp_bolt_to_playfield` effective radius = `radius.0 * bolt_entity_scale.map_or(1.0, |s| s.0)` — correct; a scaled-down bolt sits closer to the walls before triggering the clamp.
- `init_breaker_params` calls `insert_if_new` BEFORE `init_archetype` runs. Archetype's plain `insert` overwrites defaults — correct last-write-wins.
- `reset_breaker` uses `f32::midpoint(playfield.left(), playfield.right())` — correct.
- `handle_cell_hit` replaces HashSet with `Vec + peek()` early exit — correct at MAX_BOUNCES=4 bound.
- `animate_fade_out` in UI domain (Update, PlayingState::Active guard). FadeOut entities have `CleanupOnNodeExit`.
- `bolt_lost` respawn angle: `Vec2::new(speed*sin(angle), speed*cos(angle))` — angle-from-vertical convention, speed preserved.
- `set_active_layout` wraps `node_index % registry.layouts.len()` — deliberate.
- `handle_main_menu_input` reads `ButtonInput<KeyCode>` directly (InputActions cleared in FixedPostUpdate) — intentional.
- `spawn_bolt` adds `BoltServing` only on first node; subsequent nodes launch immediately.
- `animate_bump_visual` subtracts previous frame's offset before applying new one — correct differential.
- `track_node_completion` uses `remaining.is_changed()` — correct guard.
- `handle_cell_hit` despawns via commands while iterating `reader.read()` — safe; commands flush later.
- `spawn_side_panels` has `existing.is_empty()` guard — does NOT re-spawn on node re-entry. StatusPanel persists (CleanupOnRunEnd).
- `spawn_timer_hud` has explicit `if !existing.is_empty() { return; }` guard.
- `spawn_lives_display` uses `existing.iter().next().is_some()` guard.
- Lives wrapper has no cleanup marker — cleaned via parent cascade when StatusPanel despawned.
- Timer wrapper has `CleanupOnNodeExit` — cleaned at node exit, gone by RunEnd.
- `handle_run_setup_input` and `handle_pause_input` use `ButtonInput<KeyCode>` directly — same pattern as main menu.
- `toggle_pause` guarded by `run_if(in_state(GameState::Playing))`.
- `RunSetupSelection`, `PauseMenuSelection`, `ChipSelectTimer`, `ChipSelectSelection`, `ChipOffers` — stale-resource pattern. All re-inserted fresh on OnEnter. Correct.
- `transition_queued` in RunState: `advance_node` resets to false on each node transition.
- Bevy 0.18 sub-state `OnExit` fires when parent state exits. No redundant cleanup needed for pause quit.
- `GameRng::default()` seeds from 0. `reset_run_state` reseeds via `ChaCha8Rng::from_os_rng()`.
- `MenuLeft`/`MenuRight` share keys with `MoveLeft`/`MoveRight` — harmless, different state contexts.
- `update_run_setup_colors` sorts cards alphabetically, matching `handle_run_setup_input`.
- `apply_debug_setup` uses post-teleport `transform.translation` for `ScenarioPhysicsFrozen.target` — correct because mutation happens before the insert call.
- `check_timer_monotonically_decreasing` resets Local to None when NodeTimer absent (node transition) — correct; no false positive on new node start. Now uses `(remaining, total)` tuple — total comparison uses `f32::EPSILON` safely because total is set once and not mutated intra-node.
- `check_valid_breaker_state` `retain(|e, _| breakers.contains(*e))` runs after the per-entity insert loop — correct ordering; Bevy generational IDs ensure recycled entity IDs are distinct. Do not re-flag.
- `check_bolt_in_bounds` margin `bolt_radius.map_or(0.0, |r| r.0 + 1.0)` applied to all four walls — correct. Missing BoltRadius degrades to 0.0 (no false positives). Violation message does not show effective bound (cosmetic, not a logic bug).
- `check_bolt_count_reasonable` queries `With<ScenarioTagBolt>` — one-frame tolerance on newly spawned extra bolts is acceptable.
- `evaluate_pass` with `expected_violations: Some([])` → requires `violations.is_empty() && logs.is_empty()` — correct (vacuously all-fired AND all-in-list).
- `HybridInput::actions_for_frame` returns empty during scripted phase without advancing chaos RNG — correct, RNG state only advances when chaos phase is active.
- `#![cfg_attr(test, allow(...))]` in lib.rs is the approved conditional form, not a bare `#[allow(...)]` — do not re-flag.
- `check_valid_breaker_state` legal set includes `Settling → Dashing` — correct; `handle_idle_or_settling` allows dash from Settling state.
- `RenderSetupPlugin` inserts `ClearColor(PlayfieldConfig::default().background_color())` at plugin-build time using compile-time defaults — intentional; RON default matches Rust default `[0.02, 0.01, 0.04]`.

## Wave 2a Two-Phase Destruction Correct Patterns (feature/spatial-physics-extraction, 2026-03-25)
- `cleanup_destroyed_cells` ordering `.after(EffectSystems::Bridge)` — cleanup runs after the entire Bridge set, including `bridge_cell_death` (which is inside Bridge). Order is correct: entity lives during bridge evaluation, is despawned after.
- `tick_until_timers` / `check_until_triggers` / `reverse_children` — systems are correctly implemented. `UntilTimers`/`UntilTriggers` components are intentionally only populated by test code in Wave 2a; Until wiring is Wave 2b scope.
- `apply_speed_boosts` empty-vec path: `product()` of empty iterator returns 1.0, so `base_speed * 1.0 = base_speed`. Only runs on entities with `ActiveSpeedBoosts` — not auto-inserted in production, so this runs on zero entities. Do not flag as "always overrides velocity."
- `bridge_timer_threshold` index-based removal in reverse order — correct; indices collected ascending, removed descending to preserve validity.
- `bridge_cell_death` `any_destroyed` flag: fires global `OnCellDestroyed` evaluation once per frame (not per cell). This matches the pre-existing `bridge_cell_destroyed` semantics. Intentional fire-once-if-any pattern. Do not re-flag.
- `bridge_bump` `BumpSuccess` evaluation uses `targets = vec![EffectTarget::Entity(bolt_entity)]`: targets set to bolt entity. Correct — `BumpSuccess` and grade-specific triggers both use the bolt as the effect target.
- `CellDestroyed` type is entirely removed in Wave 2a. `RequestCellDestroyed` (internal bridge trigger) + `CellDestroyedAt` (downstream consumers) are the two-phase replacement. No dual-read path exists.
- `bridge_timer_threshold` zero-total: `ratio = 0.0` when `timer.total == 0.0`. All positive thresholds are satisfied immediately. Design intent: a node with no timer fires threshold chains immediately. Do not re-flag as zero-division.
- `BoltLostWriters` `Result<MessageWriter<RequestBoltDestroyed>, SystemParamValidationError>` fallback: the `Err` arm runs only when the message is not registered. In production it is always registered. Legacy graceful-degradation pattern. Do not flag as bug.

## B4-B6 Template/Inventory/Offering Confirmed Correct (feature/spatial-physics-extraction, 2026-03-24)
- `expand_template` sets `max_stacks = template.max_taken` on all rarity variants — all variants from the same template always share the same cap value. Do not flag as inconsistency.
- `template_taken` counts total stacks across all rarity variants (one increment per `add_chip` call, one decrement per `remove_chip` call). This is intentional and correct.
- `remove_chip` grabs `template_name` from the entry BEFORE decrementing stacks. Even when stacks hit 0 and the entry is removed, the local `template_name` binding is still valid for the subsequent template_taken decrement. Safe.
- `generate_offerings` dedup loop produces fewer than `offers_per_node` results when fewer unique templates exist. This is intentional and tested by `generate_offerings_fewer_templates_than_slots`. Do not flag as count bug.
- `is_template_maxed` is test-only (not called in production code). Production pool gate is `is_chip_available`. The two separate sources (`template_maxes` vs `def.max_stacks`) do not diverge in production because `expand_template` guarantees uniform `max_stacks` per template.
- `seed_chip_registry` skips `Rarity::Evolution` chips from the `chips` collection — intentional; evolutions are handled by `seed_evolution_registry`.
- `seed_chip_registry` `Local<bool>` guard: persists for app lifetime. Seeding only runs once — correct.
- `add_chip` template cap check comes BEFORE individual cap check — correct ordering per spec (prevent template overflow before allowing individual increment).
- `or_insert(def.max_stacks)` in template_maxes is a first-write-wins register for the template's max. Since all chips from the same template have identical max_stacks (guaranteed by expand_template), any write order produces the same value.

## SeedableRegistry Phase 1 (develop, 2026-03-26)
- `MessageReader<AssetEvent<D>>` in `propagate_defaults` is correct in Bevy 0.18 — `MessageReader` is the renamed `EventReader` and works for all `Event`/`Message` types, confirmed by `propagate_node_layout_changes.rs:52` production use.
- `handles.loaded = true` set inside the folder-resolution block before per-asset collection succeeds is intentional: the handles vec is stable once the folder loads; the asset loop retries on subsequent frames via early-return. Not a bug.
- Zero-handles after `try_typed` filtering: `seed_registry` treats this as a successful seed of empty registry and sets `*seeded = true`. BUG — see bug-patterns.md.
- `R::extensions()` is NOT used inside `seed_registry` — only by `RonAssetLoader` registration. The filter-by-type via `try_typed` is correct; extension filtering is the loader's responsibility.
- `propagate_registry` system does not exist — hot-reload for registries is not implemented by this plugin. This is a missing feature, not a logic bug in the seed path.

## B1-B3 TriggerChain Flatten Confirmed Correct (feature/spatial-physics-extraction, 2026-03-24)
- `apply_chip_effect` match order `OnSelected` → `is_leaf()` → catchall: `OnSelected` is not a leaf so it is correctly intercepted by arm 1 before reaching the catchall that pushes to `ActiveChains`. Correct arm ordering.
- Bare leaf fallback arm (line 51-57): `chain if chain.is_leaf()` fires `ChipEffectApplied` immediately. `OnPerfectBump` etc. are not leaves, so they correctly fall through to the `ActiveChains` push arm. No OnSelected variant can erroneously reach `ActiveChains`.
- 9 handlers (`handle_piercing`, `handle_damage_boost`, `handle_bolt_speed_boost`, `handle_chain_hit`, `handle_bolt_size_boost`, `handle_width_boost`, `handle_breaker_speed_boost`, `handle_bump_force_boost`, `handle_tilt_control_boost`) all correctly early-return via `let TriggerChain::Variant = ... else { return; }` pattern. No cross-contamination.
- `handle_bolt_speed_boost` matches `Target::Bolt` only; `handle_breaker_speed_boost` matches `Target::Breaker` only; `Target::AllBolts` routes to `handle_speed_boost` (`effect/effects/speed_boost.rs` — was `behaviors/effects/speed_boost.rs` before C7-R) which observes `SpeedBoostFired` typed event (was `EffectFired` before C7-R), not `ChipEffectApplied` — so `AllBolts` used in `OnSelected` is a silent no-op. This is INTENTIONAL per test `speed_boost_all_bolts_via_on_selected_is_silent_noop` (apply_chip_effect.rs:492). AllBolts is only meaningful at trigger-fire time, not chip-select time.
- `evaluate()` in `effect/evaluate.rs` (was `behaviors/evaluate.rs` before C7-R): or-pattern covers TriggerKind variants (PerfectBump, CellImpact, BreakerImpact, WallImpact, BumpSuccess, CellDestroyed, BoltLost, EarlyBump, LateBump, BumpWhiff). `OnSelected` is intentionally absent — it's not a runtime trigger. Correct.
- `effect/active.rs` doc update (was `behaviors/active.rs` before C7-R): `None` for archetype chains, `Some(name)` for chip/evolution chains. Correct; `dispatch_chip_effects` pushes `Some(msg.name.clone())`.

## feature/spatial-physics-extraction Confirmed Correct (2026-03-24)
- `handle_multi_bolt` formula `base_count + stacks.saturating_sub(1) * count_per_level` — correct. Operator precedence: `*` binds tighter than `+`; `stacks.saturating_sub(1) * count_per_level` is the extra-level term added to base. Do not re-flag.
- `detect_most_powerful_evolution` uses `.max_by(|a, b| a.1.total_cmp(b.1))` — correct for NaN-free f32 damage values. `total_cmp` is the right choice here.
- `spawn_run_end_screen` `spawn_highlights_section` match on `HighlightKind` is exhaustive over all 15 variants. Confirmed correct.
- `track_evolution_damage` `entry(name.clone()).or_insert(0.0) += msg.damage` — correct accumulation pattern, does not double-count.
- `tick_shield_removes_at_zero_or_below` test comment says "dt ~0.0167" (wrong — actual fixed delta is 1/64 ≈ 0.015625 s). The test passes correctly because 0.01 - 0.015625 < 0.0. The comment is inaccurate but not a logic bug. Do not re-flag the test as wrong.

## feature/seedable-registry EvolutionTemplate Refactor Confirmed Correct (2026-03-27)
- `expand_evolution_template` sets `template_name: None` — evolution chips have no template and no per-rarity variants. Correct.
- `expand_chip_template` sets `description: String::new()` — chip templates have no description field; description is evolution-only. Correct design decision.
- `build_chip_catalog` inserts recipe THEN def (line 56-57): recipe first, chip second. Both use `template.name.clone()`. `recipe.result_name` == `def.name` at build time. `eligible_recipes` resolves by name from ChipCatalog — the chip is guaranteed to be in catalog when recipes are evaluated at runtime. Correct ordering.
- `propagate_chip_catalog` double-guards `!template_registry.is_added()` AND `!evolution_registry.is_added()` — prevents spurious rebuild on app startup when both registries are freshly initialized. Correct.
- `ChipTemplate` keeps `description: String::new()` in `expand_chip_template` — intentional absence of per-chip description for template-expanded chips. Not a missing field bug.
- Triggers `DestroyedCell`, `Died`, `Impacted(Cell)` in new chip RONs now have bridge systems in `effect/triggers/`. Do NOT re-flag as inert.

## refactor/rantzsoft-prelude-and-defaults lifecycle.rs Confirmed Correct (2026-03-26)
- `apply_perfect_tracking` bump condition `bolt_position.y > breaker_position.0.y && distance <= PERFECT_TRACKING_BUMP_THRESHOLD` — correct; fires while bolt is above breaker and within 20 world units of contact. Bolt travels downward (y < 0), so condition is only reachable for the relevant hemisphere.
- `BumpMode::AlwaysWhiff` → `force_grade.0 = None` AND Bump IS injected by proximity check (AlwaysWhiff is NOT excluded by the NeverBump guard) — correct compound behavior. Bump action fires (opens the window) but ForceBumpGrade=None grades it as a whiff. The test `perfect_tracking_always_whiff_mode_writes_bump` confirms Bump is written. Do not flag the NeverBump guard exclusion as missing AlwaysWhiff.
- `BumpMode::NeverBump` → `force_grade.0 = None` + no Bump injection — correct; bolt never triggers a bump window.
- `BumpMode::Random` `choices.choose(&mut perfect.rng)` — `choices` is a non-empty 3-element slice; `choose()` always returns `Some`. `if let Some(&chosen)` always matches. Correct.
- `ChipSelectionIndex` reset in `bypass_menu_to_playing` + increment in `auto_skip_chip_select` — correct; index resets on each run restart (OnEnter MainMenu), then advances once per chip select node. Maps node N chip select to `chip_selections[N-1]`.

## Effect Domain Rewrite Confirmed Correct (develop, 2026-03-27)
- `TransferCommand` splits `Do` vs non-`Do` children: pushes non-Do to BoundEffects/StagedEffects first, then fires Do effects. Order is correct — effects fire after chains are installed (world is mutably accessed for fire after the entity_ref borrow ends). No borrow conflict.
- `walk_bound_node` only handles top-level `When` nodes — silently skips bare Do, Once, Until. Correct by design: `BoundEffects` holds When-nodes; bare Do and Until are handled elsewhere.
- `evaluate_staged_effects` `retain + additions` pattern — old entries removed and new additions extended after the loop. Net result: consumed entries are dropped, their produced children are added. Correct.
- `Res<Time>` used in `FixedUpdate` systems (`tick_time_expires`, `tick_shockwave`) — in Bevy 0.18, `Res<Time>` inside `FixedUpdate` gives the fixed-step delta automatically. Not a bug.
- `desugar_until`: non-Do children pushed to BOTH `new_bound` and `pushed_chains` (for Reverse) — correct; the chain is installed (BoundEffects) and simultaneously tracked for reversal (Reverse.chains). Do not flag as duplication.
- `RemoveChainsCommand` removes by value equality from BoundEffects — correct; clones from Until desugaring are equal by PartialEq. Safe for the normal case of distinct chip stacks.
- `init_breaker` only pushes `On(target: Breaker, ...)` entries — `On(target: Bolt, ...)` are skipped. Intentional; Bolt-targeted effects from breaker RON are dispatched by `dispatch_breaker_effects` (Wave 6 stub).
- `InitBreakerQuery` `Without<LivesCount>` guard: when `life_pool = Some(n)`, LivesCount is inserted via commands and on the next frame the entity no longer matches. Idempotent for the `Some` case across multiple `OnEnter(Playing)` transitions. Correct for all production breakers (Aegis has `life_pool: Some(3)`).
- `pulse::fire` collects bolt positions first then spawns — avoids world borrow conflict. Correct.
