# Name
StampTarget

# Syntax
```rust
enum StampTarget {
    Bolt,
    Breaker,
    ActiveBolts,
    EveryBolt,
    PrimaryBolts,
    ExtraBolts,
    ActiveCells,
    EveryCell,
    ActiveWalls,
    EveryWall,
    ActiveBreakers,
    EveryBreaker,
}
```

# Description
- Bolt: Primary bolt entity. See [bolt](../ron-syntax/stamp-targets/bolt.md)
- Breaker: Primary breaker entity. See [breaker](../ron-syntax/stamp-targets/breaker.md)
- ActiveBolts: All bolt entities that exist right now. See [active-bolts](../ron-syntax/stamp-targets/active-bolts.md)
- EveryBolt: All existing bolts + all bolts spawned in the future. See [every-bolt](../ron-syntax/stamp-targets/every-bolt.md)
- PrimaryBolts: All bolts with the PrimaryBolt marker. See [primary-bolts](../ron-syntax/stamp-targets/primary-bolts.md)
- ExtraBolts: All bolts with the ExtraBolt marker. See [extra-bolts](../ron-syntax/stamp-targets/extra-bolts.md)
- ActiveCells: All cell entities that exist right now. See [active-cells](../ron-syntax/stamp-targets/active-cells.md)
- EveryCell: All existing cells + all cells spawned in the future. See [every-cell](../ron-syntax/stamp-targets/every-cell.md)
- ActiveWalls: All wall entities that exist right now. See [active-walls](../ron-syntax/stamp-targets/active-walls.md)
- EveryWall: All existing walls + all walls spawned in the future. See [every-wall](../ron-syntax/stamp-targets/every-wall.md)
- ActiveBreakers: All breaker entities that exist right now. See [active-breakers](../ron-syntax/stamp-targets/active-breakers.md)
- EveryBreaker: All existing breakers + all breakers spawned in the future. See [every-breaker](../ron-syntax/stamp-targets/every-breaker.md)
