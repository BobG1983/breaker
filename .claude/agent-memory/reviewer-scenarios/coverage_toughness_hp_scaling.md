---
name: Toughness + HP Scaling Coverage Map
description: Coverage gaps for Toughness enum, ToughnessConfig, guardian_hp_fraction, hp_mult removal, and HP scaling at tier/boss; audit done after cell-builder-pattern merge
type: project
---

## Feature Summary

HP values are no longer hardcoded. The `Toughness` enum (Weak=10, Standard=20, Tough=30 at tier 0) 
drives base HP via `ToughnessConfig`. `hp_mult` was removed from the difficulty curve; 
`ToughnessConfig.tier_multiplier` / `node_multiplier` / `boss_multiplier` replace it.
Guardian HP is derived from parent HP via `guardian_hp_fraction` in `GuardedBehavior`.

**Key code:** `breaker-game/src/cells/resources/data.rs` (ToughnessConfig), 
`breaker-game/src/cells/definition/data.rs` (Toughness, GuardedBehavior),
`breaker-game/src/state/run/node/systems/spawn_cells_from_layout/system.rs` (HpScale, compute_hp, guardian_hp).

## Unit Test Coverage (Good)

All critical math paths are unit-tested in `cells/resources/tests.rs` and `spawn_cells_from_layout/tests/behaviors.rs`:
- base_hp per toughness variant at tier 0, pos 0
- tier_scale with tier only, pos only, both
- hp_for(Weak/Standard/Tough, tier 3, pos 4)
- hp_for_boss with boss_multiplier
- guardian HP = parent_hp * guardian_hp_fraction (Behaviors 39-41)
- No-ToughnessConfig fallback to default_base_hp()

## Scenario Coverage (Gaps)

### What existing scenarios cover
- Dense layout has T and S cells → Tough and Standard cells receive bolt impacts
- Bastion layout has Gu + gu (Tough guarded, Weak guardian) → AabbMatchesEntityDimensions validates guardian AABB
- BossArena layout has T and S cells
- All layouts: scenario runner does NOT inject NodeOutcome/NodeSequence/ToughnessConfig
  → ALL scenarios run at tier=0, position_in_tier=0, is_boss=false, using default_base_hp fallback
  → hp_mult removal is never verified in scenarios (no scenario confirmed hp_mult was ever tested)
  → tier scaling path (tier>0) is NEVER exercised by any scenario
  → boss_multiplier path (is_boss=true) is NEVER exercised by any scenario

### HIGH gaps
1. No scenario verifies that Standard(20)/Tough(30)/Weak(10) base HP values are actually what gets 
   spawned at tier 0 — scenarios don't assert on CellHealth values
2. No scenario uses a layout with ONLY Tough cells to verify harder cells require more bumps
3. No scenario verifies tier-scaled HP means cells survive longer mid-run progression
4. No scenario tests guardian HP fraction: that guardians at a given tier die in fewer hits than parent
5. No InvariantKind::CellHpPositive (cell HP never goes negative or NaN) — NoNaN covers NaN but 
   not negative-HP cells surviving indefinitely

### Invariant gaps
- No invariant catches: cell HP < 0 (NoNaN catches NaN, but not underflow to negative)
- No invariant catches: guardian HP > parent HP (hp_fraction > 1.0 would already be caught by 
  GuardedBehavior::validate(), so this is load-time only)
- No scenario runner capability to inject ToughnessConfig with non-default values (no MutationKind for it)

## Why: What changed that broke coverage

The old design had `hp_mult` in `NodeAssignment`; scenarios exercised the cell HP path at tier 0 
only and `hp_mult` was always 1.0 in scenarios (NodeOutcome not injected). The removal of hp_mult 
and replacement with ToughnessConfig is transparent to scenarios because scenarios never inject 
tier/position context — they always hit the no-config fallback path.

## How to apply

When writing toughness scenarios: the runner cannot inject ToughnessConfig overrides or NodeOutcome. 
Verify toughness differentiation by observing that Tough cells require more bolt bounces to clear 
than Standard cells in the same layout (use ramping_damage or high damage chips + DamageBoost to 
measure destruction rate). Alternatively, add a MutationKind::InjectToughnessConfig or 
DebugSetup field to override the resource at spawn time.
