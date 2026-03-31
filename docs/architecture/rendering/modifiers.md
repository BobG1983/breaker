# Visual Modifier System

The VFX crate owns the modifier system. Gameplay domains communicate dynamic visual state entirely through modifier messages. There are no `*RenderState` bridge components.

**Modifiers work on any entity** registered via `AttachVisuals` — bolts, breakers, cells, walls. The crate doesn't care what kind of entity it is. Example: Powder Keg chip sends `AddModifier(AlphaOscillation { ... })` to a cell entity for flickering.

## Three Message Types

```rust
/// Overwrite a modifier by source key. Used for per-frame dynamic state.
#[derive(Message, Clone)]
pub struct SetModifier {
    pub entity: Entity,
    pub modifier: VisualModifier,
    pub source: &'static str,
}

/// Add a stacking modifier. Used for chip effects. Stacks with DR.
#[derive(Message, Clone)]
pub struct AddModifier {
    pub entity: Entity,
    pub modifier: VisualModifier,
    pub source: &'static str,
}

/// Remove a modifier by source key.
#[derive(Message, Clone)]
pub struct RemoveModifier {
    pub entity: Entity,
    pub source: &'static str,
}
```

- `SetModifier`: same source key overwrites. Used for per-frame dynamic state (speed → trail length).
- `AddModifier`: stacks with diminishing returns. Used for chip effects (fired once, removed on reverse).
- `RemoveModifier`: removes by source key. Used when chip effects reverse.

**`source: &'static str`** is intentional — source keys are string literals (`"bolt_speed"`, `"speed_boost"`, etc.), not runtime-generated strings. This avoids per-frame `String` allocation for `SetModifier` messages sent every FixedUpdate. All source keys are known at compile time. If a future need arises for dynamic source keys, the type can be changed to `Cow<'static, str>`.

## How Gameplay Uses It

```rust
// bolt/ domain, every FixedUpdate — dynamic state (overwrites each frame):
world.send(SetModifier {
    entity: bolt,
    modifier: VisualModifier::TrailLength(speed / max_speed * 2.0),
    source: "bolt_speed",
});

// effect/ domain, when SpeedBoost fires — chip effect (stacks):
world.send(AddModifier {
    entity: bolt,
    modifier: VisualModifier::TrailLength(1.5),
    source: "speed_boost",  // one entry per effect kind, not per chip instance
});

// effect/ domain, when SpeedBoost reverses:
world.send(RemoveModifier {
    entity: bolt,
    source: "speed_boost",  // one entry per effect kind, not per chip instance
});
```

## Stacking Semantics by Modifier Type

| Modifier Type | Stacking | Diminishing Returns |
|---------------|----------|---------------------|
| `TrailLength(f32)` | Numeric — values multiply | Yes (DR curve) |
| `GlowIntensity(f32)` | Numeric — values multiply | Yes |
| `CoreBrightness(f32)` | Numeric — values multiply | Yes |
| `HaloRadius(f32)` | Numeric — values multiply | Yes |
| `ShapeScale(f32)` | Numeric — values multiply | Yes |
| `SpikeCount(u32)` | Numeric — values sum | Yes |
| `RotationSpeed(f32)` | Numeric — values sum | Yes |
| `ColorShift(Hue)` | **Latest wins** — no stacking | No |
| `ColorCycle { .. }` | **Latest wins** — no stacking | No |
| `AlphaOscillation { .. }` | **Latest wins** — no stacking | No |
| `AfterimageTrail(bool)` | **Any true = enabled** | No |
| `SquashStretch { .. }` | **Latest wins** — no stacking | No |

Non-numeric modifiers (ColorShift, ColorCycle, AlphaOscillation, SquashStretch) use "latest wins" — the most recently added modifier of that kind takes effect. They are exempt from diminishing returns.

## Diminishing Returns

Applied to numeric `AddModifier` stacks only (NOT to `SetModifier`, NOT to non-numeric modifiers).

Per-modifier-type curves configured by the game:

```rust
#[derive(Resource)]
pub struct ModifierConfig {
    pub curves: HashMap<ModifierKind, Vec<f32>>,
    pub default_curve: Vec<f32>,
}

// Game configures at startup:
app.insert_resource(ModifierConfig {
    curves: HashMap::from([
        (ModifierKind::TrailLength, vec![1.0, 0.7, 0.4, 0.2]),
        (ModifierKind::GlowIntensity, vec![1.0, 0.8, 0.6, 0.4]),
    ]),
    default_curve: vec![1.0, 0.7, 0.4, 0.2],
});
```

Values beyond the vec length use the last entry.

**Key rule**: Diminishing returns are visual only. Gameplay multipliers are unaffected.

## Modifier Computation

The crate maintains a `ModifierStack` component on each entity registered for modifier tracking. Each frame, the crate:

1. Reads all `SetModifier` entries (latest value per source key)
2. Reads all `AddModifier` entries (stacked with DR)
3. For each `VisualModifier` kind: combines Set value with DR-scaled Add values
4. Updates the entity's shader uniforms from the computed result

## Entity Cleanup

Modifier tracking uses a `Component` on the entity. When the entity despawns, Bevy's standard cleanup removes the component. Aura and trail child entities are also cleaned up by Bevy's hierarchy system. No explicit `DetachVisuals` message needed.
