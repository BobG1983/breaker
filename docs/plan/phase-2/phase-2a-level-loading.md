# Phase 2a: Level Loading & Dev Tooling

**Goal**: Load levels from data and detect completion. Speed up dev iteration with CLI spawning.

---

## Level Loading

- **Hand-authored RON layouts**: 3+ grid layouts authored by hand in RON files
  - Explicit cell positions, types (standard/tough), and health values
  - Eventually a dedicated editor tool, but hand-authored for now
- **Layout selection**: Nodes load layouts in sequence from the run definition
- **Spawn cells from layout data**: Replace the current hardcoded grid spawning with data-driven spawning from RON

---

## Node Completion Detection

- **All target cells destroyed**: Track remaining target cells, fire a `NodeCleared` message when count hits zero
- **Non-target cells**: Architecture should support cells that don't count toward completion (future: decorative, indestructible)

---

## Dev Tooling

- **CLI test-level spawning** (dev/debug mode only): Command-line argument to skip menus and spawn directly into a specific test level layout
- Lives in the debug domain alongside existing debug overlays
- Speeds up iteration for all subsequent Phase 2 work

---

## Checklist

- [ ] RON level layout format defined
- [ ] 3+ hand-authored layouts
- [ ] Data-driven cell spawning from layouts
- [ ] Node completion detection (all target cells cleared)
- [ ] `NodeCleared` message
- [ ] CLI test-level spawning (debug/dev mode)
