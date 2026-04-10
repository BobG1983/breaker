# Name
apply_damage::\<Breaker\>

# SystemSet
`DeathPipelineSystems::ApplyDamage`. Runs in FixedUpdate.

# Filepath
`src/shared/systems/apply_damage.rs` — same generic system, monomorphized for Breaker.

# Queries/Filters
- A list of `(&mut Hp, &mut KilledBy)` with `Breaker` component, `Without<Dead>`

# Description
Read all `DamageDealt<Breaker>` messages. For each message, look up the target entity. Decrement Hp.current by the damage amount.

If this is the killing blow (Hp crossed from positive to ≤ 0), set KilledBy.dealer. First kill wins — do not overwrite.

Breaker damage comes from the LoseLife effect (amount: 1.0, dealer: None) triggered by BoltLostOccurred. Breakers without Hp (infinite lives) are not queryable and are unaffected.

DO NOT despawn. DO NOT send KillYourself.
