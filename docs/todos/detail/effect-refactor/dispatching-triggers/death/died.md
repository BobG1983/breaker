# Name
Died

# When it fires
An entity's HP reaches zero or it is otherwise killed.

# Scope
Local. Fires on the victim entity only.

On targets resolve as:
- `Death(Victim)` → the entity that died (same as the entity being walked)
- `Death(Killer)` → the entity that caused the death. May be absent for environmental deaths (timer expiry, DoT, etc.)

# Description
Died fires on the entity that is dying. This is where death-triggered effects like "explode when I die" (powder keg pattern) live. The entity's trees are walked before the entity is despawned.

DO fire Died before despawning the entity — its trees need to evaluate.
DO fire Killed(EntityKind) on the killer entity in the same frame.
DO fire DeathOccurred(EntityKind) globally in the same frame.
DO populate the killer in the trigger context when a killer exists.
DO NOT populate the killer when the death was environmental — On(Death(Killer)) will resolve to nothing and skip.
