# Phase 2b: Run Structure & Node Timer

**Goal**: A playable 3-node loop with time pressure and a game-over state.

---

## Run Structure

- **3-node sequential run**: Clear node → (upgrade screen in 2d) → next node → ... → run complete
- **Run state tracking**: Current node index, run outcome (in progress / won / lost)
- **Node transitions**: On `NodeCleared`, advance to next node or trigger run-won if final node
- **Run-end screen**: Basic win/lose screen showing run outcome

---

## Node Timer

- **Countdown per node**: Visible timer counting down during play
- **Timer expiry = game over**: Timer hits zero → run ends immediately (lost)
- **Timer duration per node**: Tunable in the RON layout data (different nodes can have different time limits)
- **Visual urgency feedback**: At minimum, timer text changes color as time runs low (full screen effects deferred to Phase 4)
- **Timer UI**: Prominent countdown display

---

## Checklist

- [ ] Run state machine (node sequencing, win/lose tracking)
- [ ] Node transition flow (NodeCleared → next node or run-won)
- [ ] Node timer (countdown, per-node duration from layout data)
- [ ] Timer expiry triggers game over
- [ ] Timer UI with urgency color shift
- [ ] Run-end screen (win/lose)
