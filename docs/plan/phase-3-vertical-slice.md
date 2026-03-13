# Phase 3: Vertical Slice — Mini-Run

**Goal**: A playable 3-5 node run that proves the architecture and feels like the game.

- **Upgrade system (v1)** — Three categories, each modifying a different part of the system:
  - **Amps**: Passive bolt modifications (speed, size, damage, piercing, ricochet, etc.)
  - **Augments**: Passive breaker modifications (width, speed, bump strength, tilt angles, dash distance, etc.)
  - **Overclocks**: Triggered abilities — effects that fire when a game condition is met (explosion on cell break, multi-bolt on perfect bump, shield on dash, etc.)
  - **Selection screen between nodes**: Pick 1 of 3 random options (can be any mix of Amp/Augment/Overclock). Upgrades **stack** across the run, building synergies.
  - **Timed selection**: Countdown timer on the pick screen. If time expires, **you get nothing**. The tension never stops.
  - Upgrades as Bevy components/systems that modify existing behavior
  - Upgrade definitions in RON data (category, stats, trigger conditions for Overclocks)
- **Breaker selection**:
  - Pre-run screen: choose your breaker (Guardian / Chrono for the slice)
  - Breaker abilities are TBD beyond bolt-lost behavior — architecture must be flexible for future abilities
  - Validates the composable breaker architecture early
- **Run structure**:
  - **Linear sequence seeded by a run seed** — deterministic given a seed, enabling shareable/replayable runs
  - 3-5 nodes for the slice
  - Basic difficulty scaling: cells get tougher, timer gets shorter
  - Run-end screen (win/lose) with basic stats
- **Node types (v1)**:
  - Passive nodes only for the slice (classic breakout)
  - 2-3 hand-crafted level layouts
