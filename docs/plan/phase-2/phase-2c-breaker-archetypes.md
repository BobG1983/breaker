# Phase 2c: Breaker Archetypes

**Goal**: Three distinct breakers with polymorphic bolt-lost behavior and archetype-exclusive triggered abilities.

---

## Archetype System

- **Polymorphic bolt-lost**: Each breaker defines its own consequence via a breaker-specific system that listens for `BoltLost`
- **Triggered abilities**: Archetype-exclusive abilities that fire automatically when a game condition is met
- **Per-breaker stats**: Each archetype has different base stats (speed, width, bump windows, etc.) loaded from per-breaker RON config
- **Bump speed modification becomes archetype-exclusive**: The current universal bump speed multiplier is removed from the base system and becomes a triggered ability on Aegis and Chrono only

---

## Aegis (lives-based)

- **Bolt-lost**: Lose a life. Run ends at zero lives.
- **Triggered ability**: Bump speed modification — perfect bumps boost bolt speed, weak bumps reduce it (clamped to base speed floor)
- **Identity**: Durable, forgiving on bolt-loss but rewards precise bump timing with speed control

---

## Chrono (time-penalty)

- **Bolt-lost**: Subtract time from the node timer. Bolt respawns immediately.
- **Triggered ability**: Bump speed modification (same as Aegis)
- **Identity**: No life limit but every bolt-loss accelerates the timer toward game-over

---

## Prism (multi-bolt)

- **Bolt-lost**: Standard respawn (one bolt baseline — but accumulated bolts from the ability provide a safety net)
- **Triggered ability**: Perfect bump spawns an additional bolt immediately — multiple bolts active simultaneously. No bump speed modification (bumps are speed-neutral).
- **Identity**: Trades speed control for coverage. Rewards consistent perfect bumps with an escalating swarm. Harder to control but clears nodes fast.

---

## Universal Mechanics

- **Bump and dash are universal**: All breakers share movement, bump, dash/brake/settle. Archetypes layer on top, never remove.
- **Bolt-loss visual indicator**: Per-archetype graphical representation (lives icons for Aegis, time-penalty flash for Chrono, bolt count for Prism)

---

## Checklist

- [ ] Archetype system (polymorphic bolt-lost dispatch, per-breaker RON config)
- [ ] Remove universal bump speed modification from base system
- [ ] Aegis breaker (lives, bump speed triggered ability)
- [ ] Chrono breaker (time penalty, bump speed triggered ability)
- [ ] Prism breaker (multi-bolt on perfect bump, speed-neutral)
- [ ] Per-breaker stats loaded from RON
- [ ] Bolt-loss visual indicator per archetype
