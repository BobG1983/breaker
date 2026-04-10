# Systems to Remove

These systems are replaced by the unified death pipeline.

| System | File | Replaced by |
|--------|------|-------------|
| `handle_cell_hit` | `src/cells/systems/handle_cell_hit/system.rs` | `apply_damage::<Cell>` (damage + HP) + `detect_cell_deaths` (death detection) |
| `cleanup_cell` | `src/cells/systems/cleanup_cell.rs` | Domain kill handler for Cell + `process_despawn_requests` |
| `cleanup_destroyed_bolts` | `src/bolt/systems/cleanup_destroyed_bolts.rs` | Domain kill handler for Bolt + `process_despawn_requests` |
| `tick_bolt_lifespan` (death portion) | `src/bolt/systems/tick_bolt_lifespan.rs` | Lifespan expiry sends `KillYourself<Bolt>` instead of `RequestBoltDestroyed` |

Note: `handle_cell_hit` currently does both damage application AND visual feedback. The visual feedback portion (material color update) will need a separate system or be folded into a damage response system — it is NOT part of the death pipeline.
