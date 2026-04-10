# Name
apply_damage::\<Cell\>

# SystemSet
New: `DeathPipelineSystems::ApplyDamage`. Runs in FixedUpdate, after `BoltSystems::CellCollision` (damage messages must be sent before they're processed). Must run after `check_lock_release` so cells unlocked this frame can receive damage.

# Filepath
`src/shared/systems/apply_damage.rs` — generic system, monomorphized per T.

# Queries/Filters
- A list of `(&mut Hp, &mut KilledBy)` with `Cell` component, without `Locked` component

# Description
Read all `DamageDealt<Cell>` messages. For each message, look up the target entity. Decrement Hp.current by the damage amount.

If Hp was positive before this message and is now ≤ 0, this is the killing blow. Set `KilledBy.dealer` to the message's dealer field. If KilledBy.dealer is already set (another message killed it first this frame), DO NOT overwrite — first kill wins.

DO skip entities with the `Locked` component — locked cells cannot take damage.
DO NOT despawn the entity. DO NOT send KillYourself. That is detect_cell_deaths' job.
DO NOT update visual feedback — that is a separate concern.
