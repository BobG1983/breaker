# Name
detect_bolt_deaths

# SystemSet
`DeathPipelineSystems::DetectDeaths`. Runs in FixedUpdate, after `DeathPipelineSystems::ApplyDamage`.

# Filepath
`src/bolt/systems/detect_bolt_deaths.rs`

# Queries/Filters
- A list of `(Entity, &KilledBy, &Hp)` with `Bolt` component, filtered to `Changed<Hp>`

# Description
Find all bolt entities whose Hp just changed and is now ≤ 0. For each, send `KillYourself<Bolt>` with the victim entity and killer from KilledBy.dealer.

Most bolt deaths are environmental (lifespan expiry, falling off-screen) and bypass Hp entirely — those paths send KillYourself\<Bolt\> directly with killer = None. This system handles the Hp-based death path for future damage-to-bolt mechanics.

DO NOT despawn the entity.
