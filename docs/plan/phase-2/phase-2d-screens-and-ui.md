# Phase 2d: Screens

**Goal**: The three screens that complete the game loop: breaker selection (RunSetup), placeholder upgrade selection (UpgradeSelect), and pause menu. Visual interpolation is deferred to 2e.

---

## Breaker Selection Screen (RunSetup)

- **Pre-run screen**: Choose from available breakers before the run starts
- **Display per breaker**: Callsign, brief description of bolt-lost behavior and triggered ability
- **Initially one breaker (Aegis)**: Selection screen works with one, scales to three when 2e ships
- **Flow change**: Main menu Play → RunSetup → Playing (was Main menu Play → Playing)

---

## Placeholder Upgrade Selection Screen (UpgradeSelect)

- **Between nodes**: After clearing a node, transition to upgrade pick screen instead of directly to next node
- **Three placeholder cards side by side**: WIP text, neon-style borders
- **Clicking a card or timer expiry advances to the next node** (no actual upgrade effect yet)
- **Countdown timer**: Timer expires = skip (no upgrade). Maximum pressure.
- **Flow change**: NodeTransition → UpgradeSelect → Playing (was NodeTransition → Playing)

---

## Pause Menu

- **Basic pause overlay**: Escape key toggles pause mid-node
- **Resume / quit options**: Resume play or abandon run (return to main menu)
- **Uses PlayingState sub-state**: Active ↔ Paused

---

## Checklist

- [ ] Breaker selection screen (RunSetup) — works with 1 breaker, scales to 3
- [ ] Main menu Play → RunSetup flow change
- [ ] Placeholder upgrade selection screen (3 cards, WIP, countdown timer)
- [ ] NodeTransition → UpgradeSelect flow change
- [ ] Pause menu (Escape toggle, resume, quit to menu)
- [ ] Full flow: menu → select breaker → run nodes → upgrade screens → run-end → menu
