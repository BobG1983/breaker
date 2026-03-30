---
name: Agent memory audit history
description: Recurring staleness patterns and duplication issues found in agent memory audits — use to direct future audits
type: project
---

## Full Audit — 2026-03-30 (Full Verification Tier, develop post-merge c9964b7)

**Scope:** All agent memory directories (full audit)

**Context:** Refactor commit c9964b7 split 23 oversized .rs files into directory modules.
This made the reviewer-file-length phase findings massively stale.
Also: three bugs previously tracked as OPEN in reviewer-correctness/bug-patterns.md were
actually FIXED in the feature/source-chip-shield-absorption branch.

**Issues found and fixed (15):**

1. **runner-tests/build-failure-bolt-breaker-query.md** — Bug was fixed; code at line 127-142
   uses correct 2-level tuple nesting. Tombstoned file, removed from MEMORY.md.

2. **runner-tests/build-failure-dstest-2026-03-29.md** — Already marked RESOLVED in MEMORY.md
   but file had full error details. Replaced with brief tombstone. Removed from MEMORY.md.

3. **reviewer-correctness/bug-patterns.md** — Three OPEN bugs were actually FIXED:
   dispatch_chip_effects max-stacks (has `continue;`), bypass_menu PendingBreakerEffects
   (fully implemented), apply_pending_bolt_effects (uses insert_if_new). Updated all three
   to FIXED status with confirmation details.

4. **reviewer-correctness/known-correct-effects.md** — `kill_count` stale field name in
   entropy_engine section; field is `cells_destroyed`. Test name also stale. Updated both.

5. **reviewer-quality/MEMORY.md** — Description for phase5 still said "kill_count vocabulary".
   Updated to "cells_destroyed vocabulary (not kill_count)".

6. **guard-security/vetted_dependencies.md** — Listed `proptest 1 (dev-dependency)` which was
   removed. Updated to note removal.

7. **researcher-bevy-api/MEMORY.md** — Empty (1 line) despite `confirmed-patterns.md` existing.
   Added link to confirmed-patterns.md.

8. **guard-performance/MEMORY.md** — Empty (1 line). Added Session History convention line.

9. **researcher-crates/MEMORY.md** — Empty. Added Session History line.

10. **researcher-rust-errors/MEMORY.md** — Empty. Added Session History line.

11. **researcher-rust-idioms/MEMORY.md** — Empty. Added Session History line.

12. **researcher-system-dependencies/MEMORY.md** — Empty. Added Session History line.

13. **runner-release/MEMORY.md** — Empty. Added Session History line.

14. **reviewer-file-length/phase4_findings.md** — Massively stale: all HIGH and most MEDIUM
    files from c9964b7 list are now split. Rewrote entirely to show post-refactor state with
    remaining MEDIUM open items. Updated MEMORY.md description.

15. **reviewer-file-length/phase3_findings.md** — All Phase 3 flagged files split. Converted
    to archived reference. Updated MEMORY.md description.

16. **guard-file-length/split-patterns.md** — "Only dispatch_breaker_effects/tests.rs at 812
    lines needs Strategy C conversion" — it was split. Updated to note resolution.

17. **guard-file-length/ephemeral/{phase3,phase4}_findings.md + pattern_bolt_module_structure.md**
    — Exact duplicates of reviewer-file-length stable files, stored in wrong agent's ephemeral
    directory. Replaced with tombstone content pointing to reviewer-file-length.

18. **guard-file-length/MEMORY.md** — Added cross-reference to reviewer-file-length for
    per-phase findings, and Session History line.

**Stale patterns confirmed still open:**
- gravity_well.rs still lacks CleanupOnNodeExit (reviewer-architecture/known_gap_cleanup_markers.md)
- piercing_beam.rs Transform fallback still open (reviewer-bevy-api/confirmed-patterns.md line 78)
- TransferCommand silently drops non-Do children (reviewer-correctness/bug-patterns.md)
- 1-frame stale ordering gap for Effective* consumers (reviewer-correctness/bug-patterns.md)
- runner-scenarios bugs: speed_clamp_bypass, second_wind_multi_wall, entropy_engine_bolt_storm

