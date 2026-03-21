# Entity Scale

**Decision**: Per-layout breaker/bolt scaling via `entity_scale` (0.5..=1.0).

## Mechanic

Any node layout can optionally specify `entity_scale` to shrink the breaker and bolt proportionally. Both visual size (`Transform.scale`) AND collision hitboxes (`BreakerWidth`, `BreakerHeight`, `BoltRadius`) scale together — no visual-only tricks.

- Defaults to `1.0` (no scaling) for all layouts
- Minimum floor at `0.5` — below this, the bolt becomes visually illegible (~4px) and gameplay becomes "cheap" not "hard"

## Core Rule: Speed Is Constant

Bolt speed and breaker movement speed are **NOT** affected by entity scale. This is the central design decision: smaller hitboxes at the same speed = tighter gameplay. The breaker covers less area but moves just as fast, so the player must position more precisely. The bolt passes through gaps more easily but is harder to catch.

## Stacking with Chips

Entity scale applies as a final multiplier on the total (base + boost):

```
effective_width = (base_width + width_boost) * entity_scale
effective_radius = bolt_radius * entity_scale
```

This creates emergent chip synergies:
- **WidthBoost** counters the scale penalty — strategically valuable for boss encounters
- **Piercing** hits more cells per traversal in dense scaled grids
- **Shockwave** hits more cells in tight formations

## Recommended Ranges

| Layout Pool | entity_scale | Feel |
|-------------|-------------|------|
| Passive | 1.0 (default) | Standard |
| Active | 0.85–1.0 | Subtle tightening |
| Boss | 0.6–0.8 | Noticeably smaller, genuine skill test |

## Rationale

Entity scale is infrastructure for boss encounters and progressive difficulty. A boss arena at `0.7` with a dense 40×25 grid feels mechanically distinct — the player is "zoomed out," the breaker is small, and every catch requires precision. This reinforces Pillar 1 (The Escalation) and Pillar 3 (Mechanical Floor, Strategic Ceiling) without adding new mechanics — it recontextualizes existing ones.
