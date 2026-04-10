# Type Migration

| Before | After | Location |
|--------|-------|----------|
| `CellHealth` | `Hp` | `src/cells/components/types.rs` → `src/shared/components.rs` (shared) |
| `DamageCell` | `DamageDealt<Cell>` | `src/cells/messages.rs` → `src/shared/messages.rs` |
| `RequestCellDestroyed` | `KillYourself<Cell>` | `src/cells/messages.rs` → `src/shared/messages.rs` |
| `CellDestroyedAt` | `Destroyed<Cell>` | `src/cells/messages.rs` → `src/shared/messages.rs` |
| `RequestBoltDestroyed` | `KillYourself<Bolt>` | `src/bolt/messages.rs` → `src/shared/messages.rs` |
| (none) | `DamageDealt<Bolt>` | New — `src/shared/messages.rs` |
| (none) | `DamageDealt<Wall>` | New — `src/shared/messages.rs` |
| (none) | `Destroyed<Bolt>` | New — `src/shared/messages.rs` |
| (none) | `Destroyed<Wall>` | New — `src/shared/messages.rs` |
| (none) | `KillYourself<Bolt>` | New — `src/shared/messages.rs` |
| (none) | `KillYourself<Wall>` | New — `src/shared/messages.rs` |
| (none) | `KilledBy` | New — `src/shared/components.rs` |
| (none) | `DespawnEntity` | New — `src/shared/messages.rs` |
| (none) | `GameEntity` trait | New — `src/shared/traits.rs` |