**Key new pattern (c9964b7 fallout):**
Phase findings files in reviewer-file-length go stale after any large refactor that splits files.
After any commit with "split" or "refactor" in the message touching many .rs files, immediately
verify the HIGH/MEDIUM open lists in reviewer-file-length/phase4_findings.md.

---

## Targeted Audit — 2026-03-29 (pre-merge gate, feature/source-chip-shield-absorption)

**Scope:** All agent memory directories (full audit)

**Issues found and fixed (9):**

1. **runner-tests/MEMORY.md** — empty despite `build-failure-bolt-breaker-query.md` existing. Fixed: added link and Session History line.

2. **guard-architecture/MEMORY.md** — completely empty file. Fixed: added Session History line per convention.

3. **researcher-impact/MEMORY.md** — header-only, no content. Fixed: replaced with Session History line.

4. **runner-linting/lint_patterns_core.md** — stale: described "Wave 8 stub" warning patterns (missing_const_for_fn, needless_pass_by_ref_mut, dead_code on init_breaker/dispatch_breaker_effects, use_self, suboptimal_flops, redundant_clone) that are all resolved. Current warnings are only 4 dead_code on unused tuple fields in source marker structs. Fixed: rewrote file to reflect current state; moved old patterns to Historical section.

5. **runner-linting/MEMORY.md** — description for lint_patterns_core.md listed stale warning category names. Fixed: updated description to note it's historical.

6. **researcher-codebase/effect-domain-inventory.md** — "Placeholder stubs (still Wave 8)" section listed all bump/bumped/impact/impacted/node_start/node_end/cell_destroyed triggers as stubs — ALL are fully real implementations. Fixed: replaced stub section with REAL status for all bridges.

7. **researcher-codebase/effect-system-domain-map.md** — "Most trigger bridges are stubs" — completely false. All are real. Fixed: replaced stubs list with accurate REAL status for each bridge.

8. **researcher-quality/phase3-stat-effects-patterns.md** — `dispatch_breaker_effects` described as "documented stub with TODO: Wave 6 comment" — it's now a real system in its own directory. Fixed: updated to REAL status.

9. **reviewer-architecture/pattern_effect_direct_spawn.md** — still said "The messages.md doc lists SpawnChainBolt as a message" — that doc drift was fixed in a prior session. Fixed: updated to note inconsistency is resolved.

**Stale patterns still open (confirmed correct):**
- gravity_well.rs still lacks CleanupOnNodeExit (reviewer-architecture/known_gap_cleanup_markers.md — confirmed by code)
- piercing_beam.rs Transform fallback still open (reviewer-bevy-api/confirmed-patterns.md — confirmed by code at lines 43-49)
- dispatch_chip_effects max-stacks double-dispatch bug still open (reviewer-correctness/bug-patterns.md — confirmed by code at system.rs lines 52-56)
- 1-frame stale ordering gap for Effective* consumers still open (reviewer-correctness/bug-patterns.md — correct)

**Key new finding (trigger bridges all real):**
This branch included Wave 8 trigger bridge implementation that converted ALL trigger bridges from stubs to real implementations. Future audits after this branch merges should NOT flag these as stubs.

---

## Targeted Audit — 2026-03-29 (post source_chip threading, feature/runtime-effects)

**Scope:** orchestrator, reviewer-tests, reviewer-correctness, reviewer-quality, reviewer-bevy-api, reviewer-architecture, reviewer-performance, runner-linting, plus dependent researcher-codebase and guard-docs files

**Issues found and fixed (12):**

1. **orchestrator/MEMORY.md** — empty (1 line). Fixed: added Session History line per convention.

2. **reviewer-tests/MEMORY.md** — missing Session History line despite ephemeral/ dir existing. Fixed: added Session History link.

3. **runner-linting/MEMORY.md** — description for lint_state_current.md said "2 ERRORS in chain_lightning/tests/tick_tests.rs" but file body said errors resolved. Fixed: updated description to match file content.

4. **researcher-codebase/effect-domain-inventory.md** — listed ChainLightning, TetherBeam, EntropyEngine, PiercingBeam, SpawnBolts, Explode, SpawnPhantom as "placeholders" — all now real implementations. SpawnPhantom listed `PhantomTimer` which is removed. Trigger bridge `bolt_lost` listed as stub — now real. `recalculate_*` systems described as "placeholder body" — now real. Fixed: updated REAL section to include all Phase 4/5/source_chip effects; updated trigger bridge status; corrected recalculate note.

