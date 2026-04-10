# Name
detect_bolt_deaths

# SystemSet
`DeathPipelineSystems::DetectDeaths`. Runs in FixedUpdate, after `DeathPipelineSystems::ApplyDamage`.

# Filepath
`src/bolt/systems/detect_bolt_deaths.rs`

# Queries/Filters
- A list of `(Entity, &KilledBy, &Hp)` with `Bolt` component, `Without<Dead>`

# Description
Find all living bolt entities whose Hp is ≤ 0. For each, send `KillYourself<Bolt>` with the victim entity and killer from KilledBy.dealer.

Most bolt deaths are environmental (lifespan expiry, falling off-screen) and bypass Hp entirely — those paths send `KillYourself<Bolt>` directly with killer = None. The domain kill handler inserts `Dead`, so this system skips them via `Without<Dead>`. This system handles the Hp-based death path for future damage-to-bolt mechanics.

DO NOT insert `Dead`. The domain kill handler decides whether the entity actually dies.
DO NOT despawn the entity.
