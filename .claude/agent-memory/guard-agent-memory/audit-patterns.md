---
name: Recurring Audit Patterns
description: Staleness patterns and cross-agent duplication patterns observed across audits
type: reference
---

## Last Full Audit
2026-03-19 (run 1-2) — 26 agent directories each. Run 1: 10 issues, 10 fixed. Run 2: 8 issues, 8 fixed.
2026-03-19 (run 3) — 26 directories. 6 issues found, 6 fixed. Key: stale cast-pattern memories, orphaned writer-tests file, entity_scale duplication, planner-review stale method name.
2026-03-20 (targeted: session 7 overclock-trigger-chain changes) — 12 directories audited. 20 issues found, 20 fixed. Key: ChipKind removal, OverclockEffectFired.bolt Entity→Option<Entity>, TriggerChain stacking fields, shockwave violation resolution (DamageCell pattern), handle_cell_hit consumer migration from BoltHitCell to DamageCell, performance-baseline stale cell query.
2026-03-21 (targeted: refactor/unify-behaviors) — 15 directories audited. 40+ stale references fixed across 17 files. Key changes: bolt/behaviors/ subdomain DELETED, behaviors/consequences/ DELETED, ActiveOverclocks→ActiveChains, OverclockEffectFired→EffectFired, OverclockTriggerKind→TriggerKind, BoltBehaviorsPlugin DELETED, all bridge/effect systems consolidated in behaviors/. Agents affected: planner-spec, researcher-system-dependencies (3 files), reviewer-architecture (5 files), reviewer-correctness (2 files), reviewer-quality, reviewer-performance, reviewer-bevy-api, runner-linting, guard-docs (3 files), guard-game-design.
2026-03-21 (targeted: SpeedBoost generalization / apply_bump_velocity deletion) — 16 files across 9 agent directories. 28 stale references fixed. Key changes: TriggerChain::BoltSpeedBoost→SpeedBoost{target,multiplier}, apply_bump_velocity DELETED, BumpPerformed.multiplier field REMOVED, BumpPerfectMultiplier/BumpWeakMultiplier components DELETED. Disambiguation required: BoltSpeedBoost(f32) Amp chip component (chips/components.rs) still exists — required careful per-reference judgment rather than blanket rename. See ephemeral/audit-2026-03-21-speedboost.md.
2026-03-22 (full phase-boundary: Wave 3 of Phase 4) — 27 directories audited. 50+ stale references fixed across 20+ files. Key changes: GameState::NodeTransition DELETED (→ TransitionOut + TransitionIn), upgrade_select/ RENAMED to chip_select/, bridge_overclock_* system names removed (_overclock_ infix no longer exists — functions are bridge_bolt_lost, bridge_bump, etc.), OfferingNoDuplicates + MaxedChipNeverOffered invariants added. Agents affected: reviewer-correctness (3 files), reviewer-architecture (5 files), researcher-system-dependencies (4 files), reviewer-performance, reviewer-quality (2 files), reviewer-bevy-api, guard-docs, guard-game-design, guard-agent-memory.
2026-03-23 (full phase-boundary: Wave 4 of Phase 4) — 28 directories audited. 7 stale references fixed across 5 files. Key changes: RunStats + HighlightTracker + 8 stat-tracking systems now registered in RunPlugin (bug-patterns.md had all 4 as OPEN); spawn_chip_select now reads Res<ChipOffers> (bug fixed); advance_node no longer sets NextState(Playing) (state-machine.md and bug-patterns.md updated); ChipInventory::add_chip() IS called in production at apply_chip_effect.rs:32 (chip-select-flow.md corrected); performance calibration updated to reflect 39+ scenarios. Agents affected: reviewer-correctness (bug-patterns.md, state-machine.md), researcher-codebase (chip-select-flow.md), guard-performance (calibration.md), researcher-system-dependencies (system-map.md).
2026-03-23 (full phase-boundary: memorable moments wave) — 28 directories audited. 7 issues fixed across 5 files. Key changes: HighlightKind 6→15 variants, HighlightTriggered message added, HighlightConfig RON pipeline documented, 5 detection systems documented, spawn_highlight_text wiring gap noted. Agents affected: guard-docs/known-state.md (highlights section rewritten), guard-docs/phase-log.md (new wave entry), reviewer-architecture/message-inventory.md (HighlightTriggered added), researcher-system-dependencies/message-flow.md (HighlightTriggered added), reviewer-correctness/bug-patterns.md (stale MassDestruction/FirstEvolution bug resolved), planner-review/MEMORY.md (pattern_onenter_deferred_resource_chain.md indexed).

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

