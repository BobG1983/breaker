# Phase 2d: Screens

**Goal**: The three screens that complete the game loop: breaker selection (RunSetup), placeholder chip selection (ChipSelect), and pause menu. Visual interpolation is deferred to 2e.

---

## Breaker Selection Screen (RunSetup)

- **Pre-run screen**: Choose from available breakers before the run starts
- **Display per breaker**: Callsign, brief description of bolt-lost behavior and triggered ability
- **Initially one breaker (Aegis)**: Selection screen works with one, scales to three when 2e ships
- **Flow change**: Main menu Play → RunSetup → Playing (was Main menu Play → Playing)

---

## Chip Selection Screen (ChipSelect)

- **Between nodes**: After clearing a node, transition to chip pick screen instead of directly to next node
- **Three chip cards side by side**: Chip name and description, neon-style borders
- **Clicking a card or timer expiry advances to the next node** (no actual chip effect yet)
- **Countdown timer**: Timer expires = skip (no chip). Maximum pressure.
- **Flow change**: NodeTransition → ChipSelect → Playing (was NodeTransition → Playing)

---

## Pause Menu

- **Basic pause overlay**: Escape key toggles pause mid-node
- **Resume / quit options**: Resume play or abandon run (return to main menu)
- **Uses PlayingState sub-state**: Active ↔ Paused

---

## Checklist

- [x] Breaker selection screen (RunSetup) — works with 1 breaker, scales to 3
- [x] Main menu Play → RunSetup flow change
- [x] Chip selection screen (3 cards, countdown timer)
- [x] NodeTransition → ChipSelect flow change
- [x] Pause menu (Escape toggle, resume, quit to menu)
- [x] Full flow: menu → select breaker → run nodes → chip screens → run-end → menu
