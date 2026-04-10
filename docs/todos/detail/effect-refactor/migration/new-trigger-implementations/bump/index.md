# Bump Trigger Implementations

## Types
- [types.md](types.md) — existing messages consumed by bump bridges

## Local Triggers (on participants)
- [on_bumped.md](on_bumped.md) — any successful bump, walks bolt + breaker
- [on_perfect_bumped.md](on_perfect_bumped.md) — Perfect grade, walks bolt + breaker
- [on_early_bumped.md](on_early_bumped.md) — Early grade, walks bolt + breaker
- [on_late_bumped.md](on_late_bumped.md) — Late grade, walks bolt + breaker

## Global Triggers (on all entities with BoundEffects/StagedEffects)
- [on_bump_occurred.md](on_bump_occurred.md) — any successful bump
- [on_perfect_bump_occurred.md](on_perfect_bump_occurred.md) — Perfect grade
- [on_early_bump_occurred.md](on_early_bump_occurred.md) — Early grade
- [on_late_bump_occurred.md](on_late_bump_occurred.md) — Late grade
- [on_bump_whiff_occurred.md](on_bump_whiff_occurred.md) — bump window expired without contact
- [on_no_bump_occurred.md](on_no_bump_occurred.md) — bolt contacted breaker with no bump input (blocked)
