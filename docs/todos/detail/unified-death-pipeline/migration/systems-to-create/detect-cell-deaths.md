# Name
detect_cell_deaths

# SystemSet
`DeathPipelineSystems::DetectDeaths`. Runs in FixedUpdate, after `DeathPipelineSystems::ApplyDamage`.

# Filepath
`src/cells/systems/detect_cell_deaths.rs`

# Queries/Filters
- A list of `(Entity, &KilledBy, &Hp, Has<RequiredToClear>)` with `Cell` component, `Without<Dead>`

# Description
Find all living cell entities whose Hp is ≤ 0. For each, send `KillYourself<Cell>` with the victim entity and killer from KilledBy.dealer.

The `Without<Dead>` filter skips entities already confirmed dead by their domain kill handler. This prevents re-sending KillYourself for entities that were killed directly (e.g., by the Die effect) and already processed by the kill handler.

DO read RequiredToClear status — downstream systems (node completion tracking) need this. Pass it through via the message or let downstream read it from the still-alive entity.
DO NOT insert `Dead`. The domain kill handler decides whether the entity actually dies.
DO NOT despawn the entity. The domain kill handler does that later.