5. **researcher-codebase/effect-system-domain-map.md** — `bolt_lost` trigger listed as stub. Fixed: marked as REAL with description.

6. **researcher-codebase/bolt-message-pattern-map.md** — `SpawnAdditionalBolt` section described it as defined in messages.rs (removed). Phantom bolt section described old `PhantomTimer`/`Transform` approach (superseded). Fixed: rewrote both sections to reflect current state.

7. **researcher-codebase/MEMORY.md** — description for bolt-message-pattern-map.md still said "SpawnAdditionalBolt placeholder status". Fixed: updated description.

8. **reviewer-quality/phase5-complex-effects-patterns.md** — `kill_count` vocabulary note described field that was renamed to `cells_destroyed`. Fixed: updated note to reflect correct field name.

9. **reviewer-quality/phase4-runtime-effects-patterns.md** — `PhantomTimer kept as backward-compatibility stub` — PhantomTimer no longer exists. Fixed: updated to note it was removed.

10. **reviewer-quality/MEMORY.md** — description for phase4 file mentioned "PhantomTimer stub". Fixed: updated description.

11. **reviewer-performance/phase5-complex-effects.md** — Entity scale listed `ChainLightningRequest` (old model, gone). Fixed: replaced with ChainLightningChain/Arc.

12. **reviewer-performance/phase3-stat-effects.md** — recalculate systems described as "Wave 6 placeholder" — now real. Fixed: added clarifying note.

13. **guard-docs/known-state.md** — SpawnAdditionalBolt section said "Intentional Dead Registration" but message is fully removed. Fixed: updated to "REMOVED".

14. **guard-docs/terminology.md** — said SpawnAdditionalBolt "not actively used" — it's removed entirely. Fixed: updated framing.

15. **reviewer-bevy-api/confirmed-patterns.md** — Position Source section said both chain_lightning AND piercing_beam use Transform (wrong). chain_lightning was fixed in rework. Fixed: separated the two, marked chain_lightning as fixed.

16. **reviewer-correctness/bug-patterns.md** — Transform vs Position2D section listed both files as broken. Fixed: separated, marked chain_lightning as fixed.

17. **reviewer-performance/phase5-complex-effects.md** — fire() description still described old "Position2D -> Transform fallback" for chain_lightning. Fixed: updated to reflect Position2D-only fallback.

18. **reviewer-correctness/known-correct-effects.md** — `arc_speed == 0.0` stuck chain bug listed as OPEN. Fixed in rework (arc_speed <= 0 guard at line 82). Fixed: updated to FIXED status.

19. **researcher-codebase/effect-domain-inventory.md** — recalculate columns still said "placeholder body"; ShieldActive had wrong fields; Attraction said "placeholder bodies"; SpawnPhantom duplicate row; TimePenalty/RandomEffect listed as placeholders (both real). Fixed: all corrected.

**Stale patterns confirmed still open:**
- gravity_well.rs still lacks CleanupOnNodeExit (reviewer-architecture/known_gap_cleanup_markers.md — correct)
- chain_lightning/piercing_beam Transform vs Position2D bug still open (reviewer-bevy-api/confirmed-patterns.md — correct)
- 1-frame stale ordering gap for Effective* consumers still open (reviewer-correctness/bug-patterns.md — correct)

## Full Audit — 2026-03-28 (post Phase 4+5 effect implementation)

**Scope:** researcher-codebase, reviewer-tests, reviewer-correctness, reviewer-quality, reviewer-performance, reviewer-architecture, runner-linting

**Issues found and fixed (7):**

1. **researcher-codebase/MEMORY.md** — `effect-trigger-design-inventory.md` existed but was not linked. Fixed: added link.

2. **researcher-codebase/effect-domain-inventory.md** — Pulse test count was "0" but Phase 4/5 added 18 tests. Fixed: updated test count and component list for Pulse.

3. **researcher-codebase/bolt-spawn-component-map.md** — Referenced `SpawnChainBolt` in `messages.rs` (removed) and described `chain_bolt.rs::fire()` as spawning marker-only entities (superseded by real bolt spawn). Fixed: removed stale SpawnChainBolt section, updated to describe direct full-bolt spawn pattern. Split file into `bolt-spawn-component-map.md` (components/CCD) and `bolt-message-pattern-map.md` (messages/patterns) at 174 lines.

