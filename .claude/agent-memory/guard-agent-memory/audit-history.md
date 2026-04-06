---
name: Agent memory audit history
description: Recurring staleness patterns and duplication issues found in agent memory audits — use to direct future audits
type: project
---

## Full Audit — 2026-04-06 (refactor/cross-domain-prelude branch)

**Scope:** All agent memory directories (full audit)

**Context:** refactor/cross-domain-prelude branch. Active work on cross-domain prelude module. Wall builder, breaker builder, and transition infrastructure all merged to develop. Phase 7 file-split findings open.

**Issues found and fixed (6):**

1. **researcher-impact/wall-construction-sites-2026-04-02.md** — Date-stamped research for completed feature (wall-builder-pattern). Moved to ephemeral/. Updated MEMORY.md to "No active stable memories."

2. **guard-docs/phase-log.md** — Append-only session review log (dates, branches, files reviewed, drifts found). All durable facts are in known-state.md. Moved to ephemeral/phase-log.md. Updated MEMORY.md to remove link and add Session History section.

3. **guard-file-length/MEMORY.md** — Cross-reference pointed to phase6_findings.md as current. Superseded by phase7_findings.md (Wave 11). Fixed: updated to phase7_findings.md.

4. **guard-security/vetted_dependencies.md** — File accumulated per-session "Phase N note" entries (date-stamped branch-specific run outputs). Dep list was stale (2026-03-30). Durable content (paste advisory, machete clean baseline, rantzsoft_lifecycle security patterns, RON panic surface) extracted; session notes stripped. Updated MEMORY.md description.

5. **guard-dependencies/MEMORY.md** — dependency-snapshot.md link description said "2026-03-28" but file was updated to 2026-04-06. Fixed: updated description.

6. **guard-game-design/MEMORY.md** — evaluation-full-verification-2026-03-30.md link description said "Blocker 2 (catalog drift) open" but the file itself marks it RESOLVED (2026-04-06). Fixed: updated description to reflect both blockers RESOLVED.

**Files checked and NOT changed:**
- `reviewer-correctness/bug-patterns.md` — orchestration vacuous assertion still OPEN; confirmed
- `reviewer-performance/` — all 17 entries checked; accurate
- `writer-scenarios/` — all 8 files checked; accurate
- `guard-game-design/` — all evaluation files accurate
- `reviewer-architecture/` — all files accurate (Velocity2D cross-domain write gap still valid OPEN)
- `guard-docs/known-state.md` — comprehensive and current
- `planner-spec/` — MEMORY.md has bolt-domain-inventory.md link; confirmed inventory is current
- `guard-agent-memory/MEMORY.md` — correct (two links only)
- `reviewer-quality/MEMORY.md` — correctly uses ephemeral/ section for ephemeral files

**No cross-agent duplication found.**
**No broken MEMORY.md links found (all linked files verified to exist).**

**New staleness patterns detected this audit:**
- **Session review logs accumulate in stable root**: guard-docs/phase-log.md is an append-only session history log that should never have been in stable root. Rule: if a file description says "record of sessions", "per-session note", or "what was reviewed on date X", it belongs in ephemeral.
- **Security audit notes drift into per-session append pattern**: vetted_dependencies.md accumulated 15+ "Phase N note" sections mirroring the guard-docs/phase-log.md pattern. Security guards should store durable security baseline facts only; per-run "no new deps" notes are ephemeral.
- **MEMORY.md link descriptions go stale when files are updated**: guard-dependencies description said "2026-03-28" but the file had been updated to 2026-04-06. guard-game-design description still said "Blocker 2 open" after the file marked it RESOLVED. When updating a memory file, always re-read its MEMORY.md link description and update if stale.
- **Date-stamped research files for completed features**: researcher-impact stored wall builder research (2026-04-02) as stable even though the feature was merged weeks ago. Research gathered for a specific completed feature is ephemeral — move immediately after the feature merges.

---

## Full Audit — 2026-04-06 (develop — pre-merge of refactor/transition-post-process)

**Scope:** All agent memory directories (full audit)

**Context:** develop branch post-merge of refactor/transition-post-process. `rantzsoft_lifecycle` transition effects have `elapsed += time.delta_secs()` in all run systems. Phase 6 file-split spec open. InvariantKind is now 21 variants (not 23 as previously recorded).

**Issues found and fixed (10):**

1. **guard-docs/terminology.md** — `ShieldActive Variants` section described eliminated component. Fixed: replaced with ELIMINATED notice (matches shield_cross_domain_write.md).

2. **guard-docs/terminology.md** — InvariantKind count said 23 (chip-evolution-ecosystem era). Fixed: corrected to 21; noted non-existent variants `ValidStateTransitions`, `ValidBreakerState`, `PhysicsFrozenDuringPause`; actual variant is `ValidDashState`.

3. **guard-docs/known-state.md** — Line 86: InvariantKind total said 23, MutationKind said "verify current count." Fixed: corrected to 21 (InvariantKind) and 16 (MutationKind); noted `InjectWrongBoltSpeed` doesn't exist.

4. **writer-scenarios/pattern_invariant_substitution.md** — Listed 23 variants including 3 that don't exist (`ValidStateTransitions`, `ValidBreakerState`, `PhysicsFrozenDuringPause`). Referenced `InjectWrongBoltSpeed` (doesn't exist in MutationKind). Fixed: corrected to 21 variants with correct names including `ValidDashState`; corrected BoltSpeedAccurate mutation note.

5. **reviewer-scenarios/pattern_adversarial_quality.md** — 3 occurrences of `BoltSpeedInRange` (renamed to `BoltSpeedAccurate`). Fixed: all 3 replaced.

6. **guard-file-length/MEMORY.md** — Cross-reference pointed to `phase4_findings.md` as current open findings. Fixed: updated to `phase6_findings.md` (Wave 10, 2026-04-06).

7. **writer-scenarios/effect_ron_syntax.md** — `Explode` variant used `damage_mult` (field was renamed to `damage` in 2026-04-02 refactor; same fix applied to explode-field-rename.md in 2026-04-03 audit but `effect_ron_syntax.md` was missed). Fixed: corrected to `damage: 15.0`.

8. **reviewer-correctness/bug-patterns.md** — `rantzsoft_lifecycle` elapsed-never-incremented bug was OPEN (2026-04-03). Verified FIXED in develop: all 6 effect run systems now call `progress.elapsed += time.delta_secs()`. Fixed: updated to FIXED.

9. **guard-game-design/evaluation-full-verification-2026-03-30.md** — Blocker 2 (chip catalog doc drift) said "status unknown." Verified RESOLVED: chip-catalog.md uses `duration:` field for Shield, no additive/multiplicative framing. Fixed: marked RESOLVED.

10. **guard-docs/MEMORY.md** — `known-state.md` description was stale (said "through crate-routing-migration 2026-04-03"). Fixed: updated to "through prelude refactor 2026-04-06".

