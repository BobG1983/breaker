# Name
detect_cell_deaths

# SystemSet
New: `DeathPipelineSystems::DetectDeaths`. Runs in FixedUpdate, after `DeathPipelineSystems::ApplyDamage`.

# Filepath
`src/cells/systems/detect_cell_deaths.rs`

# Queries/Filters
- A list of `(Entity, &KilledBy, &Hp, Has<RequiredToClear>)` with `Cell` component, filtered to `Changed<Hp>`

# Description
Find all cell entities whose Hp just changed and is now ≤ 0. For each, send `KillYourself<Cell>` with the victim entity and killer from KilledBy.dealer.

The `Changed<Hp>` filter ensures we only process cells that took damage this frame, not all cells every frame.

DO read RequiredToClear status — downstream systems (node completion tracking) need this. Pass it through via the message or let downstream read it from the still-alive entity.
DO NOT despawn the entity. The domain kill handler does that later.
DO NOT process cells that were already dead in a previous frame — Changed<Hp> handles this naturally since Hp only changes on the damage frame.
