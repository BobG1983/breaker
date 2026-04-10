# Name
detect_breaker_deaths

# SystemSet
`DeathPipelineSystems::DetectDeaths`. Runs in FixedUpdate, after `DeathPipelineSystems::ApplyDamage`.

# Filepath
`src/breaker/systems/detect_breaker_deaths.rs`

# Queries/Filters
- A list of `(Entity, &KilledBy, &Hp)` with `Breaker` component, filtered to `Changed<Hp>`

# Description
Find all breaker entities whose Hp just changed and is now ≤ 0. For each, send `KillYourself<Breaker>` with the victim entity and killer from KilledBy.dealer.

Breaker death means game over (or run end). The domain kill handler for Breaker decides what happens — it does NOT despawn the breaker, it transitions game state.

DO NOT despawn the entity.
