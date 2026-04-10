# RON Deserializing

## Strategy

Add `#[derive(Deserialize)]` to every type in the effect tree. No raw types, no custom deserializers, no validation layer. RON 0.12 + serde handles everything.

## Types that need Deserialize

All of these get `#[derive(Deserialize)]`:

### Tree types
- `RootNode`
- `Tree`
- `ScopedTree`
- `Terminal`
- `ScopedTerminal`

### Enums
- `StampTarget`
- `EffectType`
- `ReversibleEffectType`
- `Trigger`
- `Condition`
- `EntityKind`
- `AttractionType`
- `RouteType`
- `ParticipantTarget`
- `BumpTarget`
- `ImpactTarget`
- `DeathTarget`
- `BoltLostTarget`

### All 30 config structs
Every config struct in `rust-types/configs/`. They also need `#[derive(Clone, PartialEq, Eq)]` for EffectStack matching, and `OrderedFloat<f32>` for all float fields.

## What does NOT need Deserialize

- `TriggerContext` — runtime only, never in RON
- `BoundEffects`, `StagedEffects`, `SpawnStampRegistry` — runtime storage
- `EffectStack<T>` — runtime storage
- All effect components (ShockwaveSource, AnchorActive, etc.) — runtime only
- All messages (EffectTimerExpired, etc.) — runtime only
- Fireable, Reversible, PassiveEffect — traits, not data

## OrderedFloat is transparent

`OrderedFloat<f32>` serializes/deserializes as a bare number. RON authors write `multiplier: 1.5`, not `multiplier: OrderedFloat(1.5)`. The wrapper is invisible in RON syntax.

## Named config structs in RON

Enum newtype variants wrapping structs use named syntax:
```ron
Shockwave(ShockwaveConfig(base_range: 32.0, range_per_level: 8.0, stacks: 1, speed: 400.0))
```

The outer parens are the enum variant, the inner `TypeName(fields)` is the named struct. RON 0.12 accepts this with bare `#[derive(Deserialize)]`.

## Validated

All 56 test entries (every node type, every config, every enum variant) deserialize successfully with ron 0.12 + bare serde derive. See the test project at `/tmp/ron_test_project/` for the validation code.
