---
name: Recurring Audit Patterns
description: Staleness patterns and cross-agent duplication patterns observed across audits
type: reference
---

## Last Full Audit
2026-03-19 — 26 agent directories audited, 10 issues found, 10 fixed.

## Recurring Staleness Patterns

### Phase completion drift
Agents that track plugin stub status drift when plugins gain systems between audit cycles.
Observed: `researcher-system-dependencies/architectural-facts.md` still listed `UpgradesPlugin` (removed) and called `ChipsPlugin` a stub (now fully implemented).
**Watch:** researcher-system-dependencies and reviewer-architecture — they maintain system inventories that become stale as phases complete.

### Observer event file location drift
When observer trigger events move between files (e.g., `chips/messages.rs` → `chips/definition.rs`), multiple agents reference the old file path.
Observed: `guard-docs/known-state.md` referenced `chips/messages.rs` after `ChipEffectApplied` was moved to `definition.rs`.
**Watch:** guard-docs/known-state.md — it tracks architecture-level file locations.

### Message consumer "future" labels persist after implementation
Messages listed as "no active gameplay receivers (future phases)" remain labeled that way even after consumers are added.
Observed: `ChipSelected` in message-inventory, message-flow, architectural-facts, and scenario-patterns all said "no receiver" after `apply_chip_effect` was added.
**Watch:** Any message row marked "(future: X)" — verify against current code on each audit.

### ChaosMonkey → ChaosDriver rename drift
Multiple agents still referenced the old `ChaosMonkey` name after the rename to `ChaosDriver`.
Observed: writer-code and writer-tests pattern_doc_markdown_clippy.md.
**Pattern:** When a type is renamed, grep all agent memories for the old name.

## Recurring Cross-Agent Duplication Patterns

### pattern_doc_markdown_clippy.md duplicated in writer-code and writer-tests
Both agents own nearly identical files. This is intentional (both write doc comments) but the files can drift independently.
**Accepted duplication** — both agents need it. Ensure both are updated together when clippy rules change.

### Performance intentional patterns scattered
guard-performance/MEMORY.md had 22 inline bullet points of "Known Intentional Patterns" in the index file (not a stable file). Extracted to `intentional-patterns.md`.
**Watch:** guard-performance MEMORY.md — tendency to accumulate inline content.

## Oversized Files (>150 lines) — Do Not Split Without Cause
These files are legitimately large due to the scope of their content:
- `researcher-system-dependencies/system-map.md` (~510 lines) — full system inventory, consulted as a whole
- `researcher-system-dependencies/known-conflicts.md` (~470 lines) — full conflict history, consulted as a whole
- `researcher-system-dependencies/message-flow.md` (~226 lines) — complete message flow map
- `reviewer-bevy-api/confirmed-patterns.md` (~250 lines) — complete API reference
- `reviewer-correctness/known-correct.md` (~93 lines) — approaching warning threshold
- `reviewer-correctness/scenario-patterns.md` (~130 lines) — approaching split threshold

Split these only if the content becomes domain-segmentable (e.g., scenario runner vs gameplay systems).
