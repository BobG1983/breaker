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

## Wave 6C additions (Salvo death pipeline)

**3 new Salvo generic systems (plugin.rs)**
- apply_damage::<Salvo>, detect_deaths::<Salvo>, handle_kill::<Salvo> added
- All in FixedUpdate inside existing DeathPipelineSystems set order — scheduling is correct
- handle_kill::<Breaker> intentionally ABSENT (Breaker uses separate handle_breaker_death) — correct
- Salvo entity count: small (a few per firing turret, not hundreds)

**on_salvo_destroyed bridge (death/bridges/system.rs)**
- Structurally identical to on_cell_destroyed / on_bolt_destroyed — same double global_query.iter() pattern
- Salvo carries NO BoundEffects — victim local walk will always hit the guard and skip
- Only the global DeathOccurred walks fire; those are over Breaker (1 entity) only in practice
- Acceptable at current scale; already noted in death bridge double global_query walk memory

**ImpactReaders salvo_breaker field (impact/bridges/system.rs)**
- One new MessageReader<SalvoImpactBreaker> added to SystemParam struct
- Each of on_impacted and on_impact_occurred gets its own ImpactReaders instance — no shared cursor
- Both systems are in the same system set (EffectV3Systems::Bridge) — Bevy schedules them sequentially or in parallel as allowed; MessageReader access is compatible
- on_impact_occurred adds 3 kinds.push() per salvo-breaker collision — Vec alloc pre-existing; no new concern

**stamp_required_effects at breaker spawn**
- Iterates bolt_lost + projectile_hit (2 optional Vecs) per breaker spawn
- spawn-time only, called once per breaker entity (1 entity total)
- No per-frame cost

**Why:** All patterns are acceptable at 1 Breaker, 1–few Bolts, few Salvos, 50–200 Cells scale.
**How to apply:** Do not re-flag these in future reviews unless entity counts grow significantly beyond current phase expectations.
