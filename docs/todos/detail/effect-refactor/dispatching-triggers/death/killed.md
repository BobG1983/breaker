# Name
Killed(EntityKind)

# When it fires
This entity killed an entity of the specified kind.

# Scope
Local. Fires on the killer entity only.

On targets resolve as:
- `Death(Killer)` → the killer entity (same as the entity being walked)
- `Death(Victim)` → the entity that died

# Description
Killed is the killer's perspective of a death event. `Killed(Cell)` on a bolt means "this bolt just killed a cell." The EntityKind filters which victim types this trigger matches.

`Killed(Any)` matches killing any entity type.

DO fire alongside Died on the victim and DeathOccurred globally.
DO NOT fire Killed when there is no killer (environmental deaths).
