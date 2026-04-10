# Name
apply_damage::\<Bolt\>

# SystemSet
`DeathPipelineSystems::ApplyDamage`. Runs in FixedUpdate.

# Filepath
`src/shared/systems/apply_damage.rs` — same generic system as Cell, monomorphized for Bolt.

# Queries/Filters
- A list of `(&mut Hp, &mut KilledBy)` with `Bolt` component

# Description
Read all `DamageDealt<Bolt>` messages. For each message, look up the target entity. Decrement Hp.current by the damage amount.

If this is the killing blow (Hp crossed from positive to ≤ 0), set KilledBy.dealer. First kill wins — do not overwrite.

Bolts currently die from lifespan expiry or falling off-screen, not from damage. This system exists for future mechanics (bolt-damaging hazards, bolt-vs-bolt). It may process zero messages in most frames.

DO NOT despawn. DO NOT send KillYourself.
