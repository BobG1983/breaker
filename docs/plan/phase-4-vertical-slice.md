# Phase 4: Vertical Slice — Mini-Run

**Goal**: A playable 3-5 node run that proves the architecture and feels like the game.

## Upgrade System (v1)

Three categories, each modifying a different part of the system:

- **Amps**: Passive bolt modifications (speed, size, damage, piercing, ricochet, etc.)
- **Augments**: Passive breaker modifications (width, speed, bump strength, tilt angles, dash distance, etc.)
- **Overclocks**: Triggered abilities with conditional activation chains

### Selection Screen
- Pick 1 of 3 random options (seeded RNG, can be any mix of Amp/Augment/Overclock)
- Upgrades **stack** across the run, building synergies
- Countdown timer — if time expires, **you get nothing**

### Upgrade Application Architecture

Amps and Augments apply as Bevy components that modify existing behavior (e.g. `PiercingShot` component on the bolt entity skips first-hit despawn).

**Overclocks use a trigger/effect system** — RON-defined behavior chains, modeled after breaker behaviors:

- **Triggers**: `OnPerfectBump`, `OnImpact`, `OnCellDestroyed`, `OnBoltLost`, etc.
- **Trigger chains** (nested/sequential): A trigger can start a **tracking state**, and a second trigger checks that state before firing the effect. Example: Surge overclock = `OnPerfectBump` → mark bolt as "surging" → `OnImpact` while surging → fire shockwave.
- **Effects**: `Shockwave { range: f32 }`, `MultiBolt`, `Shield`, etc.
- **RON-defined**: Overclock definitions specify their trigger chain + effect + parameters in data.

### Bolt Behaviors Domain

New `src/bolt/behaviors/` module (mirrors `src/breaker/behaviors/`):
- Bolt behavior definitions in RON
- Trigger evaluation system that reads bolt state + game messages
- Effect execution systems (shockwave, piercing, etc.)
- Bolt state tracking components (e.g. `Surging` marker set by trigger, consumed by effect)

### Shockwave Effect (Surge Overclock)

The first concrete overclock implementation, proving the trigger/effect architecture:
- **Range parameter**: configurable radius, upgradeable in future phases
- **Visual**: expanding ring VFX with fadeout at range boundary (fits existing shader-driven style)
- **Damage**: any cell within range takes 1 damage
- **Trigger chain**: `OnPerfectBump` → mark surging → `OnImpact` if surging → fire shockwave at impact point

### Hot-Reload Support

Chip definitions (Amps, Augments, Overclocks) in RON need hot-reload propagation — add to the `HotReloadPlugin` chain established in Phase 3c. When a chip RON file changes: rebuild the chip registry, and if any active chips were modified, re-apply their effects to live entities.

### Upgrade Definitions in RON

```ron
// Example: Surge overclock
(
    name: "Surge",
    kind: Overclock,
    description: "Perfect bump sends a shockwave that damages adjacent cells",
    effect: Shockwave(range: 64.0),
    trigger: Chain(
        start: OnPerfectBump,
        then: OnImpact,
    ),
)
```

## Breaker Selection
- Pre-run screen: choose your breaker (Aegis / Chrono for the slice)
- Breaker abilities are TBD beyond bolt-lost behavior — architecture must be flexible for future abilities
- Validates the composable breaker architecture early

## Run Structure
- **Linear sequence seeded by a run seed** — deterministic given a seed, enabling shareable/replayable runs
- 3-5 nodes for the slice
- Basic difficulty scaling: cells get tougher, timer gets shorter
- Run-end screen (win/lose) with basic stats

## Node Types (v1)
- Passive nodes only for the slice (classic breakout)
- 2-3 hand-crafted level layouts

## Starter Upgrades (3 minimum for the slice)

| Name | Kind | Behavior |
|------|------|----------|
| Piercing Shot | Amp | Bolt passes through the first cell it hits |
| Wide Breaker | Augment | Breaker width increased |
| Surge | Overclock | Perfect bump → shockwave on impact (range-based AOE damage) |
