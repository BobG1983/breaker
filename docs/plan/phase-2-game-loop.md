# Phase 2: Game Loop & Time Pressure

**Goal**: Turn the sandbox into a game with stakes. A 3-node run with breaker selection, a timer, bolt-lost consequences, node transitions through a placeholder upgrade screen, and game-over on timer expiry.

---

## Breaker Archetypes

Three breakers, each with a callsign, a unique bolt-lost response, and an archetype-exclusive triggered ability.

### Aegis (lives-based)
- **Bolt-lost**: Lose a life. Run ends at zero lives.
- **Triggered ability**: Bump speed modification. Perfect bumps boost bolt speed, weak bumps reduce it (the current universal mechanic becomes Aegis-exclusive).
- **Identity**: Durable, forgiving on bolt-loss but rewards precise bump timing with speed control.

### Chrono (time-penalty)
- **Bolt-lost**: Subtract time from the node timer. Bolt respawns immediately.
- **Triggered ability**: Bump speed modification (same as Aegis). Perfect bumps boost, weak bumps reduce.
- **Identity**: No life limit but every bolt-loss accelerates the timer toward game-over. Same speed-control reward loop as Aegis, different stakes.

### Prism (multi-bolt)
- **Bolt-lost**: Standard respawn (one bolt at a time as baseline — but banked bolts from the ability mean you often have spares).
- **Triggered ability**: Perfect bump spawns an additional bolt immediately — multiple bolts active at the same time. No bump speed modification (bumps are speed-neutral).
- **Identity**: Trades speed control for coverage. Rewards consistent perfect bumps with an escalating swarm of bolts. Harder to control but clears nodes fast.

### Universal mechanics
- **Bump and dash are universal**: All breakers share the core movement, bump, dash/brake/settle mechanics. Archetypes layer on top, never remove.
- **Per-breaker stats**: Each archetype has different base stats (speed, width, bump windows, etc.) loaded from per-breaker RON config.
- **Pre-run selection screen**: Basic breaker-pick screen before the run starts, showing the three archetypes.

---

## Node Timer

- **Countdown per node**: Visible timer counting down during play
- **Timer expiry = game over**: If the timer hits zero, the run ends immediately
- **Visual urgency feedback**: Color shifts, screen effects as time runs low (at minimum a color change on the timer itself; full screen effects can wait for Phase 4)
- Timer duration tunable per-node in layout data

---

## Bolt-Lost Handling

- **BoltLost dispatched as a message**: The physics domain detects bolt-lost and sends the message
- **Breaker-specific response system**: Each archetype's plugin listens for BoltLost and handles its consequence
  - Aegis: decrement lives, respawn bolt (or game over if zero)
  - Chrono: subtract time from timer, respawn bolt
  - Prism: respawn bolt (standard — multi-bolt ability provides its own safety net)
- **Graphical representation per bolt-loss type**: Visual indicator of the consequence (lives icons for Aegis, time-penalty flash for Chrono, bolt count for Prism)

---

## Level Completion & Transitions

- **Node cleared detection**: All target cells destroyed triggers node completion
- **Upgrade selection screen (placeholder)**: After clearing a node, transition to an upgrade pick screen
  - Three upgrade cards displayed side by side
  - Each card: large placeholder graphic, "WIP" description text, neon/cyberpunk-style box border
  - Clicking a card advances to the next node (no actual upgrade effect yet)
  - This is a UI scaffold for Phase 3's real upgrade system
- **3-node run**: Three distinct hand-authored layouts per run, proving the full transition loop
- **Run-end screen**: Basic win/lose screen after the third node or on game over

---

## Level Loading

- **Hand-authored RON layouts**: 3+ grid layouts authored by hand in RON files
  - Explicit cell positions, types (standard/tough), and health values
  - Eventually a dedicated editor tool, but hand-authored for now
- **Layout selection**: Nodes load layouts in sequence from the run definition

---

## UI

- **Node timer display**: Countdown timer, prominent, with urgency color shift
- **Bolt-loss stakes display**: Visual representation of the current breaker's bolt-loss resource (lives for Aegis, timer penalty for Chrono, active bolt count for Prism)
- **Minimal beyond that**: No score counter, no node progress indicator yet

---

## Pause Menu

- **Basic pause overlay**: Ability to pause mid-node
- **Resume / quit options**: At minimum, resume play and abandon run

---

## Dev Tooling

- **CLI test-level spawning** (dev/debug mode only): Command-line argument to skip menus and spawn directly into a specific test level layout, speeding up iteration
- Lives in the debug domain alongside existing debug overlays

---

## Summary Checklist

- [ ] Breaker archetype system (polymorphic bolt-lost, per-breaker stats, per-breaker RON)
- [ ] Aegis breaker (lives-based bolt-lost, bump speed modification)
- [ ] Chrono breaker (time-penalty bolt-lost, bump speed modification)
- [ ] Prism breaker (multi-bolt on perfect bump, speed-neutral bumps)
- [ ] Pre-run breaker selection screen
- [ ] Node timer (countdown, game-over on expiry, urgency color)
- [ ] Bolt-lost handling delegated to archetype
- [ ] Bolt-loss visual indicator per archetype
- [ ] Node completion detection (all target cells destroyed)
- [ ] Placeholder upgrade selection screen (3 cards, WIP, neon-style borders)
- [ ] 3-node run with sequential layout loading
- [ ] Hand-authored RON level layouts (3+)
- [ ] Run-end screen (win/lose)
- [ ] Pause menu (pause, resume, quit)
- [ ] CLI test-level spawning (debug/dev mode)
