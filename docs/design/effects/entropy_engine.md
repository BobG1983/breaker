# EntropyEngine

Escalating chaos — fires multiple random effects on the primary bolt per cell destroyed.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `max_effects` | `u32` | Maximum effects fired per cell destroyed |
| `pool` | `Vec<(f32, EffectNode)>` | Weighted pool of effects to choose from |

## Behavior

On each cell destroyed, fires multiple random effects from the weighted pool on the primary bolt entity. The number of effects scales with the kill count within the current node, up to `max_effects`. Resets between nodes.

This is an evolution (combined from ingredients) — significantly more powerful than `RandomEffect`.

## Reversal

No-op. Inner effects handle their own reversal.
