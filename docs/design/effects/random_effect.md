# RandomEffect

Weighted random selection from a pool of effects.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `pool` | `Vec<(f32, EffectNode)>` | Weighted pool of effects. Each entry is `(weight, effect_node)`. |

## Behavior

Selects one effect from the pool using weighted random selection. Fires the selected effect on the entity. Weights are relative — `(2.0, A), (1.0, B)` gives A a 2/3 chance.

## Reversal

Reverses the last randomly selected effect.
