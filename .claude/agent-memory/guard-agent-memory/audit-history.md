---
name: Agent memory audit history
description: Recurring staleness patterns and duplication issues found in agent memory audits — use to direct future audits
type: project
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

**Stale patterns confirmed still open:**
- gravity_well::fire() infinite loop when max == 0 (reviewer-correctness/bug-patterns.md)
- 1-frame stale ordering for Effective* consumers (bolt/plugin.rs)
- BASE_BOLT_DAMAGE hardcoding in combat effects (guard-game-design blocker 1)
- chip-catalog doc drift (guard-game-design blocker 2)
- HIGH file-length: check_aabb_matches_entity_dimensions.rs (581 lines)
- MEDIUM file-length items: 25+ files in reviewer-file-length/phase4_findings.md

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

## Agents Accumulating Memory Fastest

1. **reviewer-file-length** — Highest staleness risk after large refactors. Phase findings go stale every time a split commit lands.
2. **researcher-codebase** — Effect domain changes every phase; inventory goes stale.
3. **reviewer-correctness** — bug-patterns.md OPEN items lag behind fixes; cross-check known-correct-effects.md.
4. **runner-linting** — lint_state_current.md is point-in-time; description drift is likely.
5. **reviewer-architecture** — known_gap_cleanup_markers.md tracks open gaps that close over time.
