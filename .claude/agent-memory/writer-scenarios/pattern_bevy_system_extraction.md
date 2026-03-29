---
name: Bevy system extraction anti-pattern
description: Do NOT extract FixedUpdate system tuples into helper functions — the return type is too complex to express
type: feedback
---

Do NOT extract a tuple of `add_systems(FixedUpdate, (...))` into a separate function to reduce line count.

**Why:** The return type for such a function is `impl IntoScheduleConfigs<ScheduleSystem, M>` where `M` is an inferred marker — but Bevy 0.18's `IntoScheduleConfigs` takes 2 generic parameters, and expressing this in a helper function return position causes either:
- `error[E0107]: trait takes 2 generic arguments but 1 generic argument was supplied`
- `error[E0277]: impl IntoScheduleConfigs<...> does not describe a valid system configuration`

The `M` marker type is an anonymous inferred type that cannot be named in source code.

**How to apply:** When `register_scenario_systems` (or any plugin `build` function) is too long due to the FixedUpdate system block, reduce the line count by:
1. Pulling out local `let checkers_a = (...).chain()` / `let checkers_b = (...).chain()` variables before the `add_systems` calls — these are just tuples, not system configs, so they have no marker type problem.
2. Tightening formatting (fewer blank lines, combining imports).
3. Extracting non-system setup code into separate `fn register_resources(app)` helpers instead.

The fix applied in this session was: instead of extracting `invariant_checker_systems() -> impl ...`, create local `let checkers_a` and `let checkers_b` tuple variables above the `.add_systems` call.
