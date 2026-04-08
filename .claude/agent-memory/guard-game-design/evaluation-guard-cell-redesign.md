---
name: Guard Cell Redesign Evaluation
description: 3x3 grid-based guard cells replacing orbit shields — approved against all 6 pillars; key synergies and sharpening notes
type: project
---

## Guard Cell Redesign — Approved

### Design
- Guarded Cell (Gu) = damageable parent at center of 3x3 grid, NOT locked
- Guardian Cells (gu) = square children (cell_height x cell_height), slide between ring positions
- Gaps between guardians = (cell_width - cell_height) / 2 on each side — threading target for skilled players
- Destroying a guardian opens a permanent gap
- Auto-despawn all guardians when parent dies

### Pillar Results
All 6 pillars: PASS

- **Speed**: No waiting (orbit model was wait-for-rotation dead air). Gaps always exist for skilled aim.
- **Skill ceiling**: Novice chews through guardians; expert threads gaps to hit parent directly. Huge floor/ceiling gap.
- **Tension**: Parent is fully damageable (not locked). Race against timer/regen. Guardians are pressure, not gates.
- **Meaningful decisions**: Aim for guardians (safer, slower) vs thread gaps (risky, fast). Context-dependent on build.
- **Synergy potential**: Piercing, AoE/Shockwave, Chain Lightning, Multi-bolt, Attraction, Phantom combos all interesting.
- **Juice**: Guardian shatter + permanent gap visual, bolt-threading whistle, formation collapse on parent death.

### Sharpening Notes
1. **Slide speed should escalate with tier** — learnable geometry in T1, prediction-required speed in T7
2. **Consider formation reassemble** — remaining guardians redistribute after a kill, creating a brief exploitation window before gaps equalize

### Why (replacing orbit model)
Orbit = "wait for rotation" = dead air = fails the speed test. Guard = "aim for gaps" = active skill expression. Strictly better for this game's identity.

**How to apply:** Reference when implementing guardian cells, tuning slide speed per tier, and designing node layouts with Gu/gu positions.
