---
name: Layout name case sensitivity — boss_arena vs BossArena
description: Scenario RON files must use the exact name from the .node.ron file's name field (PascalCase). Wrong casing causes ScenarioLayoutOverride fallback with a WARN log that fails the scenario.
type: project
---

## Layout name must match `name:` field exactly (case-sensitive)

`NodeLayoutRegistry::get_by_name` does a case-sensitive lookup. Scenario RON files that specify `layout: "boss_arena"` will NOT match the registry entry whose `name: "BossArena"` (PascalCase).

The game emits:
```
WARN breaker::state::run::node::systems::set_active_layout: ScenarioLayoutOverride: layout 'boss_arena' not found, falling back to index selection
```

This captured WARN causes the scenario to fail (all captured logs fail the verdict).

**Affected new scenarios (as of 2026-04-06 commit 8d8254d6):**
- `scenarios/chaos/node_scale_entity_chaos.scenario.ron` — `layout: "boss_arena"` should be `"BossArena"`
- `scenarios/chaos/bolt_radius_clamping_chaos.scenario.ron` — `layout: "boss_arena"` should be `"BossArena"`

**Reference:** `boss_arena_chaos.scenario.ron` correctly uses `layout: "BossArena"`.
**Layout file:** `breaker-game/assets/nodes/boss_arena.node.ron` has `name: "BossArena"`.

**Fix:** Change `layout: "boss_arena"` to `layout: "BossArena"` in both new scenario files.

**Resolution (2026-04-06):** Both files corrected. All 116 scenarios pass as of this fix. The pattern is now documented — always cross-check new scenario RON layout names against the `name:` field in the corresponding `.node.ron` file.
