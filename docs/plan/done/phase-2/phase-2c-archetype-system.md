# Phase 2c: Archetype System & Aegis

**Goal**: Build the breaker archetype system and prove it works with one real breaker (Aegis).

> Aegis/Chrono/Prism are proof-of-concept designs. The goal is validating the system, not shipping final breakers.

---

## Archetype System

- **Polymorphic bolt-lost dispatch**: Each breaker defines its own consequence via a breaker-specific system that listens for `BoltLost`
- **Triggered abilities**: Archetype-exclusive abilities that fire automatically when a game condition is met
- **Per-breaker base stats**: Each archetype has different base stats (speed, width, dash properties, bump windows) loaded from per-breaker RON config
- **Broader differentiation**: Archetypes differ in moment-to-moment feel (movement, dash, bump), not just bolt-lost consequences
- **Bump speed modification becomes archetype-exclusive**: The current universal bump speed multiplier is removed from the base system and becomes a triggered ability on specific archetypes

---

## Aegis (lives-based) — Proof-of-Concept

- **Bolt-lost**: Lose a life. Run ends at zero lives.
- **Triggered ability**: Bump speed modification — all bump grades boost bolt speed (perfect = large boost, early/late = small boost, no bump = neutral)
- **Identity**: Durable, forgiving on bolt-loss, rewards precise bump timing with speed control
- **Why first**: Simplest bolt-lost behavior (decrement counter). Lowest implementation risk. Validates the dispatch system without coupling to other systems (timer, multi-bolt).

---

## Universal Mechanics

- **Bump and dash are universal**: All breakers share movement, bump, dash/brake/settle. Archetypes layer on top, never remove.
- **Bolt-loss visual indicator**: Per-archetype graphical representation (lives icons for Aegis)

---

## Future: Upgrade Affinities (Phase 6+)

Some breakers may prefer certain types of upgrades (e.g., Prism synergizes with multi-bolt Amps). This is noted for future design but not implemented in Phase 2.

---

## Checklist

- [x] Archetype system (polymorphic bolt-lost dispatch, per-breaker RON config)
- [x] Per-breaker base stats (speed, width, dash, bump windows) loaded from RON
- [x] Remove universal bump speed modification from base system
- [x] Aegis breaker (lives, bump speed triggered ability)
- [x] Bolt-loss visual indicator for Aegis (lives display)
- [x] System validates: adding a new breaker requires only RON data + a bolt-lost system
