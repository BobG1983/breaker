# Phase 2a: Level Loading

**Goal**: Load levels from data and detect completion.

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

## Checklist

- [ ] RON level layout format defined
- [ ] 3+ hand-authored layouts
- [ ] Data-driven cell spawning from layouts
- [ ] Node completion detection (all target cells cleared)
- [ ] `NodeCleared` message
