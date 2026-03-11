# Brickbreaker Roguelite — High-Level Build Plan

## Context
Building a roguelite Arkanoid clone in Bevy (Rust). Solo dev, architecture-first approach validated through a vertical slice. Stylized shader-driven visuals, custom physics, hybrid data model (mechanics in code, content in data files), audio planned from the start.

See `DESIGN.md` for core design principles, `ARCHITECTURE.md` for technical decisions, and `TERMINOLOGY.md` for game vocabulary.

---

## Phase 0: Project Scaffolding & Architecture
**Goal**: Establish the Bevy plugin architecture and foundational systems before any gameplay code.

- **Project structure**: Workspace with a clear plugin/module boundary per system (e.g., `physics/`, `breaker/`, `bolt/`, `cells/`, `upgrades/`, `audio/`, `ui/`, `progression/`, `run/`)
- **Game state machine**: Define top-level states (MainMenu, RunSetup, Playing, UpgradeSelect, Paused, RunEnd, MetaProgression) using Bevy States
- **Event bus design**: Central events that decouple systems (BoltHitBreaker, BoltHitCell, CellDestroyed, BoltLost, NodeCleared, UpgradeSelected, TimerExpired, BumpPerformed { grade }, etc.)
- **Data layer**: Set up RON/asset loading for content definitions (cell types, upgrade definitions, level layouts, difficulty curves)
- **Camera & coordinate system**: Define the playfield dimensions, coordinate space, and camera setup
- **Debug tooling**: Basic debug overlays (hitboxes, velocity vectors, state display, FPS) that can be toggled — invest early, pays off throughout

---

## Phase 1: Core Mechanics (The Feel)
**Goal**: Breaker + bolt + cells that feel *right*. This is the foundation everything else sits on. Spend the most time here.

### 1a: Breaker / Breaker System
- **Breaker as a composable identity**: Each breaker defines its own bolt-lost behavior, base stats, and optional unique ability. This is a core architectural pattern, not a late addition.
- Two breakers for the vertical slice:
  - **Guardian**: Lives-based (3 lives, lose one per bolt-lost)
  - **Chrono**: Time-penalty-based (bolt-lost costs time off the clock)
- Base breaker mechanics shared by all breakers:
  - **Horizontal movement**: Smooth, responsive left/right movement. This is the primary movement mode.
  - **Bump**: Button press causes the breaker to jump upward slightly, transferring vertical velocity to the bolt. This is the core skill mechanic for controlling bolt trajectory.
  - **Dash**: Burst of horizontal speed. Breaker **tilts in the direction of movement** during dash, which affects bolt bounce angles. Cannot initiate a new dash while one is ongoing — must brake and settle first. Dashing is deliberately *no better or worse* than normal movement for general play; its value comes from emergency repositioning and from combining with perfect bumps.
  - **Brake**: Rapid deceleration from dash. Breaker **tilts back hard** in the opposite direction, providing another angle-control window. Hard slowdown.
  - **Settle**: Return to neutral state after braking. Breaker tilt returns to flat. Only after settling can you dash again.
  - **Breaker state machine**: Idle → Dashing → Braking → Settling → Idle (with normal movement available in Idle and Settling states)
- Breaker-specific parameters (breaker width, speed, bump strength, tilt angles, reflection behavior) defined in data
- Breaker hitbox and visual representation (placeholder shader rectangle, breaker-tinted, tilt visible)

### 1b: Bolt
- Custom physics: velocity-based movement, no engine physics
- Collision detection: breaker, cells, walls, ceiling
- **Breaker reflection model**: Direction entirely overwritten on breaker contact based on:
  - Hit position on breaker (left/right of center → angle)
  - Breaker tilt state (dashing/braking tilt modifies the effective surface angle)
  - Bump grade (perfect/early/late/none → velocity magnitude)
  - No incoming angle carryover. No perfectly vertical or horizontal reflections.
- Speed management: base speed, speed caps, speed modifications from upgrades and bump grade
- Bolt-lost detection (falls below breaker)

### 1c: Cells
- Standard cells (1 hit)
- Tough cells (N hits, visual feedback on damage)
- Cell grid layout system driven by data
- Cell destruction effects (placeholder particles/flash)

### 1d: Bump & Perfect Bump System
- **Bump**: Player presses bump button → breaker pops upward briefly → any bolt contacting the breaker during the bump receives upward velocity. This is how players actively control the bolt beyond just positioning.
- **Bump timing grades**:
  - **Early bump**: Bump pressed too early relative to bolt contact. Visual indicator (e.g., flash/color) shows it was early. Reduced or default bolt velocity transfer.
  - **Perfect bump**: Bump timed within a tight window around bolt impact. Strong visual/audio feedback (spark, screen flash, satisfying sound). Amplified velocity transfer, enhanced trajectory control. **Can cancel an ongoing dash** — key high-skill interaction.
  - **Late bump**: Bump pressed too late. Visual indicator shows it was late. Reduced or default bolt velocity transfer.
  - **No bump**: Bolt bounces off breaker passively with default reflection. No upward velocity bonus.
- Timing window parameters tunable in data (perfect window size, early/late windows, velocity multipliers per grade)
- The bump + dash-tilt + brake-tilt system creates a rich control surface: position the breaker (movement), angle the surface (dash/brake tilt), and control velocity (bump timing)

