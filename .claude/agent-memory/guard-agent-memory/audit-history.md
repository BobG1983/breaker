---
name: Agent memory audit history
description: Recurring staleness patterns and duplication issues found in agent memory audits — use to direct future audits
type: project
---

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

- **Effect domain evolves fast**: researcher-codebase effect inventory files go stale after each phase. After any phase touching `effect/effects/*.rs`, verify: test counts, placeholder status, component lists, and message references.
- **Cleanup marker gaps**: reviewer-architecture tracks which effect entities lack `CleanupOnNodeExit`. Check this file after any new effect that spawns entities.
- **runner-linting state**: `lint_state_current.md` is a session snapshot — its MEMORY.md description drifts if the description is updated without updating the file, or vice versa. After any lint-fixing session, verify both match.
- **Message removals**: When messages are removed from `messages.rs`, any memory referencing them goes stale instantly. researcher-codebase bolt-message-pattern-map.md is the primary risk area.

## Agents Accumulating Memory Fastest

1. **researcher-codebase** — Highest staleness risk. Effect domain changes every phase.
2. **runner-linting** — lint_state_current.md is point-in-time; description drift is likely.
3. **reviewer-architecture** — known_gap_cleanup_markers.md tracks open gaps that close over time.
4. **reviewer-correctness** — known-correct.md grows with each phase and may exceed size limits.
