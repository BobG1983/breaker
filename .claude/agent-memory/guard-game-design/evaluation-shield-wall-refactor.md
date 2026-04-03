---
name: Shield Wall Refactor Evaluation
description: Shield changed from charge-based ShieldActive to timed visible floor wall (ShieldWall) with physics reflection -- approved with tuning recommendations
type: project
---

## Shield Refactor Evaluation (2026-04-02)

### Change Summary
- Old: `ShieldActive { charges }` component on breaker/cell, custom absorption logic, invisible
- New: `ShieldWall` entity -- visible blue HDR floor wall, timed (5s default), bolt reflection via `bolt_wall_collision`, re-fire resets timer
- Cell shielding via Shield effect: REMOVED (cell ShieldBehavior orbits still exist as cell type behavior)

### Verdict: APPROVED

All six pillars pass or improve. Architecture is cleaner (one collision path), synergy potential dramatically improved (wall is a real physics entity), juice profile dramatically improved (visible, glowing, obvious feedback).

### Tension Concern (High Priority Tuning)
5 seconds of full-width floor safety is generous. Violates Pillar 1 (Escalation) and Pillar 5 (Pressure Not Panic) softly. Recommendations:
1. Reduce default duration to 3.0s
2. Add visual decay over timer lifetime
3. Add timer cost per bolt reflected (~0.5-1.0s per save)
4. Phase 7: partial-width wall centered on breaker position at spawn time

### Catalog Drift
Parry chip in `docs/design/chip-catalog.md` still references `Shield(stacks: 1)` -- needs update to `Shield(duration: X.X)`.

### Cell Shielding Removal: CORRECT
Cell Shield was "extra HP with a different name" -- no skill expression, no synergy. Cell types have their own protection (ShieldBehavior orbits, locked cells, regen). Clean separation.

### Physics Reflection: CORRECT
One collision path for all walls. Shield benefits from any future wall-bump improvements for free.

**Why:** Tracks the Shield redesign decision and outstanding tuning work.
**How to apply:** Reference when tuning Shield duration, designing wall-bump effects, or evaluating future defensive mechanics. The tension concern is the highest-priority tuning item.
