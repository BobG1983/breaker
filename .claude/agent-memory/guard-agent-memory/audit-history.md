---
name: Audit History
description: When full audits were run, what scope, and what was found — helps identify which agents accumulate stale memory fastest
type: project
---

## 2026-04-08 — Full Audit (feature/bolt-birthing-animation pre-merge gate)

**Scope:** All 27 agent memory directories.
**Total stable files:** ~130 across all agents.
**Issues found:** 7 (all fixed).

### Fixes Applied

1. **guard-file-length/MEMORY.md** — stale cross-reference pointing to `phase7_findings.md` as current; updated to `phase8_findings.md` (Wave 12, 2026-04-07).

2. **researcher-bevy-api/confirmed-patterns.md** — 1150 lines; split into 7 focused files: `api-time-and-virtual.md`, `api-message-system.md`, `api-query-data.md`, `api-states.md`, `api-world-and-bundle.md`, `api-run-conditions.md`, `api-ui-and-rendering.md`. Original tombstoned with redirect. MEMORY.md updated.

3. **reviewer-correctness/bug-patterns.md** — 351 lines; split into `bug-patterns-open.md` (OPEN/LATENT bugs) and `bug-patterns-resolved.md` (FIXED/CONFIRMED patterns). Original tombstoned. MEMORY.md updated.

4. **guard-docs/known-state.md** — 420 lines; split into `known-state.md` (recent sessions only, ~90 lines) and `known-state-history.md` (older phases 2026-03-28 through 2026-04-06). MEMORY.md updated.

5. **researcher-codebase/MEMORY.md** — two new files (`bolt-lost-respawn-gap.md`, `time-virtual-pausing-hazard.md`) were new/untracked in git but already correctly indexed in MEMORY.md. No fix needed.

6. **researcher-crates/MEMORY.md** — new file `iyes_progress_integration.md` was already correctly indexed in MEMORY.md. No fix needed.

7. **reviewer-bevy-api/MEMORY.md** — was missing the new split files; rebuilt to index all 8 api-*.md files.

### Staleness Patterns Observed

- **researcher-bevy-api/confirmed-patterns.md** is the most rapidly-growing file. Each new API discovery adds to it indefinitely. Pattern: split by topic proactively, not reactively.
- **guard-docs/known-state.md** accumulates one section per feature branch. Pattern: keep only the most recent 3 sessions + standing facts; archive the rest to known-state-history.md.
- **reviewer-correctness/bug-patterns.md** mixed OPEN and RESOLVED bugs making it hard to scan. Pattern: separate open from resolved to avoid double-checking already-fixed issues.
- **guard-file-length/MEMORY.md** cross-reference goes stale every phase boundary (phase3→4→5→6→7→8). Pattern: always update guard-file-length cross-reference when a new phase8+ file is created.

### Agents with No Issues Found

orchestrator, planner-spec, planner-review, writer-code, writer-tests, researcher-system-dependencies, researcher-rust-idioms, reviewer-architecture, reviewer-performance, reviewer-quality, reviewer-scenarios, reviewer-tests, reviewer-file-length, runner-linting, runner-scenarios, writer-scenarios, guard-security, guard-dependencies, guard-game-design, guard-file-length (after fix).

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
