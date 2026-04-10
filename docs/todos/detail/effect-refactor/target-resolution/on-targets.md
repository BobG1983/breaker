# Resolving On Targets

When the tree walker encounters `On(participant, terminal)`, it resolves the ParticipantTarget to a specific entity from the trigger event context. The terminal is then evaluated on that entity instead of the Owner.

## Resolution Table

### Bump(BumpTarget)

Trigger context: a bump event between a bolt and a breaker.

| BumpTarget | Resolves to |
|------------|-------------|
| Bolt | The bolt entity that was bumped |
| Breaker | The breaker entity that did the bumping |

### Impact(ImpactTarget)

Trigger context: a collision event between two entities.

| ImpactTarget | Resolves to |
|--------------|-------------|
| Impactor | The entity that initiated the collision (e.g. the bolt) |
| Impactee | The entity that was hit (e.g. the cell) |

### Death(DeathTarget)

Trigger context: an entity death event.

| DeathTarget | Resolves to |
|-------------|-------------|
| Victim | The entity that died |
| Killer | The entity that caused the death. May not exist for environmental deaths (e.g. timer expiry). |

### BoltLost(BoltLostTarget)

Trigger context: a bolt fell off the bottom of the playfield.

| BoltLostTarget | Resolves to |
|----------------|-------------|
| Bolt | The bolt entity that was lost |
| Breaker | The breaker entity that lost the bolt |

## Notes

- Each participant resolves to exactly one entity from the trigger event context.
- If the entity no longer exists in the world (e.g. despawned between trigger and resolution), the terminal is skipped.
- Killer in Death may be absent for environmental deaths — the terminal is skipped in that case.
