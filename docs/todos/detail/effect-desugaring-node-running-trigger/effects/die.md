# Die

## Config
No config fields.

**RON**: `Die`

## Reversible: NO

## Target: Any entity with health

## Fire
1. Send `KillYourself` message for the target entity
2. Killer is populated from `TriggerContext` (the entity that caused this trigger chain)
3. Domain handler receives `KillYourself`, validates, sends `Destroyed`
4. `bridge_destroyed` fires death triggers (Died, Killed, DeathOccurred)

## Reverse
Not applicable — death is irreversible.

## Messages Sent
- `KillYourself<T: GameEntity>` where T is the victim type (determined from entity). Killer is `Option<Entity>` from TriggerContext.

## Notes
- This is the NEW effect that replaces direct entity despawn
- The full chain is: `Fire(Die)` → `KillYourself` → domain handler → `Destroyed` → `bridge_destroyed` → death triggers → `DespawnEntity`
- Killer attribution propagates from TriggerContext through the entire chain
- Used for "one-shot walls" (`Route(Wall, When(Impacted(Bolt), Fire(Die)))`) and similar patterns