4. **reviewer-architecture/known_gap_cleanup_markers.md** — Listed `shockwave.rs` as lacking `CleanupOnNodeExit`, but shockwave now has it at line 57. Fixed: moved shockwave to "correctly handled" list, kept gravity_well as still open.

5. **runner-linting/MEMORY.md** — Description said "0 errors/~59 warnings" but `lint_state_current.md` says "9 errors, 56+ warnings". Fixed: corrected description.

6. **runner-linting/lint_patterns_phase1b.md** — Historical snapshot (28 doc_markdown errors from cleanup_cell.rs, fixed long ago) in a stable file, with warning patterns duplicating `lint_patterns_core.md`. Replaced with tombstone comment. Added missing link to `lint_patterns_core.md` in MEMORY.md.

7. **reviewer-correctness/known-correct.md** — 165 lines (over 150 threshold). Split into `known-correct.md` (Phase 1 collision) and `known-correct-effects.md` (Phase 3–5 effects). MEMORY.md updated.

**Stale commit references:** Two frontmatter `description` fields in researcher-codebase referenced specific commits (35c10d1). Updated to phase references.

## Recurring Staleness Patterns

- **File-split refactors**: reviewer-file-length phase findings go stale after any commit that splits multiple files. After any "refactor: split" commit, verify HIGH/MEDIUM open lists immediately. This is the highest-velocity staleness source for file-length memory.
- **Effect domain evolves fast**: researcher-codebase effect inventory files go stale after each phase. After any phase touching `effect/effects/*.rs` or `effect/triggers/*.rs`, verify: test counts, placeholder status, component lists, and message references.
- **Bug status drift**: reviewer-correctness/bug-patterns.md OPEN items should be verified against the codebase each audit — bugs get fixed but the memory lags. Cross-check with known-correct-effects.md which often records the FIXED status first.
- **Trigger bridge stub → real transitions**: All trigger bridges moved from stubs to real implementations in the feature/source-chip-shield-absorption branch. Any future audit noting "stubs" for bump/bumped/impact/impacted/node_start/node_end/cell_destroyed should verify against the codebase — they are real.
- **lint_patterns_core.md description drift**: This file describes historical warning patterns from effect system stub phases. After any major cleanup round, verify the "current" vs "historical" sections are accurate.
- **Cleanup marker gaps**: reviewer-architecture tracks which effect entities lack `CleanupOnNodeExit`. Check this file after any new effect that spawns entities.
- **runner-linting state**: `lint_state_current.md` is a session snapshot — its MEMORY.md description drifts if the description is updated without updating the file, or vice versa. After any lint-fixing session, verify both match.
- **Message removals**: When messages are removed from `messages.rs`, any memory referencing them goes stale instantly. researcher-codebase bolt-message-pattern-map.md and guard-docs/known-state.md are the primary risk areas.
- **Component renames and removals**: When components like PhantomTimer are removed from the codebase, multiple memory files across reviewer-quality and researcher-codebase reference them. grep for the old name before auditing.
- **Placeholder-to-real transitions**: Each effect phase converts placeholders to real implementations. The inventory (`effect-domain-inventory.md`) must be updated at each phase; so must reviewer-performance phase files that note entity scale and run_if gaps.
- **EntropyEngineState field names**: The field was renamed from `kill_count` to `cells_destroyed`. Watch for similar renames in game-vocabulary cleanup sessions.
- **guard-file-length vs reviewer-file-length duplication**: guard-file-length should NOT maintain per-phase findings; those belong in reviewer-file-length. If guard-file-length/ephemeral/ contains phase findings files, they are duplicates.

## Agents Accumulating Memory Fastest

1. **reviewer-file-length** — Highest staleness risk after large refactors. phase findings go stale every time a split commit lands.
2. **researcher-codebase** — Effect domain changes every phase; inventory goes stale.
3. **reviewer-correctness** — bug-patterns.md OPEN items lag behind fixes; cross-check known-correct-effects.md.
4. **runner-linting** — lint_state_current.md is point-in-time; description drift is likely.
5. **reviewer-architecture** — known_gap_cleanup_markers.md tracks open gaps that close over time.
