---
name: Agent memory audit history
description: Recurring staleness patterns and duplication issues found in agent memory audits — use to direct future audits
type: project
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

- **Effect domain evolves fast**: researcher-codebase effect inventory files go stale after each phase. After any phase touching `effect/effects/*.rs`, verify: test counts, placeholder status, component lists, and message references.
- **Cleanup marker gaps**: reviewer-architecture tracks which effect entities lack `CleanupOnNodeExit`. Check this file after any new effect that spawns entities.
- **runner-linting state**: `lint_state_current.md` is a session snapshot — its MEMORY.md description drifts if the description is updated without updating the file, or vice versa. After any lint-fixing session, verify both match.
- **Message removals**: When messages are removed from `messages.rs`, any memory referencing them goes stale instantly. researcher-codebase bolt-message-pattern-map.md and guard-docs/known-state.md are the primary risk areas.
- **Component renames and removals**: When components like PhantomTimer are removed from the codebase, multiple memory files across reviewer-quality and researcher-codebase reference them. grep for the old name before auditing.
- **Placeholder-to-real transitions**: Each effect phase converts placeholders to real implementations. The inventory (`effect-domain-inventory.md`) must be updated at each phase; so must reviewer-performance phase files that note entity scale and run_if gaps.
- **EntropyEngineState field names**: The field was renamed from `kill_count` to `cells_destroyed`. Watch for similar renames in game-vocabulary cleanup sessions.

## Agents Accumulating Memory Fastest

1. **researcher-codebase** — Highest staleness risk. Effect domain changes every phase.
2. **runner-linting** — lint_state_current.md is point-in-time; description drift is likely.
3. **reviewer-architecture** — known_gap_cleanup_markers.md tracks open gaps that close over time.
4. **reviewer-correctness** — known-correct.md grows with each phase and may exceed size limits.