**Files checked and NOT changed:**
- `reviewer-performance/` — all 17 entries accurate
- `reviewer-scenarios/coverage_effect_system.md` — accurate, well-maintained
- `guard-docs/known-state.md` — comprehensive; content correct except InvariantKind count (fixed above)
- `reviewer-correctness/bug-patterns.md` — rantzsoft_lifecycle orchestration tests 8/9 vacuous assertion remains OPEN; cannot verify test pass/fail without running cargo
- `writer-scenarios/adversarial_patterns.md` — all BoltSpeedAccurate (correct)
- `writer-scenarios/pattern_effect_ron_syntax.md` — Explode uses `damage` (correct; was fixed in 2026-04-03 audit)
- `planner-review/MEMORY.md` — all 5 links present (fixed in prior audit)
- `reviewer-tests/MEMORY.md` — all 7 pattern files linked
- `guard-agent-memory/MEMORY.md` — correct

**Previous audit note corrected:**
- The 2026-04-06 (prior) audit noted `writer-scenarios/pattern_invariant_substitution.md` as "accurate (23 variants)" — this was incorrect. The 23-variant list included `ValidStateTransitions`, `ValidBreakerState`, `PhysicsFrozenDuringPause` which never existed. The actual count was 21. Fixed this audit.

**No cross-agent duplication found.**

**New staleness patterns detected this audit:**
- **InvariantKind variant list drifts silently**: `pattern_invariant_substitution.md` and `guard-docs/known-state.md` both claimed 23 variants with phantom variant names. After any InvariantKind change (add/remove/rename), IMMEDIATELY check both these files plus `guard-docs/terminology.md`.
- **Explode RON syntax slips across files**: `explode-field-rename.md` was fixed in 2026-04-03 audit but `effect_ron_syntax.md` was not. When fixing an RON field rename in one writer-scenarios file, check ALL writer-scenarios RON syntax reference files.
- **OPEN bug files don't get updated when fixes land on develop**: The `elapsed` bug landed on develop without updating the bug-patterns.md OPEN record. When a fix merges to develop, update the OPEN bug entry to FIXED immediately.

---

## Full Audit — 2026-04-06 (feature/effect-placeholder-visuals branch)

**Scope:** All agent memory directories (full audit)

**Context:** feature/effect-placeholder-visuals branch. Active work on effect visual placeholders. `rantzsoft_lifecycle` crate added with transition effects. Phase 5 file-split spec open (`phase5_findings.md`). `PauseMenuPlugin` now guards `handle_pause_input` with `not_in_transition + any_with_component::<PauseMenuScreen>`.

**Issues found and fixed (5):**

1. **planner-review/MEMORY.md** — 5 stable files unlinked: `feedback_counter_resource_state.md`, `feedback_test_impl_path_mismatch.md`, `patterns_counter_semantics.md`, `patterns_per_owner_state.md`, `patterns_world_borrow_conflict.md`. Fixed: added links in two new sections.

2. **runner-scenarios/stable/pause-menu-selection-missing-resource.md** — Described as open bug; the fix is confirmed in plugin.rs (`not_in_transition + any_with_component::<PauseMenuScreen>` compound run condition). Fixed: rewrote as RESOLVED record.

3. **runner-scenarios/MEMORY.md** — Link description still said "panics when..." (open bug framing). Fixed: updated to RESOLVED description.

4. **guard-game-design/audit-chip-evolution-coherence.md** — `chain_hit` chip status said "unknown; grep to verify." Verified: no RON file exists. Fixed: updated to "STILL MISSING (verified 2026-04-06)".

