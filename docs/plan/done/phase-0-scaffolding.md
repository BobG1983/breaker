# Phase 0: Project Scaffolding & Architecture

**Goal**: Establish the Bevy plugin architecture and foundational systems before any gameplay code.

- **Project structure**: Workspace with a clear plugin/module boundary per system (e.g., `physics/`, `breaker/`, `bolt/`, `cells/`, `upgrades/`, `audio/`, `ui/`, `progression/`, `run/`)
- **Game state machine**: Define top-level states (MainMenu, RunSetup, Playing, UpgradeSelect, Paused, RunEnd, MetaProgression) using Bevy States
- **Event bus design**: Central events that decouple systems (BoltHitBreaker, BoltHitCell, CellDestroyed, BoltLost, NodeCleared, UpgradeSelected, TimerExpired, BumpPerformed { grade }, etc.)
- **Data layer**: Set up RON/asset loading for content definitions (cell types, upgrade definitions, level layouts, difficulty curves)
- **Camera & coordinate system**: Define the playfield dimensions, coordinate space, and camera setup
- **Debug tooling**: Basic debug overlays (hitboxes, velocity vectors, state display, FPS) that can be toggled — invest early, pays off throughout

## What actually shipped

Beyond the plan:
- GameConfig derive macro eliminating duplicate config/defaults structs
- RON asset loading pipeline with loading screen
- SystemSet-based ordering for cross-domain dependencies
- Message-driven architecture (Bevy 0.18 messages, not events)
- Main menu screen with keyboard/mouse navigation
- Comprehensive debug UI (hitbox gizmos, velocity arrows, bump info, input actions window)
