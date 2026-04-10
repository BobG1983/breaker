# Name
detect_wall_deaths

# SystemSet
`DeathPipelineSystems::DetectDeaths`. Runs in FixedUpdate, after `DeathPipelineSystems::ApplyDamage`.

# Filepath
`src/walls/systems/detect_wall_deaths.rs`

# Queries/Filters
- A list of `(Entity, &KilledBy, &Hp)` with `Wall` component, `Without<Dead>`

# Description
Find all living wall entities whose Hp is ≤ 0. For each, send `KillYourself<Wall>` with the victim entity and killer from KilledBy.dealer.

Only walls that have Hp are queryable. Permanent walls without Hp are unaffected. One-shot walls (shield walls) have Hp and die on first hit.

DO NOT insert `Dead`. The domain kill handler decides whether the entity actually dies.
DO NOT despawn the entity.
