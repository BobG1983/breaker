---
name: Agent memory audit history — archive
description: Older audit records (pre-develop-2026-03-30-final); see audit-history.md for recent audits and recurring patterns
type: project
---

## Full Audit — 2026-03-30 (post feature/full-verification-fixes merge)

**Scope:** All agent memory directories (full audit)

**Context:** feature/full-verification-fixes branch fixed Transform/Position2D violations in gravity_well, shockwave, explode, pulse, and piercing_beam. It also added CleanupOnNodeExit to gravity_well.

**Issues found and fixed (14):**

1. reviewer-architecture/known_gap_cleanup_markers.md — gravity_well updated to correctly handled.
2. reviewer-architecture/MEMORY.md — cleanup markers description updated.
3. reviewer-architecture/known_gap_transform_usage.md — all violations fixed; only chain_lightning arc_transforms remains (correct).
4. reviewer-architecture/MEMORY.md — transform usage description updated.
5. reviewer-correctness/known-correct-effects.md — dispatch_chip_effects stale WIP framing removed.
6. reviewer-bevy-api/confirmed-patterns.md — stale "piercing_beam Transform fallback still open" line removed.
7-8. runner-scenarios/MEMORY.md + all_passing_2026_03_30.md — scenario count fixed (103, 86 named, 17 stress).
9. runner-tests tombstone files — overwritten with minimal deletion markers.
10. reviewer-architecture/known_gap_effects_mod_production_logic.md — helpers.rs → fire_helpers.rs, RESOLVED.
11. reviewer-correctness/MEMORY.md — added Session History convention line.
12. reviewer-file-length/phase4_findings.md — rewrote HIGH/mod.rs sections to SPLIT/FIXED; preserved 25 MEDIUM items.
13-14. (same as 7-8 above, duplicated entry in original)

---

## Full Audit — 2026-03-30 (Full Verification Tier, develop post-merge c9964b7)

**Scope:** All agent memory directories (full audit)

**Context:** Refactor commit c9964b7 split 23 oversized .rs files into directory modules. Three bugs in reviewer-correctness/bug-patterns.md were actually FIXED in feature/source-chip-shield-absorption.

**Issues found and fixed (18):**

1. runner-tests tombstone files — tombstoned, removed from MEMORY.md.
2. runner-tests tombstone files — same.
3. reviewer-correctness/bug-patterns.md — 3 OPEN bugs marked FIXED (dispatch_chip_effects max-stacks, bypass_menu PendingBreakerEffects, apply_pending_bolt_effects).
4. reviewer-correctness/known-correct-effects.md — kill_count → cells_destroyed.
5. reviewer-quality/MEMORY.md — kill_count description updated.
6. guard-security/vetted_dependencies.md — proptest removal noted.
7. researcher-bevy-api/MEMORY.md — empty; added confirmed-patterns.md link.
8. guard-performance/MEMORY.md — empty; added Session History.
9. researcher-crates/MEMORY.md — empty; added Session History.
10. researcher-rust-errors/MEMORY.md — empty; added Session History.
11. researcher-rust-idioms/MEMORY.md — empty; added Session History.
12. researcher-system-dependencies/MEMORY.md — empty; added Session History.
13. runner-release/MEMORY.md — empty; added Session History.
14. reviewer-file-length/phase4_findings.md — rewrote entirely (all HIGH/MEDIUM stale post-c9964b7).
15. reviewer-file-length/phase3_findings.md — converted to archived reference.
16. guard-file-length/split-patterns.md — dispatch_breaker_effects split noted as resolved.
17. guard-file-length/ephemeral/ — duplicate phase findings files tombstoned.
18. guard-file-length/MEMORY.md — added cross-reference to reviewer-file-length.

---

## Targeted Audit — 2026-03-29 (pre-merge gate, feature/source-chip-shield-absorption)

**Scope:** All agent memory directories (full audit)

**Issues found and fixed (9):**

