# Name
Impacted

# Parameters
`EntityKind` — what type of entity was in the collision.

# Description
Fires when the Owner entity collides with an entity of the specified kind. `Impacted(Cell)` on a bolt means "I just hit a cell." `Impacted(Bolt)` on a cell means "a bolt just hit me." Fires on both entities involved in the collision. The EntityKind parameter filters which collisions trigger it — `Impacted(Any)` fires on any collision.
