# Phase 2d: Screens & UI

**Goal**: The screens and UI that connect the game loop into a playable experience. Built after the archetype system (2c) so breaker selection UI is designed with a concrete archetype in hand.

---

## Breaker Selection Screen

- **Pre-run screen**: Choose from available breakers before the run starts
- **Display per breaker**: Callsign, brief description of bolt-lost behavior and triggered ability
- **Initially one breaker (Aegis)**: Selection screen works with one, scales to three when 2e ships
- **Validates the full flow**: Main menu → breaker select → run → (nodes) → run-end → main menu

---

## Placeholder Upgrade Selection Screen

- **Between nodes**: After clearing a node, transition to an upgrade pick screen
- **Three upgrade cards side by side**: Each card has:
  - Large placeholder graphic
  - "WIP" description text
  - Neon/cyberpunk-style box border
- **Clicking a card advances to the next node** (no actual upgrade effect yet)
- **Timer on selection screen**: Countdown to make a choice. Timer expires = you get nothing (skip). Maximum pressure, fits the design pillars.
- **UI scaffold for Phase 3**: The real upgrade system (Amps/Augments/Overclocks) gets wired in later

---

## Pause Menu

- **Basic pause overlay**: Ability to pause mid-node
- **Resume / quit options**: At minimum, resume play and abandon run (return to main menu)

---

## In-Game UI

- **Node timer display**: Prominent countdown (built in 2b, may need refinement here)
- **Bolt-loss stakes display**: Per-archetype visual — lives for Aegis (others added in 2e)
- **Visual interpolation**: Transform interpolation between FixedUpdate ticks for bolt and breaker. Without this, movement appears jerky at 64Hz fixed vs 60Hz+ display. Critical for the "speed, juice, adrenaline" identity.
- **Minimal beyond that**: No score counter, no node progress indicator yet

---

## Checklist

- [ ] Breaker selection screen (works with 1 breaker, scales to 3)
- [ ] Placeholder upgrade selection screen (3 cards, WIP, neon borders, clickable)
- [ ] Upgrade selection timer (expires = skip, no upgrade)
- [ ] Pause menu (pause, resume, quit to menu)
- [ ] Bolt-loss stakes display (Aegis lives)
- [ ] Visual interpolation (smooth bolt/breaker between FixedUpdate ticks)
- [ ] Full flow: menu → select breaker → run nodes → upgrade screens → run-end → menu
