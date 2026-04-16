# FlashStep

Breaker teleport on dash reversal during settling. Rewards players who can read the bolt's trajectory and react with a direction reversal — skipping the settling penalty entirely.

For technical details (config struct, fire/reverse behavior), see `docs/architecture/effects/effect_reference.md`.

## Ingredients

Breaker Speed x2 + Reflex x1.

## VFX

- On teleport trigger: Breaker disintegrates into energy streak particles at departure point
- Light-streak connects departure and arrival positions (1-2 frames)
- Departure afterimage fades ~0.3s
- Arrival: radial distortion burst + rematerialization (particles converge to form breaker)
- Small screen shake on arrival
