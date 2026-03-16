# Phase 2e: Visual Polish & Additional Archetypes

**Goal**: Visual interpolation for smooth rendering, plus Chrono and Prism breakers to fill out the archetype system.

---

## Visual Interpolation

- **Transform interpolation**: Smooth bolt and breaker movement between FixedUpdate ticks
- **Why here**: Without this, movement appears jerky at 64Hz fixed vs 60Hz+ display. Critical for the "speed, juice, adrenaline" identity.

---

## Chrono (time-penalty)

- **Bolt-lost**: Subtract time from the node timer. Bolt respawns immediately.
- **Triggered ability**: Bump speed modification (same as Aegis — all grades boost, perfect = large)
- **Identity**: No life limit but every bolt-loss accelerates the timer toward game-over
- **Unique base stats**: Different speed/width/bump windows than Aegis (loaded from per-breaker RON)

---

## Prism (multi-bolt)

- **Bolt-lost**: Standard respawn (one bolt baseline — accumulated bolts from the ability provide a safety net)
- **Triggered ability**: Perfect bump spawns an additional bolt immediately — multiple bolts active simultaneously. No bump speed modification (bumps are speed-neutral).
- **Identity**: Trades speed control for coverage. Rewards consistent perfect bumps with an escalating swarm.
- **Unique base stats**: Different speed/width/bump windows than Aegis and Chrono

---

## Bolt-Loss Visual Indicators

- **Chrono**: Time-penalty flash or timer highlight on bolt-loss
- **Prism**: Active bolt count display

---

## Checklist

- [ ] Visual interpolation (smooth bolt/breaker between FixedUpdate ticks)
- [ ] Chrono breaker (time penalty bolt-lost, bump speed triggered ability)
- [ ] Chrono per-breaker RON config with unique base stats
- [ ] Prism breaker (multi-bolt on perfect bump, speed-neutral)
- [ ] Prism per-breaker RON config with unique base stats
- [ ] Bolt-loss visual indicator for Chrono
- [ ] Bolt-loss visual indicator for Prism
- [ ] All three breakers playable with distinct feel
