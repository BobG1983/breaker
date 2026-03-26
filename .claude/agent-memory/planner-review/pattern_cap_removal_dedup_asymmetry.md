---
name: Cap removal exposes dedup asymmetry across detection systems
description: Run domain highlight detection systems have two dedup patterns — per-event systems use per-kind checks, track_node_cleared_stats uses only the cap. Removing the cap without addressing this creates unbounded growth.
type: feedback
---

The run domain's highlight detection has asymmetric dedup behavior:
- **Per-event systems** (detect_mass_destruction, detect_close_save, detect_nail_biter, detect_combo_and_pinball, detect_first_evolution): Check `let already = stats.highlights.iter().any(|h| h.kind == ...)` before pushing. One entry per kind per run.
- **track_node_cleared_stats**: NO per-kind dedup. Pushes ClutchClear, NoDamageNode, FastClear, PerfectStreak, SpeedDemon, Untouchable, Comeback, PerfectNode on every qualifying NodeCleared. Only the `highlight_cap` prevents unbounded growth.

**Why:** When a spec proposes removing the cap from detection time (deferring to display-time selection), `track_node_cleared_stats` would accumulate N_nodes * up_to_8 entries. This isn't necessarily wrong (selection picks the best), but the spec MUST explicitly address whether duplicates are intended or whether per-kind dedup should be added.

**How to apply:** Any spec that modifies highlight recording limits must audit ALL detection systems for their dedup behavior, not assume they're uniform.
