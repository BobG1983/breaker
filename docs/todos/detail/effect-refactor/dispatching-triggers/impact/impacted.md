# Name
Impacted(EntityKind)

# When it fires
Two entities collide. Fires on both collision participants when the other entity matches the specified EntityKind.

# Scope
Local. Fires on both the impactor and the impactee.

On targets resolve as:
- `Impact(Impactor)` → the entity that initiated the collision (e.g. the bolt)
- `Impact(Impactee)` → the entity that was hit (e.g. the cell)

# Description
Impacted is parameterized by EntityKind. `Impacted(Cell)` on a bolt means "this bolt hit a cell." `Impacted(Wall)` on a bolt means "this bolt hit a wall." The EntityKind filters which collisions this trigger matches.

`Impacted(Any)` matches any collision regardless of the other entity's type.

The trigger fires on both participants — the bolt's trees see `Impacted(Cell)` and the cell's trees see `Impacted(Bolt)`. Each entity sees the other's kind.

DO fire on both participants with the appropriate EntityKind for each.
DO NOT fire Impacted when entities pass through each other without collision (e.g. piercing).
