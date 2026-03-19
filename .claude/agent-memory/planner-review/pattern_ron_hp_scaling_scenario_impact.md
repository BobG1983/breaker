---
name: ron_hp_scaling_scenario_impact
description: Scaling RON cell HP values dramatically changes scenario dynamics — frame limits may need updating
type: feedback
---

When cell HP values in RON files are scaled up (e.g., 1→10, 3→30), all existing scenarios that exercise node clearing will take proportionally longer. Scenarios with tight `max_frames` limits (like `chrono_clear_race`) may fail.

**Why:** Scenarios run the real game with real RON files. Higher HP = more hits per cell = more frames to clear nodes.

**How to apply:** Any spec that changes RON cell HP values must explicitly address whether scenario frame limits need updating. Cross-check `chrono_clear_race.scenario.ron` and any other scenarios that depend on node clearing within a frame budget.
