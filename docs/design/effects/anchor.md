# Anchor

Plant mechanic — after the breaker remains stationary for a delay period, it becomes "planted" with boosted bump force and a wider perfect bump window. Moving or dashing cancels the planted state. Creates a "dash → stop → wait → plant → bump → dash" rhythm — the delay adds commitment tension.

For technical details (config struct, components, fire/reverse behavior), see `docs/architecture/effects/effect_reference.md`.

## Ingredients

Quick Stop x2 + Bump Force x2.

## VFX

- While charging: subtle glow beneath breaker (building anticipation)
- When planted: ground-anchor glow locks in with a brief flash
- On bump while planted: concentrated impact flash scaled by the boosted force
- On movement (cancelling plant): anchor glow dissipates
- The visual communicates three states: "charging", "planted and ready", "moving/not anchored"
