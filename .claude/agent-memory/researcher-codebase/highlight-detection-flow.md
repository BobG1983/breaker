---
name: highlight-detection-flow
description: End-to-end highlight detection, storage, cap, display, and juice flow map for the run domain
type: reference
---

# Highlight System Flow Map

## Data Types
- `HighlightKind` (enum, 15 variants) — `resources.rs`
- `RunHighlight { kind, node_index, value }` — `resources.rs`
- `HighlightTracker` (resource, per-node + cross-node fields) — `resources.rs`
- `HighlightConfig` (resource, generated from `HighlightDefaults` via `GameConfig` derive) — `definition.rs`
- `HighlightTriggered { kind }` (message) — `messages.rs`

## Detection Systems (6 system functions, 15 kinds)

### Node-clear batch (track_node_cleared_stats) — FixedUpdate, PlayingState::Active
Trigger: `NodeCleared` message. Checks 8 kinds in order:
1. ClutchClear, 2. NoDamageNode, 3. FastClear, 4. PerfectStreak, 5. SpeedDemon, 6. Untouchable, 7. Comeback, 8. PerfectNode
No per-kind dedup — same kind can appear multiple times (but cap blocks).

### Real-time detectors (FixedUpdate, PlayingState::Active)
- `detect_mass_destruction` — trigger: CellDestroyed messages, per-kind dedup (only 1 MassDestruction per run)
- `detect_close_save` — trigger: BumpPerformed messages, per-kind dedup
- `detect_combo_and_pinball` — trigger: CellDestroyed + BoltHitCell + BoltHitBreaker messages, per-kind dedup for both ComboKing and PinballWizard
- `detect_nail_biter` — trigger: NodeCleared messages, per-kind dedup

### Chip-select detector (Update, GameState::ChipSelect)
- `detect_first_evolution` — trigger: ChipSelected messages, per-kind dedup + first_evolution_recorded flag

### Unimplemented
- `MostPowerfulEvolution` — enum variant exists, display text exists, but no detection system

## Storage
All detection writes to `RunStats.highlights: Vec<RunHighlight>`.
Cap checked at detection time: `stats.highlights.len() < config.highlight_cap as usize`.
FIFO order — first detected wins when cap is reached.

## Juice
All detectors emit `HighlightTriggered` message BEFORE checking cap/dedup.
`spawn_highlight_text` (Update, PlayingState::Active) reads these messages and spawns floating Text2d with FadeOut.

## Display
`spawn_run_end_screen` (OnEnter GameState::RunEnd) iterates `stats.highlights.iter().take(cap)`.
Cap from `Option<Res<HighlightConfig>>` with fallback 3.

## Tracker Reset
`reset_highlight_tracker` runs OnEnter(GameState::Playing) — resets per-node fields, preserves cross-node fields.
