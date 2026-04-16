---
name: Sequence Wave 2 performance patterns
description: init_sequence_groups, reset_inactive_sequence_hp, advance_sequence — all acceptable at current cell scale
type: project
---

Reviewed Wave 2 sequence behavior systems. Note: plugin.rs was merged into
breaker-game/src/cells/plugin.rs (sequence scheduling lives there now, not in
a dedicated sequence plugin file). Tests expanded to groups A–G across
breaker-game/src/cells/behaviors/sequence/tests/. Production system code is
unchanged from original review.

## File locations (post-refactor)

- breaker-game/src/cells/behaviors/sequence/systems/reset_inactive_sequence_hp.rs
- breaker-game/src/cells/behaviors/sequence/systems/advance_sequence.rs
- breaker-game/src/cells/behaviors/sequence/systems/init_sequence_groups.rs
- breaker-game/src/cells/behaviors/sequence/components.rs
- Scheduling: breaker-game/src/cells/plugin.rs (lines 39–57)

## reset_inactive_sequence_hp (FixedUpdate hot path)

- Query: `(With<SequenceCell>, Without<SequenceActive>)` — clean filter pair
- `SequenceActive` is never explicitly removed from live entities; departs only
  via despawn, so no runtime archetype churn on this component
- Loop body: two field reads + two conditional writes — zero allocations confirmed
- `mut Hp` + `mut KilledBy` access is intentional; forced by the
  `killed_by.dealer = None` clear requirement; no weaker access possible
- `run_if(in_state(NodeState::Playing))` guard means zero cost outside Playing

## advance_sequence (FixedUpdate, rare path)

- Message-driven: inner body only runs when `Destroyed<Cell>` messages are
  present; effectively zero-cost ticks with no deaths
- O(N_deaths × M_candidates) inner scan; N is 1–3 per tick in practice; M is
  bounded by inactive sequence cells (~50–200 max)
- SequenceGroup(u32) + SequencePosition(u32) comparisons are integer equality —
  no allocations, no heap, no HashMap
- `dying.get(msg.victim)` is O(1) entity lookup, not a scan
- `commands.entity().insert()` is deferred — no synchronous structural change

## init_sequence_groups (OnEnter cold path)

- Single pass over all sequence cells; N `commands.entity(e).insert(SequenceActive)`
  calls proportional to grid size — negligible at spawn time
- Only fires once per NodeState::Playing entry

## Archetype fragmentation assessment

- SequenceActive presence/absence creates exactly 2 archetype variants per base
  set (active vs inactive) — intended design, not fragmentation
- No runtime insert/remove of SequenceActive on live entities — it only arrives
  via init_sequence_groups/advance_sequence and departs via despawn; churn is
  bounded by cell deaths, not per-frame
- DyingActiveSequenceQuery uses triple With<Cell, SequenceCell, SequenceActive>
  and NextSequenceMemberQuery uses With<Cell, SequenceCell> + Without<SequenceActive>
  — both are tight and archetype-correct

## Scheduling (cells/plugin.rs lines 39–57)

- All three sequence systems are in the same FixedUpdate add_systems call, gated
  by run_if(in_state(NodeState::Playing))
- init_sequence_groups on OnEnter(NodeState::Playing) — correct, one-shot
- reset_inactive_sequence_hp: after ApplyDamage, before DetectDeaths — correct
- advance_sequence: after EffectV3Systems::Death — correct

**How to apply:** Flag no issues for these systems at current (50–200 cell) scale.
If Phase 3 introduces hundreds of sequence cells with frequent per-tick promotions,
revisit HashMap candidate indexing in advance_sequence. Archetype churn concern
is academic until SequenceActive is inserted/removed on live entities (currently
it never is — only despawn removes it).
