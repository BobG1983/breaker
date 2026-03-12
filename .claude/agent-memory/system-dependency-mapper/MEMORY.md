# System Dependency Mapper Memory

## Bevy Version
**0.18.1** — uses `#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>` for inter-system comms.

## Current Files
- [system-map.md](system-map.md) — Full system inventory: every system, plugin, schedule, ordering, data access
- [message-flow.md](message-flow.md) — Message flow map: who sends what, who receives what, cross-plugin boundaries
- [known-conflicts.md](known-conflicts.md) — Identified conflicts, ordering issues, missing constraints

## Key Architectural Facts
- All gameplay systems run in FixedUpdate (physics, breaker, bolt, cells) gated by `run_if(in_state(PlayingState::Active))`
- Visual-only systems run in Update (animate_bump_visual, animate_fade_out, debug overlays)
- Physics chain: `prepare_bolt_velocity` → `bolt_cell_collision` → `bolt_breaker_collision` → `bolt_lost`
- Breaker state chain: `update_bump` → `move_breaker` (BreakerSystems::Move) → `update_breaker_state` → `grade_bump` → `perfect_bump_dash_cancel`
- Bolt systems after breaker move: `hover_bolt`, `prepare_bolt_velocity` both `.after(BreakerSystems::Move)`
- Physics after bolt: all three collision systems `.after(BoltSystems::PrepareVelocity)`
- Input: `read_input_actions` in PreUpdate (after InputSystems) writes InputActions resource consumed by FixedUpdate gameplay systems

## Known Conflict (Action Required)
`apply_bump_velocity` (BoltPlugin, FixedUpdate) writes `BoltVelocity` but has NO ordering constraint relative to the physics chain (`bolt_cell_collision`, `bolt_breaker_collision`, `bolt_lost`). This means bump velocity amplification could be applied before CCD physics runs, getting overwritten. Fix: add `.after(bolt_breaker_collision)` in BoltPlugin registration.

## Stub Domains (No Systems Yet)
AudioPlugin, RunPlugin, UiPlugin, UpgradesPlugin — registered but contain no game systems as of Phase 0/1.

## Message Consumers Still Missing (Expected — Future Phases)
CellDestroyed, BoltLost (penalty), BoltHitBreaker (audio/upgrades/UI), NodeCleared, TimerExpired, UpgradeSelected.
