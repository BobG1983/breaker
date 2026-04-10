# Unified Death Pipeline

Replace the per-domain death messaging with a generic pipeline that works for all entity types.

## Why

Today each domain has its own death messages and systems: `DamageCell` → `handle_cell_hit` → `RequestCellDestroyed` → `cleanup_cell` → `CellDestroyedAt` for cells, `RequestBoltDestroyed` → `cleanup_destroyed_bolts` for bolts. There is no damage attribution (who killed what), no unified HP component, and no way for the effect system to trigger death-related effects generically.

The unified pipeline introduces:
- A single `Hp` component for all damageable entities
- A single `DamageDealt<T>` message per victim type
- Kill attribution via `KilledBy` (set on the killing blow only)
- A generic death chain: `KillYourself<T>` → domain handler → `Destroyed<T>` → trigger dispatch → `DespawnEntity`
- The effect system's `Fire(Die)` integrates directly by sending `KillYourself<T>`

## Documents

- [definitions.md](definitions.md) — Table of all terms and types
- [rust-types/](rust-types/index.md) — New type definitions
- [migration/](migration/index.md) — What to remove, what to create, where it goes
