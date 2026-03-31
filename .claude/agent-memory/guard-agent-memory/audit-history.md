---
name: Agent memory audit history
description: Recurring staleness patterns and duplication issues found in agent memory audits — use to direct future audits
type: project
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

## Agents Accumulating Memory Fastest

1. **reviewer-file-length** — Highest staleness risk after large refactors. Phase findings go stale every time a split commit lands.
2. **researcher-codebase** — Effect domain changes every phase; inventory goes stale.
3. **reviewer-correctness** — bug-patterns.md OPEN items lag behind fixes; cross-check known-correct-effects.md.
4. **runner-linting** — lint_state_current.md is point-in-time; description drift is likely.
5. **reviewer-architecture** — known_gap_cleanup_markers.md tracks open gaps that close over time.
