---
name: Audit History
description: When full audits were run, what scope, and what was found — helps identify which agents accumulate stale memory fastest
type: project
---

## 2026-04-07 — Full Audit (feature/scenario-runner-wiring pre-merge gate)

**Scope:** All 26 agent memory directories.
**Total stable files:** ~120 across all agents.
**Issues found:** 6 (all fixed).

### Fixes Applied

1. **writer-scenarios/pattern_effect_ron_syntax.md** — duplicate of `effect_ron_syntax.md` (shorter/incomplete version). Tombstoned with redirect to canonical file. MEMORY.md updated to remove link.

2. **guard-file-length/split-patterns.md** — stale note about `types/mod.rs` being a 560-line violation (already fixed in earlier branch). Updated to remove the stale item.

3. **guard-file-length/split-patterns.md** — stale cross-reference to `phase4_findings.md` as current; updated to point to `phase7_findings.md`.

4. **reviewer-file-length/phase7_findings.md** — all 11 MEDIUM actionable files confirmed split in this branch. Updated description and content to reflect resolved status.

5. **reviewer-file-length/MEMORY.md** — phase5 and phase6 findings were superseded but still indexed. Moved to Archived section in MEMORY.md (not indexed as current).

6. **runner-scenarios/stable_entered_playing_reset.md** — trimmed from incident narrative to pattern guidance (the fix is confirmed; stable value is the forward-looking pattern).

### Staleness Patterns Observed

- **reviewer-file-length** accumulates stale findings fastest — each wave of file splits obsoletes the prior wave's MEDIUM actionable list. Phase-numbered files (phase3–7) pile up. Pattern: keep only current wave in main index; archive others.
- **writer-scenarios** had an internal duplicate (effect_ron_syntax.md vs pattern_effect_ron_syntax.md) — likely from different agents writing to the same memory dir at different times.
- **guard-file-length** cross-references reviewer-file-length but its cross-reference pointer was stale (pointing to phase4 instead of phase7).

### Agents with No Issues Found

orchestrator, runner-scenarios (stable_layout_name_casing), reviewer-scenarios (all 6 files), reviewer-tests (all 7 files), reviewer-architecture (all 9 files), reviewer-correctness, reviewer-performance (all 17 files), reviewer-quality, reviewer-bevy-api, researcher-bevy-api, researcher-codebase, researcher-rust-idioms, researcher-system-dependencies, researcher-crates, guard-game-design, guard-dependencies, guard-docs, guard-security, writer-scenarios (other files), writer-code, writer-tests, planner-spec, planner-review, runner-linting.
