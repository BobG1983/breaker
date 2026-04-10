# Name
apply_damage::\<Wall\>

# SystemSet
`DeathPipelineSystems::ApplyDamage`. Runs in FixedUpdate.

# Filepath
`src/shared/systems/apply_damage.rs` — same generic system, monomorphized for Wall.

# Queries/Filters
- A list of `(&mut Hp, &mut KilledBy)` with `Wall` component, `Without<Dead>`

# Description
Read all `DamageDealt<Wall>` messages. For each message, look up the target entity. Decrement Hp.current by the damage amount.

If this is the killing blow, set KilledBy.dealer. First kill wins.

Walls that have Hp (one-shot walls, destructible walls) use this. Walls without Hp are not queryable and are unaffected.

DO NOT despawn. DO NOT send KillYourself.