5. **planner-spec/bolt-domain-inventory.md** — Referenced `PlayingState::Active` (doesn't exist). Actual state is `NodeState::Playing`. Fixed: updated.

**Files checked and NOT changed:**
- `reviewer-performance/` — all 15 entries accurate including new `cleanup_on_exit_double_registration.md` and `handle_pause_input_params.md`
- `writer-scenarios/pattern_invariant_substitution.md` — invariant list accurate (23 variants)
- `reviewer-scenarios/coverage_effect_system.md` — accurate, well-maintained
- `guard-docs/known-state.md` — comprehensive and current (330 lines, over split threshold but dense reference doc)
- `reviewer-architecture/shield_cross_domain_write.md` — correctly marks ShieldActive as ELIMINATED
- `researcher-rust-idioms/pattern_screen_lifecycle_trait.md` + `pattern_declarative_routing.md` — historical design research, no current code references, but correct as planning artifacts
- `guard-agent-memory/MEMORY.md` — already had both links, no change needed

**No cross-agent duplication found.**
**No broken MEMORY.md links found (except planner-review, fixed above).**

**New staleness patterns detected this audit:**
- **State type renames go stale in inventory files**: `PlayingState::Active` → `NodeState::Playing` was missed in `planner-spec/bolt-domain-inventory.md`. After any state type rename or state hierarchy refactor, check all domain inventory files (planner-spec/*.md) for old state names.
- **Scenario runner bug files lag behind code fixes**: `pause-menu-selection-missing-resource.md` described an open bug that was fixed in the same feature branch. When a fix is landed, update the bug memory file to RESOLVED immediately.
- **MEMORY.md index drift for agents with many files**: `planner-review` had 5 of 6 stable files unlinked. Agents that accumulate feedback/pattern files over multiple sessions tend to forget to update their index. Check planner-review MEMORY.md on every future audit.

---

## Full Audit — 2026-04-03 (feature/wall-builder-pattern branch, transition infrastructure Wave 2–3)

**Scope:** All agent memory directories (full audit)

**Context:** feature/wall-builder-pattern. `rantzsoft_lifecycle` crate added with transition effect infrastructure (fade, dissolve, pixelate, wipe, iris, slide) and orchestration system. Two open bugs logged: `elapsed` never incremented in run systems, and vacuous assertions in orchestration tests 8/9. New phase5_findings.md added to reviewer-file-length (3 HIGH, 7 MEDIUM, 1 LOW).

**Issues found and fixed (15):**

1. **reviewer-correctness/known-correct-effects.md** — Lines 196-213: `ShieldActive charge-decrement` section described an eliminated component as "confirmed correct." Fixed: replaced with elimination notice pointing to shield_cross_domain_write.md.

2. **writer-scenarios/pattern_invariant_substitution.md** — Listed `ShieldChargesConsistent` (renamed to `ShieldWallAtMostOne`). Fixed: updated to `ShieldWallAtMostOne` with RENAMED note.

3. **reviewer-scenarios/coverage_effect_system.md** — Multiple stale references: `shield_bolt_loss_prevention`/`shield_cell_charge_depletion` (scenarios don't exist; replaced by shield_wall_reflection + shield_wall_at_most_one), `SizeBoostInRange` invariant (removed), `EffectiveSizeMultiplier`, `BoltSpeedInRange` (renamed), `ShieldChargesConsistent`. Fixed: all 6 locations updated.

4. **runner-scenarios/stable/explode-field-rename.md** — Described an open bug (damage_mult vs damage in explode_chaos.scenario.ron) that was already fixed. Fixed: rewrote as RESOLVED record.

5. **runner-scenarios/MEMORY.md** — Linked the explode bug as still-open. Fixed: updated description to RESOLVED.

6. **reviewer-architecture/known_gap_velocity_cross_domain_write.md** — Said "documented exceptions are ShieldActive (bolt and cells) and debug domain" — ShieldActive no longer an exception. Fixed: updated to reflect ShieldActive exception eliminated.

7. **reviewer-architecture/MEMORY.md** — shield_cross_domain_write.md description said "authorized to mutate ShieldActive directly." Fixed: updated to ELIMINATED description.

8. **writer-scenarios/effect_ron_syntax.md** — `Do(Shield(stacks: 1))` stale (Shield changed to `duration: f32`). Fixed: updated to `Do(Shield(duration: 5.0))` with note about removal of stacks field.

9. **writer-scenarios/adversarial_patterns.md** — 6 occurrences of `BoltSpeedInRange` (renamed to `BoltSpeedAccurate`). Fixed: replaced all with `BoltSpeedAccurate`.

10. **writer-scenarios/pattern_effect_ron_syntax.md** — `Explode: range, damage_mult` (should be `damage`) and Shield note missing duration field. Fixed: updated Explode and Shield entries.

11. **reviewer-performance/wall_builder_spawn_pattern.md** — "legacy spawn_walls system (manual spawn, no builder)" — stale, spawn_walls now uses the builder. Fixed: updated description.

12. **reviewer-quality/stable/color_from_rgb_canonical.md** — Described a duplicate `color_from_rgb` in chip_select/mod.rs that was removed in the state folder restructure. Fixed: verified removed, rewrote as RESOLVED; updated MEMORY.md link description.

13. **guard-dependencies/dependency-snapshot.md** — No mention of new `rantzsoft_lifecycle` workspace member (added on this branch). Fixed: added note in Changes section with dep list and security surface summary.

14. **reviewer-bevy-api/confirmed-patterns.md** — Two entries used `ShieldActive` as the concrete example type (Option<&'static mut ShieldActive> in query alias, commands.remove::<ShieldActive>()). ShieldActive eliminated. Fixed: genericized both examples and added NOTE about elimination.

15. **reviewer-quality/MEMORY.md** — stale_helper_name.md link description still said "has stale helper name" (already RESOLVED). Fixed: updated link description to RESOLVED.

**Files checked and NOT changed:**
- `researcher-codebase/project-cross-domain-topology.md` — ShieldActive coupling already noted as ELIMINATED; accurate
- `guard-docs/known-state.md` — Already contains comprehensive Shield refactor documentation
- `reviewer-correctness/bug-patterns.md` — rantzsoft_lifecycle bugs correctly marked OPEN (2026-04-03)
- `reviewer-file-length/phase5_findings.md` — New file, correct and current
- `guard-security/known_unsafe_blocks.md` — Historical log; old `wall/` paths are correct for their dates

**No broken MEMORY.md links found.**
**No cross-agent duplication found.**

**New staleness patterns detected this audit:**
- **Shield RON field rename (stacks→duration)**: effect_ron_syntax.md and pattern_effect_ron_syntax.md had stale `stacks: u32` syntax. After any EffectKind field rename, check ALL writer-scenarios RON syntax reference files.
- **Invariant renames carry through multiple files**: `BoltSpeedInRange` → `BoltSpeedAccurate` was missed in adversarial_patterns.md (6 occurrences) even after prior audits fixed writer-scenarios/pattern_invariant_substitution.md. Check adversarial_patterns.md and coverage_effect_system.md whenever invariants are renamed.
- **ShieldActive in known-correct-effects.md**: Phase 3-5 "confirmed correct" sections persist across refactors. After any Shield-type component elimination, immediately check known-correct-effects.md for the old component in "confirmed correct" entries — these sections are maintained as append-only and drift badly.
- **runner-scenarios bug files need RESOLVED status after RON field fixes**: When a scenario RON is fixed for a field rename, the stable bug file should be updated to RESOLVED immediately — not left for the next audit to notice.

---

## Full Audit — 2026-04-02 (feature/wall-builder-pattern branch, state lifecycle refactor Wave 1)

**Scope:** All agent memory directories (full audit)

**Context:** feature/wall-builder-pattern. State lifecycle refactor Wave 1 (file restructure) moved/renamed multiple game domains: `screen/` eliminated (cleanup → `state/cleanup.rs`; UI subdomains → `state/run/node/hud/`, `state/run/chip_select/`, `state/menu/`), `run/` → `state/run/`, `wall/` → `walls/` (singular → plural). Shield refactor (2026-04-02) eliminated `ShieldActive` component entirely — Shield is now a timed visible floor wall entity (`ShieldWall` + `ShieldWallTimer`). `effect/effects/shield.rs` split into `effect/effects/shield/` directory module. `dispatch_cell_effects` moved from `cells/systems/` to `state/run/node/systems/`. `dispatch_wall_effects` deleted (inline in `Wall::builder().spawn()`). `dispatch_breaker_effects` deleted (replaced by `spawn_or_reuse_breaker` in prior branch).

**Issues found and fixed (13):**

1. **reviewer-architecture/dispatch_pattern_ownership.md** — Described `dispatch_breaker_effects` in `breaker/systems/`, `dispatch_cell_effects` in `cells/systems/`, `dispatch_wall_effects` in `wall/systems/` — all stale. Fixed: updated to reflect all three as ELIMINATED or moved (`dispatch_cell_effects` → `state/run/node/systems/`).

2. **reviewer-architecture/shield_cross_domain_write.md** — Described `ShieldActive` cross-domain writes by bolt and cells domains; `ShieldActive` no longer exists. Fixed: completely rewritten to document that this exception is ELIMINATED and the mechanism redesigned.

3. **reviewer-performance/wall_builder_spawn_pattern.md** — 5 file paths using `src/wall/` prefix. Fixed: updated all to `src/walls/` and `spawn_walls` to `src/state/run/node/systems/spawn_walls/system.rs`.

4. **reviewer-correctness/bug-patterns.md** — Two `Location:` entries referenced `breaker-game/src/wall/builder/core/terminal.rs`; also `dispatch_wall_effects` described as existing. Fixed: updated both paths to `src/walls/`; noted `dispatch_wall_effects` deleted.

5. **reviewer-file-length/phase4_findings.md** — `effect/effects/shield.rs` (889 lines) still listed as open HIGH. Fixed: moved to SPLIT; updated `wall/registry.rs` MEDIUM item to `walls/registry/core.rs`.

6. **reviewer-file-length/MEMORY.md** — Description still showed shield.rs as open HIGH and old `wall/` path. Fixed: updated description.

7. **guard-security/ron_deserialization_patterns.md** — Referenced `screen/plugin.rs` as where `CleanupOnNodeExit` entities are despawned; `screen/` eliminated. Fixed: updated to `state/cleanup.rs (previously screen/plugin.rs — screen domain eliminated)`.

8. **guard-docs/known-state.md** — `Wall::builder()` in `wall/builder/` — stale path. Fixed: updated to `walls/builder/`.

9. **reviewer-bevy-api/confirmed-patterns.md** — `wall/registry.rs` → `walls/registry/core.rs`; `wall/components.rs` → `walls/components.rs`. Fixed both.

10. **writer-scenarios/pattern_scenario_structure.md** — RON template used stale `invariants:` and `expected_violations:` field names. Fixed: updated to `disallowed_failures:` and `allowed_failures:`; added note about rename.

11. **guard-file-length/split-patterns.md** — `cells/systems/dispatch_cell_effects.rs` path stale (moved to `state/run/node/systems/`). Fixed: updated path with migration note.

12. **researcher-rust-idioms/pattern_declarative_routing.md** — Referenced `advance_node` in `run/plugin.rs:84` (stale path). Fixed: updated to `state/run/plugin.rs` with migration note.

13. **researcher-codebase/project-cross-domain-topology.md** — Referenced `screen` as "pure consumer / natural leaf crate candidate" (domain eliminated); also described `ShieldActive` cross-domain coupling (eliminated). Fixed: updated both entries to reflect current state.

**Files checked and NOT changed (already accurate or historical records):**
- `guard-security/vetted_dependencies.md` — already has `refactor/state-folder-structure (2026-04-02)` entry documenting the rename; old paths in audit notes are correct for their time
- `guard-security/known_unsafe_blocks.md` — already has state-folder-structure confirmation entry
- `guard-docs/phase-log.md` — historical session log; old paths accurate for their time

**No new cross-agent duplication found.**
**No broken MEMORY.md links found.**

**New staleness patterns detected this audit:**
- **Domain renames (singular→plural)**: `wall/` → `walls/`. Any file with `wall/` in a path (not `walls/`) is likely stale. Risk files: reviewer-correctness/bug-patterns.md (Location entries), reviewer-performance (spawn pattern files), reviewer-bevy-api/confirmed-patterns.md, guard-file-length/split-patterns.md.
- **Domain eliminations (screen→state)**: When a top-level domain folder is eliminated and its contents reorganized (screen/ → state/), ALL path references across all agents go stale at once. Risk files: any memory file with `screen/plugin.rs`, `ui/`, `run/plugin.rs`, `run/node/` paths.
- **ShieldActive elimination**: When a cross-domain component exception is eliminated by architectural redesign, the reviewer-architecture exception doc becomes entirely obsolete. After any refactor redesigning an exception pattern, check reviewer-architecture for stale exception docs.
- **RON field renames**: When scenario RON field names are renamed (invariants→disallowed_failures, expected_violations→allowed_failures), writer-scenarios RON templates go stale immediately. Check pattern_scenario_structure.md and pattern_invariant_substitution.md after any invariant infrastructure rename.
- **Shield.rs directory split**: phase4_findings.md HIGH list goes stale when a HIGH file is split as part of a feature branch (not a dedicated refactor/file-splits). After any feature merge that incidentally splits a HIGH item, update phase4_findings immediately.

---

## Full Audit — 2026-04-02 (feature/breaker-builder-pattern branch, 9 waves of implementation)

**Scope:** All agent memory directories (full audit)

**Context:** feature/breaker-builder-pattern. Recent commits include `spawn_or_reuse_breaker` wiring
(replaces 4 init systems), `PrimaryBreaker` inserted via builder `.primary()`, `BoltRespawnOffsetY`/
`BoltRespawnAngleSpread`/`BoltInitialAngle` deleted (Wave 6), `BreakerStatOverrides` eliminated,
`BoltConfig` eliminated, RON rename `.bdef.ron` → `.breaker.ron`, architecture docs fully updated
(Wave 9 doc session). Evolution RON files moved from `chips/evolution/` to `chips/evolutions/` (plural).
High/medium file splits: `bolt/builder/` dir, `anchor/tests/`, `tether_beam/tests/fire_tests/`,
`dash/tests/flash_step/`, `move_breaker/` dir, `bump/tests/anchor_multipliers/` dir.

**Issues found and fixed (9):**

1. **reviewer-quality/stable/stale_helper_name.md** — Said `default_bump_visual_params()` is stale.
   The function was already renamed to `default_bump_feedback()`. Fixed: updated to RESOLVED.

2. **reviewer-architecture/pattern_breaker_builder_migration.md** — Said "Old spawn chain STILL WIRED
   in plugin.rs" and listed a visibility fix. Both resolved: `spawn_or_reuse_breaker` is live,
   `pub(crate) mod builder` is correct. Fixed: updated Current state to 2026-04-02, architecture doc
   drift section updated to reflect Wave 9 fixes.

3. **reviewer-correctness/bug-patterns.md** — "PrimaryBreaker marker never inserted" was LATENT;
   now FIXED (builder `.primary()` inserts it). ".definition() omits BoltRespawnOffsetY" was LATENT;
   now MOOT (components deleted in Wave 6). BreakerBuilder with_width/with_lives/rendered() latent
   bugs had stale production-context ("not yet wired"); updated to reflect builder is wired.
   Fixed: updated all four entries.

4. **researcher-system-dependencies/cross-domain-ordering-map.md** — Row for `BreakerSystems::InitParams`
   (eliminated in builder migration). Fixed: removed row.

5. **guard-game-design/audit-chip-evolution-coherence.md** — "5 Missing RON Files" section listed
   `flashstep.evolution.ron`, `chain_reaction.evolution.ron`, `feedback_loop.chip.ron`,
   `powder_keg.chip.ron` as missing. All now exist. Fixed: updated to show current status.

6. **reviewer-scenarios/coverage_bolt_builder_migration.md** — Referenced `BoltRespawnOffsetY`,
   `BoltRespawnAngleSpread`, `BoltInitialAngle` (deleted Wave 6). Fixed: updated gap #2 description
   and coverage table row to use current component names.

7. **guard-file-length/split-patterns.md** — Referenced `dispatch_breaker_effects/tests.rs` which
   no longer exists (replaced by spawn_or_reuse_breaker). Fixed: updated note.

8. **reviewer-file-length/phase4_findings.md** — All 4 HIGH priority files now SPLIT (bolt/builder.rs
   → dir module; anchor/tether_beam/fire_tests/flash_step tests → dir modules). mod.rs violations
   FIXED. Two MEDIUM items also split (move_breaker, anchor_multipliers). Fixed: updated all sections.

9. **reviewer-file-length/MEMORY.md** — Description for phase4_findings.md showed stale "4 HIGH open".
   Fixed: updated description.

**Additional fix:**
- **planner-spec/bolt-domain-inventory.md** — Listed deleted components `BoltRespawnOffsetY`,
  `BoltRespawnAngleSpread`, `BoltInitialAngle` as current. Fixed: updated to Wave 6 state.

**No new cross-agent duplication found.**
**No broken MEMORY.md links found.**

**New staleness patterns detected this audit:**
- **Breaker builder wiring swap**: When `spawn_or_reuse_breaker` replaces old init systems, memory
  files saying "old spawn chain still wired" go stale immediately. Check: reviewer-architecture/
  pattern_breaker_builder_migration.md, reviewer-correctness/bug-patterns.md (PrimaryBreaker bug).
- **Wave-numbered component deletions**: When a wave deletes components (Wave 6: BoltRespawnOffsetY,
  BoltRespawnAngleSpread, BoltInitialAngle), check: planner-spec/bolt-domain-inventory.md,
  reviewer-scenarios/coverage_bolt_builder_migration.md, reviewer-correctness/bug-patterns.md.
- **File split completions**: When HIGH/MEDIUM items from phase4_findings.md are split, update both
  phase4_findings.md and reviewer-file-length/MEMORY.md simultaneously.
- **RON directory renames**: When RON asset directories are renamed (evolution/ → evolutions/),
  check guard-game-design audit files that reference specific paths.

---

## Full Audit — 2026-04-01 (feature/chip-evolution-ecosystem branch, speed_boost inline + InvariantKind renames)

**Scope:** All agent memory directories (full audit)

**Context:** feature/chip-evolution-ecosystem. Recent commits include bolt typestate builder pattern
(48766c5) and attraction/gravity_well steering model fix (c007143). This audit focused on
staleness introduced by: (1) speed_boost.rs `recalculate_velocity` becoming an inline helper
(not a registered ECS system); (2) InvariantKind variant removals/renames (`EffectiveSpeedConsistent`
and `SizeBoostInRange` removed, `BoltSpeedInRange` renamed to `BoltSpeedAccurate`); (3) chain_reaction
evolution rename to "Shock Chain"; (4) `prepare_bolt_velocity` ordering references.

**Issues found and fixed (15):**

1. **researcher-codebase/effect-domain-inventory.md** — Table rows for SpeedBoost/DamageBoost/
   Piercing/SizeBoost/BumpForce listed `recalculate_speed`/`recalculate_damage` etc as `(REAL)`
   runtime ECS systems. These were eliminated in the Effective* cache removal. Also: stale key
   file path `src/effect/core/types.rs` → `src/effect/core/types/definitions/enums.rs`. Fixed.

2. **researcher-codebase/collision-system-map.md** — Line 29: `**Ordering:** after BoltSystems::PrepareVelocity`
   — `PrepareVelocity` was eliminated in bolt builder migration. Fixed: updated ordering note.

3. **reviewer-correctness/known-correct-effects.md** — Top section referenced `recalculate_*` systems
   and `Effective*` components that no longer exist; did not explicitly warn against re-flagging
   their absence. Fixed: rewrote section to reflect direct-read model and warn against re-flagging.

4. **reviewer-architecture/known_gap_velocity_cross_domain_write.md** — Said both attraction and
   gravity_well `run .before(BoltSystems::PrepareVelocity)` — PrepareVelocity was eliminated.
   Fixed: updated to reference `apply_velocity_formula()` inline and note elimination.

5. **reviewer-architecture/pattern_bolt_builder_migration.md** — Open item listed bolt/builder.rs
   at stale 2511 lines. Fixed: updated to ~2700 lines per vetted_dependencies.md 2026-03-31.

6. **reviewer-bevy-api/MEMORY.md** — Description overstated confirmed-patterns.md coverage claiming
   "message system, queries, state, SystemParam derive, Time API, and component spawning" all covered,
   when many sections are still TODO. Fixed: updated description to reflect actual coverage.

7. **planner-review/MEMORY.md** — Missing Session History line for ephemeral files. Fixed: added
   `## Session History / See ephemeral/ — not committed.`

8. **planner-review/effect-domain-patterns.md** — Push/Pop section referenced `EffectiveBumpForce`
   (removed in cache removal). Fixed: added NOTE about removal and correct on-demand path
   (`ActiveBumpForces.total()`).

9. **writer-scenarios/pattern_invariant_substitution.md** — Listed 25 variants including removed
   `EffectiveSpeedConsistent` and `SizeBoostInRange`; named `BoltSpeedInRange` (renamed to
   `BoltSpeedAccurate`); also included stale `InjectWrongSizeMultiplier` mutation. Fixed: updated
   to 23 variants, added REMOVED section, corrected self-test mutations.

10. **guard-docs/known-state.md** — InvariantKind total listed as 25 with removed variants still
    present. Fixed: updated to 23 variants with explanation of removals and rename.

11. **runner-scenarios/bug_chain_reaction_collision.md** — Bug marked as still open. Verified
    `chain_reaction.evolution.ron` now has `name: "Shock Chain"` — collision is RESOLVED. Fixed:
    added RESOLVED header and moved original content to historical record.

12. **runner-scenarios/MEMORY.md** — chain_reaction entry showed no RESOLVED status. Fixed: updated
    to show RESOLVED status matching the bug file.

13. **runner-scenarios/bug_tether_beam_bolt_accumulation.md** — Referenced `spawn_extra_bolt()` which
    was removed in the bolt builder migration. Fixed: updated to `Bolt::builder()` with note.

14. **researcher-system-dependencies/speed-boost-checker-ordering.md** — Did not mention that
    `recalculate_velocity()` is now inline in `fire()`/`reverse()` (Option B fix implemented).
    Fixed: added inline recalculation description and Option B resolution note.

15. **researcher-system-dependencies/MEMORY.md** — Description for speed-boost-checker-ordering.md
    did not reflect Option B resolution. Fixed: updated description.

**Additional fix from previous session (reviewer-performance/phase3-stat-effects.md):**
- Listed `prepare_bolt_velocity` as a hot-path `.multiplier()` call site (eliminated in builder
  migration). Fixed: updated to `apply_velocity_formula()` with note about elimination.

**No new cross-agent duplication found.**
**No broken MEMORY.md links found.**

**New staleness patterns detected this audit:**
- **InvariantKind variant lifecycle**: When InvariantKind variants are removed or renamed,
  these files ALL need updates: writer-scenarios/pattern_invariant_substitution.md (full list),
  guard-docs/known-state.md (total count), runner-scenarios/bug_* files (if variant was the bug
  trigger). Also check: planner-spec/bolt-domain-inventory.md if it lists scenario coverage.
- **Inline helper vs registered ECS system**: When a system is converted to an inline helper
  (not registered via `app.add_systems`), memory files describing "runtime systems" go stale.
  researcher-codebase/effect-domain-inventory.md runtime systems column is primary risk file.
- **evolution RON renames**: When an evolution chip's `name:` field changes, runner-scenarios
  bug files documenting name-collision bugs should be checked for RESOLVED status.

---

## Full Audit — 2026-03-31 (feature/chip-evolution-ecosystem branch, bolt builder migration)

**Scope:** All agent memory directories (full audit)

**Context:** feature/chip-evolution-ecosystem. Bolt builder migration eliminated `init_bolt_params`,
`prepare_bolt_velocity`, `BoltSystems::PrepareVelocity`, `BoltSystems::InitParams`, `spawn_extra_bolt`,
`CollisionQueryBolt`, and all `Effective*` components. Multiple memory files had stale references.

**Issues found and fixed (14):**

1. **researcher-codebase/bolt-spawn-component-map.md** — Referenced `init_bolt_params`, `CollisionQueryBolt`,
   `EffectivePiercing`, `EffectiveDamageMultiplier` — all eliminated. Fixed: rewrote component inventory and
   query section for current builder-based spawn and `BoltCollisionData` named struct.

2. **planner-spec/bolt-domain-inventory.md** — Listed `BoltBaseSpeed`/`BoltMinSpeed`/`BoltMaxSpeed`
   as components (now fields on `BoltConfig` + spatial crate types), `CollisionQueryBolt` (replaced by
   `BoltCollisionData`), `InitParams`/`PrepareVelocity` set variants (eliminated). Fixed.

3. **researcher-system-dependencies/system-map-bolt-velocity2d.md** — Listed `prepare_bolt_velocity`
   and `BoltSystems::PrepareVelocity` as active. Fixed: updated table and ordering section; added WARNING
   about stale gravity_well/attraction ordering anchors.

4. **runner-scenarios/bug_effective_speed_state_gate.md** — Bug is moot; Effective* cache removal
   eliminated all types involved. Fixed: added RESOLVED header with rationale; updated MEMORY.md entry.

5. **guard-docs/known-state.md** — "Confirmed Correct (stat-effects merge)" section described
   `EffectivePiercing`/`EffectiveDamageMultiplier` in plugins.md as correct; "Key Architectural Fact"
   still mentioned `EffectiveDamageMultiplier` as active. Fixed: marked stat-effects section as
   SUPERSEDED; updated key fact section to use `ActiveDamageBoosts`.

6. **researcher-codebase/scenario-failure-trace.md** — Failures 2+3 described Effective*-based bugs
   now fixed. Fixed: added RESOLVED note at top; updated MEMORY.md description.

7. **researcher-codebase/effect-domain-inventory.md** — Two rows referenced `spawn_extra_bolt` (removed);
   `recalculate_*` pattern section described removed systems. Fixed: updated spawn references; rewrote
   stat model note section.

8. **researcher-codebase/entity-leak-analysis.md** — Two references to `spawn_extra_bolt` (removed).
   Fixed: replaced with `Bolt::builder()` attribution.

9. **researcher-codebase/collision-system-map.md** — Wall collision section said "resets PiercingRemaining
   to `EffectivePiercing.0`" — `EffectivePiercing` removed. Fixed.

10. **reviewer-performance/phase3-stat-effects.md** — Two references to `CollisionQueryBolt` (replaced
    by `BoltCollisionData`). Fixed: updated to new query type name with clarifying note.

11. **planner-spec/effect-spawn-bolts-inventory.md** — Listed `spawn_extra_bolt` as key helper (removed).
    Fixed: replaced with builder call pattern.

12. **reviewer-architecture/known_gap_effects_mod_production_logic.md** — Said `spawn_extra_bolt` lives
    in fire_helpers.rs; it was subsequently removed. Fixed.

13. **reviewer-architecture/pattern_bolt_builder_migration.md** — Open item about arch docs partially
    resolved; updated to reflect guard-docs/known-state.md 2026-03-31 additions.

14. **researcher-bevy-api/MEMORY.md** — Description overstated coverage of confirmed-patterns.md
    (many sections still TODO). Fixed: updated description to be accurate.

**No new cross-agent duplication found.**
**No broken MEMORY.md links found.**

**New staleness patterns detected this audit:**
- **Bolt builder migration scope**: init_bolt_params, prepare_bolt_velocity, BoltBaseSpeed/Min/Max,
  CollisionQueryBolt, spawn_extra_bolt, EffectivePiercing, EffectiveDamageMultiplier all eliminated in
  one branch. At least 10 memory files had stale references. After any bolt domain refactor, audit:
  researcher-codebase (bolt-spawn, collision, entity-leak, effect-domain, scenario-failure-trace),
  planner-spec (bolt-inventory, effect-spawn-bolts), researcher-system-dependencies, reviewer-performance,
  reviewer-architecture.
- **`EffectiveSpeedConsistent` and `SizeBoostInRange` bug files**: After cache removal, any runner-scenarios
  bug file referencing Effective* should be marked RESOLVED — the architectural cause no longer exists.

---

## Full Audit — 2026-03-30 (feature/scenario-coverage branch, Wave 3 review)

**Scope:** All agent memory directories (full audit)

**Context:** feature/scenario-coverage branch (current). refactor/file-splits merged into develop (commit 35dfece), splitting all 25 previous MEDIUM items plus adding new Wave 3 HIGH/MEDIUM findings. Previous "gravity_well max==0 infinite loop" bug is FIXED in scenario-coverage branch (confirmed in code lines 51-53).

**Issues found and fixed (8):**

1. **researcher-codebase/effect-system-domain-map.md** — Stale holdover at line 92: "but all trigger bridges are stubs, so no trigger evaluation happens yet." Contradicted by line 49 of the same file ("ALL trigger bridges are REAL"). Fixed: corrected to "all trigger bridges are REAL."

2. **researcher-bevy-api/MEMORY.md** — Listed individual ephemeral file `ephemeral/bevy018-post-processing.md` as an indexed entry. Violates convention (ephemeral = one line only). Fixed: removed individual file link.

3. **reviewer-performance/wave1-stat-boost-and-fifo-effects.md** — Claimed the second `is_none()` guard in stat-boost fire() is "dead code." Code inspection (speed_boost.rs lines 21-25) and reviewer-quality/wave1-lazy-init-fifo-patterns.md both confirm it IS reachable (entity with Active* but no Effective*). Fixed: corrected the claim.

4. **reviewer-file-length/phase4_findings.md** — "Previously open MEDIUM priority" section listed 25 files as needing splits. All 25 have been split by refactor/file-splits. Also the frontmatter description was stale. Fixed: replaced MEDIUM section with SPLIT confirmation; updated description.

5. **reviewer-file-length/MEMORY.md** — Description still referenced "25 MEDIUM items remain open." Fixed: updated to reflect Wave 3 state.

6. **runner-scenarios/MEMORY.md** — File was 1 line (effectively empty). Fixed: added Session History line.

7. **reviewer-architecture/known_gap_velocity_cross_domain_write.md** — Referenced `effect/effects/gravity_well.rs` (line 89) but gravity_well is now a directory module. Fixed: updated path to `gravity_well/effect.rs`.

8. **reviewer-architecture/known_gap_cleanup_markers.md** — Referenced `effect/effects/gravity_well.rs` (line 69, stale line number). Fixed: updated path to `gravity_well/effect.rs`.

**No new cross-agent duplication found.**
**No broken MEMORY.md links found.**

**Stale patterns confirmed still open:**
- 1-frame stale ordering for Effective* consumers (bolt/plugin.rs) — reviewer-correctness/bug-patterns.md
- BASE_BOLT_DAMAGE hardcoding in combat effects (guard-game-design blocker 1)
- chip-catalog doc drift (guard-game-design blocker 2)
- check_aabb_matches_entity_dimensions false positive for non-1.0 EntityScale layouts — reviewer-correctness/bug-patterns.md
- gravity_well + spawn_phantom missing despawned-entity guard — reviewer-correctness/bug-patterns.md OPEN
- TetherChainActive resource leaks across node boundaries — reviewer-correctness/bug-patterns.md OPEN
- file-length HIGH items: 6 files in reviewer-file-length/phase4_findings.md (Wave 3 scan)
- file-length mod.rs violations: 2 files in reviewer-file-length/phase4_findings.md (Wave 3 scan)

**Items resolved since last audit:**
- gravity_well::fire() infinite loop when max == 0: FIXED in scenario-coverage branch (lines 51-53)
- check_aabb_matches_entity_dimensions.rs 581-line HIGH item: SPLIT (now directory module)
- All 25 previous MEDIUM items: ALL SPLIT by refactor/file-splits

**New staleness patterns detected this audit:**
- **Phase4_findings MEDIUM carry-forward list**: After any large refactor/file-splits merge, the "MEDIUM priority still open" section becomes entirely stale. Always clear and re-run reviewer-file-length.
- **Cross-agent performance/quality contradiction on "dead code"**: reviewer-performance wave1 file incorrectly called the second guard dead code; reviewer-quality correctly said it was reachable. When agents disagree, verify against codebase.
- **Directory module refactors change file paths**: After any refactor/file-splits merge, file-path references in memory (especially reviewer-architecture gap files) may point to .rs files that are now directory modules. Check reviewer-architecture after any split refactor.

---

## Full Audit — 2026-03-30 (develop branch, post feature/missing-unit-tests + feature/scenario-coverage merge)

**Scope:** All agent memory directories (full audit)

**Context:** develop branch at fad7dfa. Two recent branches: feature/missing-unit-tests (58 tests,
3 new invariants, source marker structs converted to unit structs) and feature/scenario-coverage
(8 new scenarios, 3 new invariant checkers, new frame mutation helpers).

**Issues found and fixed (10):**

1. **runner-linting/lint_patterns_core.md** — Stale: claimed dead_code warnings on `PulseSource.0`,
   `ShockwaveSource.0`, `TetherBoltMarker.0`, `CellDestroyedAt.position`. All four source markers
   are now unit structs (no tuple field). lint_state_current.md correctly shows only 1 doc warning.
   Fixed: rewrote "Current State" section; moved dead_code entries to Historical as RESOLVED.

2. **runner-linting/MEMORY.md** — Description for lint_patterns_core.md didn't mention source-marker
   dead_code resolution. Fixed: updated description.

3. **researcher-system-dependencies/MEMORY.md** — Linked specific ephemeral file
   (ephemeral/invariant-infrastructure-map.md) as an indexed entry. Violates convention (ephemeral =
   one line only). Fixed: removed individual link.

4. **reviewer-performance/phase4-runtime-effects.md** — Referenced `effects/helpers.rs` for
   `entity_position()` helper. Correct path is `effects/fire_helpers.rs`. Fixed.

5. **reviewer-performance/phase3-stat-effects.md** — "run_if Gap Confirmed" section described
   `EffectSystems::Recalculate` as lacking `run_if`. Code confirms it now has
   `.run_if(in_state(PlayingState::Active))` at plugin.rs line 13. Fixed: marked FIXED.

6. **reviewer-performance/phase5-complex-effects.md** — "Intentional Patterns" section still said
   `process_piercing_beam` run_if gap "acceptable at current scale" but earlier in the same file
   it was correctly marked FIXED. Fixed: corrected the Intentional Patterns entry.

7. **guard-docs/known-state.md** — Line said "Runtime Effects entry added to Current section (In
   Progress)" but phase-log confirms it was changed to Done on 2026-03-30. Fixed.

8. **writer-scenarios/pattern_invariant_substitution.md** — Main variant list had 21 items
   (missing `ChainArcCountReasonable`), then listed the remaining 4 in a separate section.
   Fixed: consolidated into single list of 25 variants.

9. **guard-docs/MEMORY.md** — Description for known-state.md and phase-log.md both had stale date
   references. Fixed: updated both descriptions.

10. **reviewer-performance/MEMORY.md** — Descriptions for phase3 and phase5 files didn't reflect
    the run_if fixes. Fixed: updated both descriptions.

**No new cross-agent duplication found.**
**No MEMORY.md broken links found** (all linked files verified to exist).

**Stale patterns confirmed still open at time of this audit (some now resolved — see audit above):**
- gravity_well::fire() infinite loop when max == 0 — FIXED in scenario-coverage branch
- 1-frame stale ordering for Effective* consumers (bolt/plugin.rs)
- BASE_BOLT_DAMAGE hardcoding in combat effects (guard-game-design blocker 1)
- chip-catalog doc drift (guard-game-design blocker 2)
- HIGH file-length: check_aabb_matches_entity_dimensions.rs (581 lines) — SPLIT
- MEDIUM file-length items: 25+ files — ALL SPLIT by refactor/file-splits

**New staleness patterns detected this audit:**
- **Source marker struct-to-unit-struct conversions**: Conversion from tuple structs to unit structs eliminates dead_code field warnings. runner-linting/lint_patterns_core.md is primary risk file.
- **EffectPlugin run_if gaps**: Recalculate set run_if gap is now fixed. New system sets added to EffectPlugin must carry run_if guard.
- **Stale "In Progress" plan.md entries**: After any phase merge, verify forward-looking "In Progress" labels in known-state.md and similar files.
- **ephemeral link convention**: MEMORY.md for agents with ephemeral content must have exactly one "See ephemeral/ — not committed." line. No individual file links.

See [audit-history-archive.md](audit-history-archive.md) for audits prior to 2026-03-30.

---

## Recurring Staleness Patterns

- **File-split refactors**: reviewer-file-length phase findings go stale after any commit that splits multiple files. After any "refactor: split" commit, verify HIGH/MEDIUM open lists immediately.
- **Effect domain evolves fast**: researcher-codebase effect inventory files go stale after each phase. After any phase touching `effect/effects/*.rs` or `effect/triggers/*.rs`, verify: test counts, placeholder status, component lists, and message references.
- **Bug status drift**: reviewer-correctness/bug-patterns.md OPEN items lag behind fixes. Cross-check with known-correct-effects.md which often records FIXED status first.
- **Trigger bridge stub → real**: All trigger bridges are real (feature/source-chip-shield-absorption). Future audits noting "stubs" for bump/bumped/impact/impacted/node_start/node_end/cell_destroyed should verify against codebase.
- **lint_patterns_core.md description drift**: After any major cleanup, verify "current" vs "historical" sections are accurate.
- **Cleanup marker gaps**: reviewer-architecture tracks which effect entities lack CleanupOnNodeExit. Check after any new effect that spawns entities.
- **runner-linting state**: lint_state_current.md is point-in-time. After any lint-fixing session, verify MEMORY.md description matches file content.
- **Message removals**: When messages removed from messages.rs, memory referencing them goes stale. researcher-codebase bolt-message-pattern-map.md and guard-docs/known-state.md are primary risk.
- **Component renames and removals**: grep for old name before auditing (e.g., PhantomTimer → gone, kill_count → cells_destroyed).
- **Placeholder-to-real transitions**: Effect phase files (inventory, performance) must be updated at each phase.
- **guard-file-length vs reviewer-file-length duplication**: guard-file-length should NOT have per-phase findings; those belong in reviewer-file-length.
- **Transform/Position2D gap files**: reviewer-architecture and reviewer-bevy-api both track these. After any branch fixing violations, check BOTH files.
- **"Forward-looking" items in guard-docs/known-state.md**: After any phase that ships content, verify the "Intentionally Forward-Looking" section.
- **Phase4 HIGH items can be split silently**: After any large feature merge, re-check the HIGH priority list in phase4_findings.md.
- **Source marker struct-to-unit-struct conversions**: After refactors touching source attribution markers, check lint_patterns_core.md.
- **EffectPlugin new system sets**: New sets added to EffectPlugin must carry run_if guard.
- **MEDIUM carry-forward list in phase4_findings.md**: After any refactor/file-splits merge, the entire "MEDIUM priority still open" section may be stale. Clear and re-run reviewer-file-length.
- **Cross-agent performance/quality disagreements**: reviewer-performance may mark code patterns as "dead code" that reviewer-quality correctly identifies as reachable. On disagreement, verify against the actual code.
- **Directory module refactors change file paths**: After any refactor/file-splits merge, file-path references in reviewer-architecture gap files may point to .rs files that are now directory modules. Check reviewer-architecture path references after any split refactor.
- **InvariantKind variant removals/renames**: When variants are removed or renamed, update: writer-scenarios/pattern_invariant_substitution.md, guard-docs/known-state.md (count), and any runner-scenarios bug files tied to the removed variant.
- **Inline helper vs registered ECS system**: When a `pub(crate)` system is refactored to a private inline helper (not registered via `add_systems`), researcher-codebase/effect-domain-inventory.md "Runtime systems" column goes stale. Verify after any recalculate_* or similar cleanup.
- **Evolution RON name renames**: When evolution chip `name:` changes, check runner-scenarios bug files for name-collision bugs — they may be RESOLVED.
- **Domain renames (singular→plural path renames)**: `wall/` → `walls/`. After any domain path rename, audit: reviewer-correctness/bug-patterns.md (Location entries), reviewer-performance (spawn pattern files), reviewer-bevy-api/confirmed-patterns.md, guard-file-length/split-patterns.md.
- **Domain eliminations**: When a top-level domain is reorganized (screen/ → state/), ALL path references across all agents go stale. High-risk agents: researcher-codebase (topology files), researcher-rust-idioms (routing pattern examples), guard-security (cleanup entity references), reviewer-architecture (dispatch pattern docs).
- **Cross-domain exception elimination**: When a known architectural exception (e.g., ShieldActive cross-domain writes) is eliminated by redesign, the reviewer-architecture exception doc becomes obsolete. After any refactor that removes an established cross-domain pattern, check reviewer-architecture for stale exception docs.
- **RON field renames (scenario infrastructure)**: When scenario RON field names change (invariants→disallowed_failures, expected_violations→allowed_failures), update: writer-scenarios/pattern_scenario_structure.md, writer-scenarios/pattern_invariant_substitution.md, spec-format-tests.md scenario coverage examples.
- **Feature-branch incidental file splits**: When a HIGH file-length item is split as a side effect of a feature branch (not a dedicated refactor/file-splits), phase4_findings.md HIGH list goes stale at merge time. Check after any feature that touches large effect files.
- **EffectKind field renames in RON syntax files**: When an EffectKind variant changes its field names (e.g., Shield stacks→duration, Explode damage_mult→damage), ALL writer-scenarios RON syntax reference files go stale: effect_ron_syntax.md, pattern_effect_ron_syntax.md, adversarial_patterns.md (if the variant appears in example patterns). After any EffectKind schema change, grep ALL writer-scenarios stable files.
- **InvariantKind renames propagate to adversarial_patterns.md**: BoltSpeedInRange→BoltSpeedAccurate was present in adversarial_patterns.md but not fixed in prior audits. InvariantKind renames must be applied to adversarial_patterns.md in addition to pattern_invariant_substitution.md and guard-docs/known-state.md.
- **InvariantKind count/list has multiple owners**: guard-docs/known-state.md, guard-docs/terminology.md, and writer-scenarios/pattern_invariant_substitution.md ALL record the InvariantKind variant list. They MUST be updated together — updating one and not the others causes divergence. After any InvariantKind add/remove/rename, grep all three.
- **OPEN bugs don't self-update when fixes land on develop**: reviewer-correctness/bug-patterns.md OPEN items must be manually updated when a fix merges to develop. The `elapsed` bug was OPEN in memory for weeks after the fix landed. Check bug-patterns.md on every audit against the actual code.
- **guard-file-length MEMORY.md cross-reference goes stale**: After each new phase6/7/N findings file is added to reviewer-file-length, the guard-file-length cross-reference must be updated to point to the latest phase file.
- **known-correct-effects.md as append-only drift risk**: Phase 3–5 "confirmed correct" sections are never cleaned up when components are eliminated. After any cross-domain component elimination (ShieldActive, Effective* types), immediately check known-correct-effects.md for stale "confirmed correct" entries describing the eliminated component's behavior.
- **runner-scenarios bug files after RON field fixes**: When a scenario RON file is updated to fix a field rename bug, the corresponding stable bug file in runner-scenarios/ should immediately be updated to RESOLVED, not left open for the next audit to find.

## Agents Accumulating Memory Fastest

1. **reviewer-file-length** — Highest staleness risk after large refactors. Phase findings go stale every time a split commit lands.
2. **researcher-codebase** — Effect domain changes every phase; inventory goes stale. Also accumulates stale domain references (screen/, ShieldActive) after structural refactors.
3. **reviewer-correctness** — bug-patterns.md OPEN items lag behind fixes; cross-check known-correct-effects.md.
4. **runner-linting** — lint_state_current.md is point-in-time; description drift is likely.
5. **reviewer-architecture** — known_gap_cleanup_markers.md tracks open gaps that close over time. Exception docs (dispatch_pattern_ownership, shield_cross_domain_write) go stale when exceptions are eliminated.
6. **writer-scenarios** — InvariantKind variant list AND RON field names must be updated when variants or infrastructure is renamed.
7. **runner-scenarios** — Bug files for resolved bugs (name collisions, Effective* state-gate issues) linger as "open" without RESOLVED markers.
8. **researcher-system-dependencies** — System ordering files go stale when system sets are eliminated or systems become inline helpers.
9. **reviewer-performance** — Spawn pattern docs with file paths (e.g., wall_builder_spawn_pattern.md) go stale after domain renames.
10. **guard-security** — CleanupOnNodeExit references (despawn locations) go stale after domain reorganizations.
