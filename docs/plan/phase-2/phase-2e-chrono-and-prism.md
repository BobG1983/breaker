# Phase 2e: Chrono & Prism

**Goal**: Add the second and third proof-of-concept breakers using the archetype system built in 2c.

> These are proof-of-concept designs. Final shipped breakers may differ significantly.

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
- **Identity**: Trades speed control for coverage. Rewards consistent perfect bumps with an escalating swarm. Harder to control but clears nodes fast.
- **Unique base stats**: Different speed/width/bump windows than Aegis and Chrono

---

## Bolt-Loss Visual Indicators

- **Chrono**: Time-penalty flash or timer highlight on bolt-loss
- **Prism**: Active bolt count display

---

## Checklist

- [ ] Chrono breaker (time penalty bolt-lost, bump speed triggered ability)
- [ ] Chrono per-breaker RON config with unique base stats
- [ ] Prism breaker (multi-bolt on perfect bump, speed-neutral)
- [ ] Prism per-breaker RON config with unique base stats
- [ ] Bolt-loss visual indicator for Chrono
- [ ] Bolt-loss visual indicator for Prism
- [ ] All three breakers playable with distinct feel