1. runner-tests/MEMORY.md — empty; added link and Session History.
2. guard-architecture/MEMORY.md — completely empty; added Session History.
3. researcher-impact/MEMORY.md — header-only; added Session History.
4. runner-linting/lint_patterns_core.md — stale Wave 8 stub patterns; rewrote to reflect current state (4 dead_code warnings on source marker tuples).
5. runner-linting/MEMORY.md — stale description; updated.
6. researcher-codebase/effect-domain-inventory.md — all trigger bridges listed as stubs; all are real. Fixed.
7. researcher-codebase/effect-system-domain-map.md — bolt_lost listed as stub. Fixed.
8. researcher-quality/phase3-stat-effects-patterns.md — dispatch_breaker_effects described as stub; it's real. Fixed.
9. reviewer-architecture/pattern_effect_direct_spawn.md — SpawnChainBolt messages.md inconsistency noted as resolved. Fixed.

---

## Targeted Audit — 2026-03-29 (post source_chip threading, feature/runtime-effects)

**Scope:** orchestrator, reviewer-tests, reviewer-correctness, reviewer-quality, reviewer-bevy-api, reviewer-architecture, reviewer-performance, runner-linting, plus dependent researcher-codebase and guard-docs files

**Issues found and fixed (19):**

1. orchestrator/MEMORY.md — empty; added Session History.
2. reviewer-tests/MEMORY.md — missing Session History; added.
3. runner-linting/MEMORY.md — stale "2 ERRORS" description; fixed.
4. researcher-codebase/effect-domain-inventory.md — major update; all Phase 4/5/source_chip effects marked REAL; trigger bridges updated; recalculate note corrected.
5. researcher-codebase/effect-system-domain-map.md — bolt_lost stub → REAL.
6. researcher-codebase/bolt-message-pattern-map.md — SpawnAdditionalBolt and PhantomTimer sections rewritten.
7. researcher-codebase/MEMORY.md — bolt-message-pattern-map description updated.
8. reviewer-quality/phase5-complex-effects-patterns.md — kill_count → cells_destroyed.
9. reviewer-quality/phase4-runtime-effects-patterns.md — PhantomTimer stub note removed (PhantomTimer removed from codebase).
10. reviewer-quality/MEMORY.md — phase4 description updated.
11. reviewer-performance/phase5-complex-effects.md — ChainLightningRequest → ChainLightningChain/Arc.
12. reviewer-performance/phase3-stat-effects.md — recalculate "Wave 6 placeholder" note clarified.
13. guard-docs/known-state.md — SpawnAdditionalBolt "Intentional Dead Registration" → "REMOVED".
14. guard-docs/terminology.md — SpawnAdditionalBolt "not actively used" → removed entirely.
15. reviewer-bevy-api/confirmed-patterns.md — chain_lightning marked fixed (Position2D); piercing_beam still open.
16. reviewer-correctness/bug-patterns.md — chain_lightning marked fixed (Transform issue).
17. reviewer-performance/phase5-complex-effects.md — chain_lightning fire() description updated.
18. reviewer-correctness/known-correct-effects.md — arc_speed <= 0 stuck chain FIXED.
19. researcher-codebase/effect-domain-inventory.md — recalculate columns, ShieldActive fields, Attraction, SpawnPhantom all corrected.

---

## Full Audit — 2026-03-28 (post Phase 4+5 effect implementation)

**Scope:** researcher-codebase, reviewer-tests, reviewer-correctness, reviewer-quality, reviewer-performance, reviewer-architecture, runner-linting

**Issues found and fixed (7):**

1. researcher-codebase/MEMORY.md — effect-trigger-design-inventory.md not linked; added.
2. researcher-codebase/effect-domain-inventory.md — Pulse test count wrong; updated.
3. researcher-codebase/bolt-spawn-component-map.md — SpawnChainBolt removed; direct spawn pattern updated. Split into bolt-spawn-component-map.md + bolt-message-pattern-map.md.
4. reviewer-architecture/known_gap_cleanup_markers.md — shockwave now has CleanupOnNodeExit; updated.
5. runner-linting/MEMORY.md — "0 errors/~59 warnings" stale; corrected.
6. runner-linting/lint_patterns_phase1b.md — historical snapshot tombstoned; lint_patterns_core.md link added to MEMORY.md.
7. reviewer-correctness/known-correct.md — 165 lines; split into known-correct.md + known-correct-effects.md.
