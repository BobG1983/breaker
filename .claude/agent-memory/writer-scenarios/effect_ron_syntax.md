---
name: Effect RON syntax reference
description: Complete EffectKind variants with exact field names as they appear in RON initial_effects
type: reference
---

## RootEffect wrapper (top level)

```ron
On(target: Bolt, then: [...])
On(target: Breaker, then: [...])
On(target: AllBolts, then: [...])
```

## EffectNode types

```ron
When(trigger: PerfectBumped, then: [...])
Do(SpeedBoost(multiplier: 1.5))
Until(trigger: TimeExpires(1.0), then: [...])
Once([...])
On(target: Cell, permanent: false, then: [...])
```

## Trigger variants

```ron
PerfectBumped       // targeted on bolt — perfect bump
EarlyBumped         // targeted on bolt — early bump
LateBumped          // targeted on bolt — late bump
Bumped              // targeted on bolt — any successful bump
PerfectBump         // global — perfect bump occurred
EarlyBump           // global
LateBump            // global
Bump                // global — any successful bump
BumpWhiff           // global — missed timing window
NoBump              // global — bolt hit breaker with no bump input
Impacted(Cell)      // targeted — impacted with cell
Impacted(Wall)      // targeted — impacted with wall
Impacted(Breaker)   // targeted — impacted with breaker
Impacted(Bolt)      // targeted — impacted with bolt
Impact(Cell)        // global — any impact with cell
CellDestroyed       // global — a cell was destroyed
NodeStart           // global — new node started
NodeEnd             // global — current node ended
NodeTimerThreshold(0.5)  // global — timer crossed ratio threshold (0.0–1.0)
TimeExpires(1.0)    // special — Until timer, fires when duration elapses
BoltLost            // global — bolt was lost
Death               // global — something died
Died                // targeted — this entity died
```

## EffectKind variants with their RON syntax

```ron
// Stat effects (reversible, stack multiplicatively)
Do(SpeedBoost(multiplier: 1.3))
Do(DamageBoost(0.5))
Do(Piercing(2))
Do(SizeBoost(0.2))
Do(BumpForce(2.5))
Do(QuickStop(multiplier: 3.0))

// AOE effects (spawn entities)
Do(Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0))
Do(ChainLightning(arcs: 3, range: 96.0, damage_mult: 1.5))
    // arc_speed is optional, serde default is 200.0 — use arc_speed: 50.0 for slow-arc stress
Do(ChainLightning(arcs: 5, range: 112.0, damage_mult: 1.5, arc_speed: 50.0))
Do(PiercingBeam(damage_mult: 2.0, width: 16.0))
Do(Explode(range: 80.0, damage_mult: 2.0))
Do(Pulse(base_range: 32.0, range_per_level: 8.0, stacks: 1, speed: 400.0))
    // interval field is optional, defaults to 0.5
Do(Pulse(base_range: 32.0, range_per_level: 8.0, stacks: 1, speed: 400.0, interval: 0.25))

// Bolt spawn effects
Do(SpawnBolts(count: 1, lifespan: 2.0, inherit: false))
    // count defaults to 1, lifespan is Option<f32> (None = no lifespan limit), inherit defaults to false
Do(ChainBolt(tether_distance: 120.0))
Do(TetherBeam(damage_mult: 1.5))
Do(SpawnPhantom(duration: 3.0, max_active: 1))

// Utility effects
Do(RampingDamage(damage_per_trigger: 0.25))
Do(Shield(stacks: 1))
Do(TimePenalty(seconds: 5.0))
Do(LoseLife)
Do(SecondWind)
Do(GravityWell(strength: 200.0, duration: 2.0, radius: 80.0, max: 1))
Do(Attraction(attraction_type: Cell, force: 500.0))
    // max_force is optional, defaults to None
Do(Attraction(attraction_type: Cell, force: 500.0, max_force: Some(300.0)))

// Complex effects
Do(RandomEffect([
    (0.5, Do(SpeedBoost(multiplier: 1.2))),
    (0.5, Do(DamageBoost(0.5))),
]))
Do(EntropyEngine(
    max_effects: 3,
    pool: [
        (0.5, Do(SpawnBolts(count: 1, lifespan: 2.0, inherit: false))),
        (0.5, Do(Shockwave(base_range: 32.0, range_per_level: 0.0, stacks: 1, speed: 400.0))),
    ],
))
```

## Target variants

```ron
Bolt        // primary bolt
AllBolts    // all bolts
Breaker     // the breaker
Cell        // single cell (context-sensitive)
AllCells    // all cells
Wall        // single wall (context-sensitive)
AllWalls    // all walls
```
