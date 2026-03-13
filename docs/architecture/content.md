# Content Identity — Enum Behaviors + RON Instances

> **Status: Not yet implemented.** This document describes the planned upgrade content system for Phase 3+. No upgrade types, registries, or RON content instances exist in the codebase yet. The `upgrades/` domain is a stub.

**Behaviors** are Rust enums. **Content instances** are RON files that compose and tune those behaviors.

```rust
// Behavior types — exhaustive, matchable, compiler-checked
#[derive(Debug, Clone, Deserialize)]
pub enum AmpEffect {
    Piercing(u32),
    DamageBoost(f32),
    SpeedBoost(f32),
    Ricochet(u32),
    SizeBoost(f32),
}

// Content instance — data-driven, no recompile to add
#[derive(Debug, Deserialize)]
pub struct AmpDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub effects: Vec<AmpEffect>,  // Multiple effects per upgrade
    pub rarity: Rarity,
    pub stacks: bool,
}
```

```ron
// assets/amps/voltspike.ron
/* @[brickbreaker::upgrades::AmpDefinition] */
(
    id: "voltspike",
    name: "Voltspike",
    description: "Pierces through cells and hits harder",
    effects: [
        Piercing(1),
        DamageBoost(2.0),
    ],
    rarity: Uncommon,
    stacks: true,
)
```

**Adding new content:** new RON file, no recompile. **Adding new behavior types:** new enum variant, requires recompile (appropriate — new behavior means new code).

Registries (`AmpRegistry`, `AugmentRegistry`, `OverclockRegistry`) are `Resource`s that load and validate all RON definitions at boot. Game logic looks up definitions through the registry, never matches on raw ID strings.

## RON Validation — ron-lsp

Every RON file MUST include a type annotation comment on the first line linking it to the Rust type it deserializes into:

```ron
// assets/amps/voltspike.ron
/* @[brickbreaker::upgrades::AmpDefinition] */
(
    id: "voltspike",
    ...
)
```

[`ron-lsp`](https://github.com/jasonjmcghee/ron-lsp) uses these annotations to validate RON files against actual Rust struct/enum definitions — catching type mismatches, missing fields, and invalid enum variants without running the game. Run `ron-lsp check .` to validate all annotated RON files in bulk.

## Upgrade Application — Components on Entities

When a player selects an upgrade, it becomes a **component on the bolt or breaker entity**. Systems query for specific upgrade components to apply their effects.

```rust
// Active upgrade component on the bolt entity
#[derive(Component)]
pub struct ActiveAmp {
    pub definition: AmpDefinition,
}

// Systems query for active upgrades
fn apply_piercing(
    bolts: Query<(&Bolt, &ActiveAmp)>,
) {
    for (bolt, amp) in &bolts {
        for effect in &amp.definition.effects {
            match effect {
                AmpEffect::Piercing(count) => { /* ... */ }
                _ => {}
            }
        }
    }
}
```

This means upgrades can carry state (remaining pierce count, cooldown timers) and multiple upgrades of the same type stack naturally as multiple components.
