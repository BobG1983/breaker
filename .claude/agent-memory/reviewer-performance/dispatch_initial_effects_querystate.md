---
name: DispatchInitialEffects QueryState allocation pattern
description: world.query_filtered() calls inside Command::apply() — acceptable at chip-equip frequency
type: project
---

`DispatchInitialEffects::apply()` calls `world.query_filtered::<Entity, ...>()` to build QueryState objects for primary breakers and primary bolts. `resolve_all()` and `resolve_default()` also build QueryState per call.

These QueryState allocations are done inside `Command::apply()`, which runs in the command flush phase — not in a per-frame system. The command is dispatched at chip equip time (node start or upgrade screen selection), which happens at most a handful of times per run.

**Why it's acceptable:** QueryState construction cost is amortized across the frequency of the call. At chip-equip frequency (non-hot path), this is not a concern.

**What would make it a concern:** If DispatchInitialEffects were enqueued every FixedUpdate frame (it isn't), QueryState churn would matter. Watch for that if usage patterns change.