---

## Phase 2: Game Loop & Time Pressure
**Goal**: Turn the sandbox into a game with stakes.

- **Level timer**: Countdown per node, visual urgency feedback (color shifts, screen effects as time runs low)
- **Bolt-lost handling**: Delegates to the active breaker's trait (Guardian: lose a life, Chrono: lose time). System dispatches a `BoltLost` event, breaker-specific system handles the consequence.
- **Level completion detection**: All target cells destroyed → level cleared
- **Level transition flow**: Level cleared → upgrade selection → next level loads
- **Basic level loading**: Read level layouts from RON data files, spawn cell grids

---

## Phase 3: Vertical Slice — Mini-Run
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

---

## Phase 4: Visual Identity
**Goal**: Establish the stylized, shader-driven aesthetic. Not polish — identity.

- **Breaker shader**: Glow, trail effect on dash, tilt visualization, visual feedback on brake/settle, bump flash
- **Bolt shader**: Glow, motion trail, impact flash
- **Cell shaders**: Damage states, destruction dissolve/shatter
- **Background**: Animated shader background that responds to game state (intensity increases with urgency)
- **Screen effects**: Screen shake on impacts, chromatic aberration on big hits, vignette on low timer
- **Particle system**: Cell destruction particles, perfect bump sparks, upgrade pickup effects
- **UI rendering**: Timer display, score, upgrade icons — shader-driven or clean minimalist

---

## Phase 5: Audio Foundation
**Goal**: Audio system architecture + placeholder sounds that establish the feel.

- **Audio plugin**: Centralized audio system that responds to game events
- **Sound categories**: SFX (breaker hit, cell break, perfect bump, upgrade), ambient (background hum/music layer), UI (menu navigation, selection)
- **Adaptive audio**: Music/ambient intensity tied to game state (timer urgency, combo streaks)
- **Placeholder sounds**: Generate or source basic sound effects to validate the system
- **Volume/mixing**: Per-category volume controls

---

## Phase 6: Content & Variety
**Goal**: Expand from vertical slice to real content depth.

- **More cell types**:
  - Twin cells (linked destruction)
  - Gate cells (unlock by clearing neighbors)
  - Portal cells (sub-level mechanic — this is complex, may defer)
- **More upgrades** (10-15 total across all three categories):
  - Synergy system: Amps, Augments, and Overclocks that interact with each other
  - Trade-off upgrades (benefit + cost)
  - Rarity/weighting per category
- **Active nodes**:
  - Cells that move or regenerate
  - Hazards that attack the breaker
  - Retaliating cells
- **Level generation**:
  - Procedural level layouts (or large hand-crafted pool) with difficulty parameters
  - Node type distribution curve (more passive early, more active late)

---

## Phase 7: Roguelite Progression
**Goal**: The meta-layer that keeps players coming back.

- **Run structure expansion**:
  - Linear seeded runs, but vary node type composition per seed (passive, active, alpha, omega mix)
  - Shop/rest nodes between levels as new node types
  - Seed-based leaderboards / sharing
- **Meta-progression**:
  - Persistent currency/XP earned per run
  - Unlockable Amps/Augments/Overclocks added to the pool
  - Permanent buffs (subtle, don't trivialize core)
  - Unlockable breakers (alternate breakers with unique abilities)
- **Alternate Breakers**:
  - Precision, Berserker, Momentum archetypes
  - Each modifies breaker physics and has a unique ability
- **Save system**: Persist meta-progression state between sessions

---

## Phase 8: Boss Nodes & Advanced Mechanics
**Goal**: The endgame content that tests mastery.

- **Alpha Nodes**: Denser layouts, tougher active mechanics, mini-boss feel
- **Omega Nodes**: Multi-phase boss encounters with escalating mechanics
- **Advanced bolt mechanics**: If needed — curve shots, charge shots, etc.
- **Difficulty modes**: Selectable starting difficulty with reward multipliers

---

## Phase 9: Polish & Ship
**Goal**: Make it a complete, shippable experience.

- **Main menu & settings**: Resolution, audio, controls, accessibility
- **Run stats & history**: Per-run breakdown, best times, favorite builds
- **Tutorial / onboarding**: Teach breaker mechanics gradually
- **Visual polish**: Refine all shaders, particles, transitions
- **Audio polish**: Final sound design, music composition/commissioning
- **Performance**: Profile and optimize (Bevy ECS should help here)
- **Playtesting**: Balance passes on difficulty curves, upgrade synergies, timer values
- **Distribution**: Build pipeline for target platform(s)

---

## Build Order Rationale

The plan front-loads **feel** (Phases 1-2) because if the breaker-bolt-cell interaction isn't satisfying, nothing else matters. The vertical slice (Phase 3) validates architecture under real gameplay conditions before investing in content. Visuals and audio (Phases 4-5) come before content expansion because the stylized aesthetic is part of the game's identity and informs how content is designed. Roguelite systems (Phase 7) are deliberately late — they're important for retention but meaningless without solid core gameplay.

---

## Open Questions (to resolve during development)
1. **Portal cell scope**: Sub-levels are a game-within-a-game. How complex? Defer to Phase 8?
2. **Breaker ability design**: Beyond bolt-lost behavior, what active abilities do breakers have? (Defined per-breaker in data?)
