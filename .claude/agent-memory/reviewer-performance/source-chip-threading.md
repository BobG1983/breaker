---
name: source_chip threading and shield absorption performance analysis
description: Analysis of EffectSourceChip component, chip_attribution allocations, DamageVisualQuery shield change, per-DamageCell clone pattern — reviewed on feature/runtime-effects branch
type: project
---

## What Was Added (feature/runtime-effects)

- `EffectSourceChip(Option<String>)` component added to 6 spawned effect entity types:
  shockwave, pulse ring (conditional insert), explode request, chain lightning request,
  piercing beam request, tether beam.
- `chip_attribution(source_chip: &str) -> Option<String>` helper in core/types.rs — converts
  empty str to None, non-empty to Some(to_string()).
- `Option<&EffectSourceChip>` added to damage queries in all 6 systems.
- `DamageCell.source_chip: Option<String>` field populated via `esc.and_then(|e| e.0.clone())`.
- `DamageVisualQuery` in cells/queries.rs changed from `Has<ShieldActive>` to
  `Option<&'static mut ShieldActive>` — required for shield charge decrement logic.
- `FireEffectCommand` / `ReverseEffectCommand` carry `source_chip: String`.

## Archetype Impact: Clean

Each of the 6 effect entity types already had a unique component set before this change. Adding
`EffectSourceChip` to each adds one field to those archetypes but does NOT split any existing
archetype that wasn't already split by the entity's other unique components. No fragmentation.

Exception: pulse rings get a CONDITIONAL insert (if emitter has EffectSourceChip). This creates
two ring archetypes: with and without EffectSourceChip. Both are covered by
`Option<&EffectSourceChip>` in apply_pulse_damage. At current scale (handful of rings) this is
harmless. If rings become numerous, unconditionally inserting EffectSourceChip(None) would
collapse to one archetype. Not worth it now.

## chip_attribution Allocation: Clean

`chip_attribution` does `to_string()` on non-empty source chips. Called once per fire() invocation.
fire() is episodic (chip dispatch, timer expiry) — never per-frame. Clean.

## Per-DamageCell Clone: Minor

`esc.and_then(|e| e.0.clone())` clones Option<String> once per cell hit per damage application.
For shockwave and pulse: bounded by dedup sets (ShockwaveDamaged/PulseDamaged) — at most ~50
clones per shockwave/ring lifetime, not per tick. For explode and beam: one clone per cell per
request (all ephemeral). The clone is None (cheap) or a short String.

If source_chip ever needs to be shared across many DamageCell writes, Option<Arc<str>> would
eliminate the clone cost. Not worth changing at current scale.

## DamageVisualQuery: Option<&mut ShieldActive> is Correct

Mutable access required — handle_cell_hit modifies shield.charges and calls
commands.entity().remove::<ShieldActive>() when charges reach 0. This is semantically correct.
The prior Has<ShieldActive> would have been read-only and insufficient. Scheduling impact: the
system already holds &mut CellHealth and ResMut<Assets<ColorMaterial>>, so no new parallelism
constraints are introduced.

## tether_beam: Per-Tick HashSet Allocation (unchanged from Phase 5, still Moderate)

damaged_this_tick: HashSet<Entity> = HashSet::new() in tick_tether_beam (line 131) is fresh
every tick per active beam. This pre-dates the source_chip work. With 0-1 beams active, it is
0-1 allocations per tick — imperceptible. Becomes Moderate at 5+ simultaneous beams.
Could be stored in TetherBeamComponent as a Vec<Entity> cleared each tick.

## FireEffectCommand / ReverseEffectCommand String: Clean

One String allocation per chip dispatch per effect fired. Chip dispatch is episodic (user
triggered). Not a concern.

## Summary Verdict

source_chip threading is implemented cleanly at this scale. No archetype fragmentation from the
new component. No hot-path allocation problems. The conditional pulse ring insert is a minor
two-archetype consequence worth remembering. The per-DamageCell clone is bounded and academic.
The tether HashSet remains the only real pattern to watch, unchanged from Phase 5 analysis.
