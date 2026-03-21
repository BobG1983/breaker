---
name: Recurring Audit Patterns
description: Staleness patterns and cross-agent duplication patterns observed across audits
type: reference
---

## Last Full Audit
2026-03-19 (run 1-2) — 26 agent directories each. Run 1: 10 issues, 10 fixed. Run 2: 8 issues, 8 fixed.
2026-03-19 (run 3) — 26 directories. 6 issues found, 6 fixed. Key: stale cast-pattern memories, orphaned writer-tests file, entity_scale duplication, planner-review stale method name.
2026-03-20 (targeted: session 7 overclock-trigger-chain changes) — 12 directories audited. 20 issues found, 20 fixed. Key: ChipKind removal, OverclockEffectFired.bolt Entity→Option<Entity>, TriggerChain stacking fields, shockwave violation resolution (DamageCell pattern), handle_cell_hit consumer migration from BoltHitCell to DamageCell, performance-baseline stale cell query.

## Recurring Staleness Patterns

### Phase completion drift
Agents that track plugin stub status drift when plugins gain systems between audit cycles.
Observed: `researcher-system-dependencies/architectural-facts.md` still listed `UpgradesPlugin` (removed) and called `ChipsPlugin` a stub (now fully implemented).
**Watch:** researcher-system-dependencies and reviewer-architecture — they maintain system inventories that become stale as phases complete.

### Message struct field add/remove drift
When a message struct gains or loses fields (e.g., `BoltHitCell.bolt: Entity` removed in one branch then re-added in another), multiple agents remember the old struct shape.
Observed: `architectural-facts.md` and `message-flow.md` both said `BoltHitCell` had no `bolt` field — but the field was re-added in feature/phase4b2-effect-consumption. Fixed in 2026-03-19 audit.
**Watch:** researcher-system-dependencies (message-flow.md, architectural-facts.md) and planner-spec/domain-inventory.md — they describe message struct shapes.

### Observer event file location drift
When observer trigger events move between files (e.g., `chips/messages.rs` → `chips/definition.rs`), multiple agents reference the old file path.
Observed: `guard-docs/known-state.md` referenced `chips/messages.rs` after `ChipEffectApplied` was moved to `definition.rs`.
**Watch:** guard-docs/known-state.md — it tracks architecture-level file locations.

### Message consumer "future" labels persist after implementation
Messages listed as "no active gameplay receivers (future phases)" remain labeled that way even after consumers are added.
Observed: `ChipSelected` in message-inventory, message-flow, architectural-facts, and scenario-patterns all said "no receiver" after `apply_chip_effect` was added. Fixed in bug-patterns.md and architectural-facts.md in 2026-03-19 audit.
**Watch:** Any message row marked "(future: X)" — verify against current code on each audit.
**Next candidates to watch:** BoltHitCell `(future: upgrades, audio)` in reviewer-architecture/message-inventory.md.

### ChaosMonkey → ChaosDriver rename drift (CLOSED 2026-03-19)
Multiple agents still referenced the old `ChaosMonkey` name after the rename to `ChaosDriver`.
Observed: writer-code and writer-tests pattern_doc_markdown_clippy.md — both fixed.
Also fixed: researcher-system-dependencies/architectural-facts.md, known-conflicts.md.
**Pattern:** When a type is renamed, grep all agent memories for the old name.

### Type migration leaves stale implementation memories
When a value type migrates (e.g., `BASE_BOLT_DAMAGE: u32 → f32`, damage arithmetic u32→f32), memories that describe the OLD type's cast/conversion patterns persist even after the new type makes those patterns irrelevant.
Observed: `researcher-rust-idioms/pattern_f32_to_u32_damage_cast.md` and `researcher-bevy-api/numeric_cast_patterns.md` both described a u32/u16/f64 cast pattern for damage that no longer exists — `CellHealth.take_damage()` uses f32 directly, no casts needed.
**Watch:** researcher-rust-idioms and researcher-bevy-api — they record patterns tied to specific codebase implementations that may become stale after type migrations.
**Action taken:** Retired both files (MEMORY.md links removed; pattern_f32_to_u32_damage_cast.md marked stale; numeric_cast_patterns.md de-indexed).

### Active Violation resolution not propagated
When a BLOCKING violation is fixed (e.g., shockwave cross-domain mutation → DamageCell message pattern), three memory files hold stale "violation" language: the reviewer-architecture/compromises.md Active Violations list, the reviewer-architecture/project-state.md audit entry, and the reviewer-correctness/bug-patterns.md bug entry.
**Watch:** Any entry marked "BLOCKING", "VIOLATION", or "pending fix" — verify against current code on each audit. Also check known-correct.md for stale correctness notes that referenced the old (broken) implementation.

### Event/message field type changes cascade to multiple memories
When a field type changes (e.g., `OverclockEffectFired.bolt: Entity → Option<Entity>`), all memories that describe usage patterns for that field become stale: domain-inventory.md (field description), known-correct.md (usage correctness notes), bug-patterns.md (bugs that depended on the old type), and message-flow/message-inventory tables.
**Watch:** planner-spec/domain-inventory.md, reviewer-correctness/known-correct.md, reviewer-correctness/bug-patterns.md, reviewer-architecture/message-inventory.md, researcher-system-dependencies/message-flow.md — all describe message/event struct shapes.

### Pending-migration language outlasts the migration
When a component type is migrated (e.g., `CellHealth: u32 → f32`), domain inventory files describe both the "before" and "after" states using "After migration:" language. The pending language persists after the migration ships.
Observed: `planner-spec/domain-inventory.md` still described `CellHealth { current: u32, max: u32 }` with the pending f32 form as "After migration:". Fixed in 2026-03-19 audit.
**Watch:** planner-spec/domain-inventory.md — updated per-phase-session with explicit pending notes.

## Recurring Cross-Agent Duplication Patterns

### pattern_doc_markdown_clippy.md duplicated in writer-code and writer-tests
Both agents own nearly identical files. This is intentional (both write doc comments) but the files can drift independently.
**Accepted duplication** — both agents need it. Ensure both are updated together when clippy rules change.

### Performance intentional patterns scattered
guard-performance/MEMORY.md had 22 inline bullet points of "Known Intentional Patterns" in the index file (not a stable file). Extracted to `intentional-patterns.md`.
**Watch:** guard-performance MEMORY.md — tendency to accumulate inline content.

### entity_scale.md duplicated by calibration.md (RESOLVED 2026-03-19)
`guard-performance/entity_scale.md` content was a strict subset of `calibration.md`. Removed `entity_scale.md` from the MEMORY.md index; `calibration.md` is now the sole calibration reference.

## Oversized Files (>150 lines) — Do Not Split Without Cause
These files are legitimately large due to the scope of their content:
- `researcher-system-dependencies/system-map.md` (~510 lines) — full system inventory, consulted as a whole
- `researcher-system-dependencies/known-conflicts.md` (~470 lines) — full conflict history, consulted as a whole
- `researcher-system-dependencies/message-flow.md` (~226 lines) — complete message flow map
- `reviewer-bevy-api/confirmed-patterns.md` (~250 lines) — complete API reference
- `reviewer-correctness/known-correct.md` (~154 lines) — **at split threshold**; consider splitting by domain (physics/gameplay correctness vs scenario runner correctness) if it continues to grow
- `reviewer-correctness/scenario-patterns.md` (~130 lines) — approaching split threshold
- `runner-scenarios/known_invariant_false_positives.md` (~201 lines) — accepted; each false positive entry is a detailed root-cause analysis that must be read together with others

Split these only if the content becomes domain-segmentable (e.g., scenario runner vs gameplay systems).
