# Phase 2d: Screens & UI

**Goal**: The screens and UI that connect the game loop into a playable experience.

---

## Breaker Selection Screen

- **Pre-run screen**: Choose from the three breakers (Aegis, Chrono, Prism) before the run starts
- **Display per breaker**: Callsign, brief description of bolt-lost behavior and triggered ability
- **Validates the full flow**: Main menu → breaker select → run → (nodes) → run-end → main menu

---

## Placeholder Upgrade Selection Screen

- **Between nodes**: After clearing a node, transition to an upgrade pick screen
- **Three upgrade cards side by side**: Each card has:
  - Large placeholder graphic
  - "WIP" description text
  - Neon/cyberpunk-style box border
- **Clicking a card advances to the next node** (no actual upgrade effect yet)
- **UI scaffold for Phase 3**: The real upgrade system (Amps/Augments/Overclocks) gets wired in later

---

## Pause Menu

- **Basic pause overlay**: Ability to pause mid-node
- **Resume / quit options**: At minimum, resume play and abandon run (return to main menu)

---

## In-Game UI

- **Node timer display**: Prominent countdown (built in 2b, may need refinement here)
- **Bolt-loss stakes display**: Per-archetype visual — lives for Aegis, timer-penalty indicator for Chrono, active bolt count for Prism
- **Minimal beyond that**: No score counter, no node progress indicator yet

---

## Checklist

- [ ] Breaker selection screen (3 breakers, descriptions, selection flow)
- [ ] Placeholder upgrade selection screen (3 cards, WIP, neon borders, clickable)
- [ ] Pause menu (pause, resume, quit to menu)
- [ ] Bolt-loss stakes display per archetype
- [ ] Full flow: menu → select breaker → run nodes → upgrade screens → run-end → menu
