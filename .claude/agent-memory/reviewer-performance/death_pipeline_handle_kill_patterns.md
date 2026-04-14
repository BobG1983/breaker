---
name: death_pipeline handle_kill patterns
description: handle_kill<T> hot-path patterns — HashSet dedup, dual Position2D queries, Dead insert archetype churn — all confirmed acceptable at current scale
type: project
---

System: `breaker-game/src/shared/death_pipeline/systems/system.rs`

## Confirmed patterns

**HashSet<Entity>::new() per invocation (line 112)**
- Only allocates when kill messages arrive (not every tick)
- At 50–200 cells, fires at most a handful of times per node at kill events
- `Local<HashSet<Entity>>` reuse would save allocations for mass-death scenarios
- Not worth complexity at current scale; revisit if Phase 3 introduces wave-clear effects hitting 50+ cells in a single tick

**Dual Position2D queries (KillVictimQuery<T> + KillerPositionQuery)**
- Both read-only — no conflict in Bevy 0.18 even with overlapping archetypes
- Killer query uses get() (O(1) entity lookup), not iteration — correct
- Victim query filtered With<T>, Without<Dead> — correctly narrows archetypes

**commands.entity(v).insert(Dead) archetype move**
- One-time per cell death, not per-frame — archetype churn is not a concern at 50–200 cells
- Dead is a ZST marker — move is cheap
- Same-frame deduplication via HashSet closes the command-deferral window correctly

**Dead-letter systems**
- apply_damage::<Wall>, apply_damage::<Breaker>, detect_deaths::<Wall>, detect_deaths::<Breaker> registered but have no message producers
- Cost is one queue peek + zero/4 query iterations per tick — negligible
- Intentionally scoped out of Wave F1 (D15 revised); flag for cleanup if Wall/Breaker damage never lands

**Why:** All patterns are acceptable at 1 Breaker, 1–few Bolts, 50–200 Cells scale.
**How to apply:** Do not re-flag these in future reviews unless entity counts grow significantly beyond current phase expectations.