### Domain deletion / consolidation cascades across all memory files
When an entire subdomain is deleted and its systems consolidated elsewhere (e.g., bolt/behaviors/ deleted, behaviors/consequences/ deleted, all content merged into behaviors/), every agent that recorded file paths, type names, or system names from the deleted domain holds stale data. This is the highest-impact staleness event — it touches file paths, type names, bridge system names, and plugin registration in every agent that covered the deleted domain.
Observed: refactor/unify-behaviors (2026-03-21) deleted bolt/behaviors/ and behaviors/consequences/, renaming 3 types (ActiveOverclocks→ActiveChains, OverclockEffectFired→EffectFired, OverclockTriggerKind→TriggerKind) and deleting 1 plugin (BoltBehaviorsPlugin). This produced stale references in 15 agent memories requiring 40+ individual edits.
**Watch:** When any subdomain is deleted, immediately audit: planner-spec/domain-inventory.md, researcher-system-dependencies (all 4 files), reviewer-architecture (all 5 files), reviewer-correctness, reviewer-quality, reviewer-performance, reviewer-bevy-api, runner-linting, guard-docs (terminology, known-state, phase-log), guard-game-design.
**Pattern:** Grep all agent memories for the old subdomain path (e.g., `bolt/behaviors`, `behaviors/consequences`) and old type names before closing the branch.

### Behavior-generalization refactors: same-name collision between deleted variant and surviving type
When a TriggerChain variant is renamed (e.g., `BoltSpeedBoost` → `SpeedBoost { target, multiplier }`) but another type with a nearly identical name survives (e.g., Amp chip component `BoltSpeedBoost(f32)` in `chips/components.rs`), memory files that reference the old variant name require case-by-case judgment. Blanket rename is wrong — some references are to the chip component (still correct) and some are to the deleted TriggerChain variant (stale).
Observed: refactor/unify-behaviors + SpeedBoost generalization (2026-03-21). Affected 9 agent directories, 16 files.
**Action:** When a variant is renamed, grep all agent memories for the old name, then for EACH hit read the surrounding context to determine which type is referenced before editing.
**Watch:** runner-linting/lint_patterns.md, reviewer-performance/performance-baseline.md, planner-spec/domain-inventory.md, reviewer-correctness/known-correct.md — all describe TriggerChain variant lists and may conflate variant names with chip component names.

### System deletion cascades to query descriptions in multiple memories
When a system is deleted (e.g., `apply_bump_velocity`), its query components remain documented in memories that describe the queries of RELATED systems (not just the deleted system's entry). Deleted system's components often appear in: domain inventory query alias tables, ordering chain tables, message consumer tables, and architectural facts system inventories.
Observed: apply_bump_velocity deletion (2026-03-21) — stale references in system-map.md (query aliases for adjacent systems), message-flow.md (consumer table), known-conflicts.md (ordering chain), architectural-facts.md (system inventory), ordering-chain.md (FixedUpdate chain table), message-inventory.md (consumer list), patterns.md (cross-domain read pattern), compromises.md (RESOLVED compromise), writer-tests pattern file (example source reference).
**Watch:** When any system is deleted, grep all agent memories for its name and check: (1) query alias tables — the deleted system's query components may appear in nearby system descriptions; (2) ordering chain tables — remove from before/after chains; (3) message consumer tables; (4) pattern example files that reference the deleted system as a reference implementation.

### Bridge system name suffix removal cascades to many memories
When bridge functions are renamed by removing a qualifier suffix (e.g., `bridge_overclock_*` → `bridge_*` during unification), ALL memories that documented the old names must be updated. The functions appear by name in: system-map.md (bridge section), message-flow.md (receiver lists and cross-plugin table), ordering-chain.md (FixedUpdate chain section), message-inventory.md (Observer Events table), performance-baseline.md (section headers), coverage-standards.md (gap lists), intentional-patterns.md (pattern notes), known-correct.md (correctness notes), bug-patterns.md (bug descriptions), confirmed-patterns.md (Bevy API patterns).
Observed: bridge_overclock_* → bridge_* rename in refactor/unify-behaviors (2026-03-21) was not fully propagated; missed in 10 memory files until Wave 3 audit (2026-03-22).
**Watch:** When bridge system functions are renamed, grep all agent memories for the old names before closing the branch.

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
- `reviewer-correctness/known-correct.md` (~174 lines, as of 2026-03-23) — **above split threshold**; consider splitting by domain (physics/gameplay correctness vs scenario runner correctness vs chip/behavior correctness) if it continues to grow
- `reviewer-correctness/scenario-patterns.md` (~130 lines) — approaching split threshold
- `runner-scenarios/known_invariant_false_positives.md` (~201 lines) — accepted; each false positive entry is a detailed root-cause analysis that must be read together with others
- `runner-linting/lint_patterns.md` (~122 lines, as of 2026-03-23 memorable moments wave) — warning range; grows ~10-15 lines per wave. Consider splitting into resolved-patterns.md (RESOLVED items) and active-patterns.md (open items) if it reaches 150.

Split these only if the content becomes domain-segmentable (e.g., scenario runner vs gameplay systems).
