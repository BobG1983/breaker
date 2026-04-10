# Resolving Stamp Targets

When a definition (chip, breaker, bolt, cell, wall) is loaded and its `effects: [...]` list is processed, each `Stamp(target, tree)` entry resolves the StampTarget to one or more entities in the world. The tree is then stamped onto each resolved entity.

## Resolution Table

| StampTarget | Resolves to | When |
|-------------|-------------|------|
| Bolt | The primary bolt entity (With\<Bolt\> + With\<PrimaryBolt\>) | Must exist at dispatch time. Skipped if no primary bolt. |
| Breaker | The primary breaker entity (With\<Breaker\> + With\<PrimaryBreaker\>) | Must exist at dispatch time. Skipped if no primary breaker. |
| ActiveBolts | All entities with Bolt component that currently exist | Snapshot at dispatch time. Future bolts are not included. |
| EveryBolt | All existing bolts + all bolts spawned in the future | Stamps onto current bolts AND registers a Spawn(Bolt) watcher. |
| PrimaryBolts | All entities with Bolt + PrimaryBolt | Snapshot at dispatch time. |
| ExtraBolts | All entities with Bolt but NOT PrimaryBolt | Snapshot at dispatch time. |
| ActiveCells | All entities with Cell component that currently exist | Snapshot at dispatch time. |
| EveryCell | All existing cells + all cells spawned in the future | Stamps onto current cells AND registers a Spawn(Cell) watcher. |
| ActiveWalls | All entities with Wall component that currently exist | Snapshot at dispatch time. |
| EveryWall | All existing walls + all walls spawned in the future | Stamps onto current walls AND registers a Spawn(Wall) watcher. |
| ActiveBreakers | All entities with Breaker component that currently exist | Snapshot at dispatch time. |
| EveryBreaker | All existing breakers + all breakers spawned in the future | Stamps onto current breakers AND registers a Spawn(Breaker) watcher. |

## Notes

- Singular targets (Bolt, Breaker) resolve to at most one entity. If the entity doesn't exist yet, the stamp is silently skipped.
- Active* targets resolve to zero or more entities. Empty results are not an error.
- Every* targets are sugar — they resolve to the Active* snapshot plus a Spawn watcher for future entities.
