# Name
detect_breaker_deaths

# SystemSet
`DeathPipelineSystems::DetectDeaths`. Runs in FixedUpdate, after `DeathPipelineSystems::ApplyDamage`.

# Filepath
`src/breaker/systems/detect_breaker_deaths.rs`

# Queries/Filters
- A list of `(Entity, &KilledBy, &Hp)` with `Breaker` component, `Without<Dead>`

# Description
Find all living breaker entities whose Hp is ≤ 0. For each, send `KillYourself<Breaker>` with the victim entity and killer from KilledBy.dealer.

Breaker death means game over (or run end). The domain kill handler for Breaker decides what happens — it does NOT despawn the breaker, it transitions game state.

DO NOT insert `Dead`. The domain kill handler decides whether the entity actually dies.
DO NOT despawn the entity.
