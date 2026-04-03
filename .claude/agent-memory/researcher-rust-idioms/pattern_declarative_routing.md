---
name: Declarative state routing — one-shot SystemId table + flat OnExit
description: How to route between Bevy states declaratively at plugin setup time, covering static routes, dynamic routes, and cross-level (parent state) routing
type: project
---

## Decision

Two-tier pattern:

1. **Static routes** (always the same next state): flat `app.add_systems(OnExit(S::AnimateIn), ...)` closures — no routing table needed. This IS declarative, setup-time registration. Same as `advance_node` in `run/plugin.rs:84`.

2. **Dynamic routes** (need runtime resource reads): one-shot systems registered via `world.register_system(fn)` → stored as `SystemId` in a `HashMap<S, SystemId>` resource. Dispatched from an exclusive system via `world.run_system(id)`. `SystemId` is `Copy` so it can be extracted before calling `run_system`, avoiding the borrow conflict.

## Cross-Level Routing

Cross-level routes (e.g., NodeState::Teardown → GameState::ChipSelect) work naturally: the route handler function (a one-shot system in the game crate) injects `ResMut<NextState<GameState>>` directly. The `rantzsoft_lifecycle` crate sees only `SystemId` and never names game types.

## Split of Ownership

- `rantzsoft_lifecycle` owns: the dispatch runner (exclusive system), the `LifecycleRouteTable<S>` resource type, and the trigger mechanism.
- Game crate owns: registering routes into the table, and the route handler functions (which may reference game vocabulary).

## Why Not Alternatives

- `Box<dyn Fn(&mut World)>` stored directly: borrow conflict at dispatch time — must use `resource_scope` or `Arc`. One-shot systems are identical power with better ergonomics (named fns, not closures).
- `(Condition, Target)` pairs: more indirection, static routes need trivially-true conditions.
- `Routable` trait on enum: breaks for cross-level routes because the crate can't name game state types.
- `ComputedStates`: no `&World` access in `compute()` — cannot read resources for dynamic routing.
- Observers: "all matching observers fire" semantics are wrong for exclusive next-state selection.

## Full Research

See `docs/todos/detail/spawn-bolt-setup-run-migration/research/declarative-routing.md`
