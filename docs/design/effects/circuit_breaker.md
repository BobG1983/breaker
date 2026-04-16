# CircuitBreaker

Charge-and-release effect. Counts perfect bumps, then fires a spawn + shockwave burst when the counter fills.

For technical details (config struct, counter mechanics, fire/reverse behavior), see `docs/architecture/effects/effect_reference.md`.

## Ingredients

Feedback Loop x1 + Bump Force x2.

## VFX

- Persistent: Three-node triangle indicator near bolt, rendered as faint connected dots
- Charge: Each perfect bump lights a node (dim → bright)
- On completion: All three nodes flash white-hot (HDR >1.5), collapse inward, circuit closes
- Spawned bolt + shockwave fire with amplified VFX (larger than base shockwave)
- Screen flash + medium shake on circuit close
- The charge phase is subtle; the payoff is dramatic
