# Adding a New Condition

Step-by-step reference for adding a condition to the effect system. Conditions are state-based (not event-based) and power `Tree::During(condition, ...)` entries.

The pattern is: **a variant in the `Condition` enum, a `is_my_condition(world: &World) -> bool` predicate function, and an arm in `evaluate_condition`.** That's it — the polling system is generic and picks up new conditions automatically.

## 1. Add the variant to the Condition enum

In `effect_v3/types/condition.rs`:

```rust
pub enum Condition {
    NodeActive,
    ShieldActive,
    ComboActive(u32),
    NewCondition,                 // or NewCondition(SomeParam)
}
```

Required derives are inherited (`Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`). If the variant carries an `f32` payload, wrap it in `OrderedFloat<f32>` so the enum keeps its `Hash`/`Eq` derives.

Parameterized conditions (like `ComboActive(u32)`) support per-threshold activation. A chip with `During(ComboActive(3), ...)` is independent from a chip with `During(ComboActive(5), ...)` — the poller treats them as different conditions because the variants compare unequal.

## 2. Write the predicate function

Create `effect_v3/conditions/new_condition.rs`:

```rust
//! `NewCondition` predicate evaluator.

use bevy::prelude::*;

/// Returns true while [some game state] holds.
pub fn is_new_condition(world: &World) -> bool {
    // Read whatever world state determines this condition.
    // Common patterns:
    //   - world.resource::<State<MyState>>().get() == &MyState::Foo
    //   - world.iter_entities().any(|e| world.get::<MarkerComponent>(e).is_some())
    //   - world.archetypes().iter().any(|a| a.contains(component_id) && !a.is_empty())
    //   - world.get_resource::<MyResource>().is_some_and(|r| r.condition_holds())
    todo!()
}
```

The function takes `&World` (not `&mut`) because it is called from inside `evaluate_conditions` during the read phase. If you need to query for a marker component without `&mut World`, use the archetype-scan pattern from `is_shield_active`:

```rust
pub fn is_shield_active(world: &World) -> bool {
    let Some(component_id) = world.component_id::<ShieldWall>() else {
        return false;
    };
    world
        .archetypes()
        .iter()
        .any(|archetype| archetype.contains(component_id) && !archetype.is_empty())
}
```

## 3. Add the predicate to evaluate_condition

In `effect_v3/conditions/evaluate_conditions/system.rs`:

```rust
pub fn evaluate_condition(condition: &Condition, world: &World) -> bool {
    match condition {
        Condition::NodeActive       => is_node_active(world),
        Condition::ShieldActive     => is_shield_active(world),
        Condition::ComboActive(n)   => is_combo_active(world, *n),
        Condition::NewCondition     => is_new_condition(world),
    }
}
```

The match is exhaustive — Rust will refuse to compile until you add the arm.

## 4. Re-export from conditions/mod.rs

In `effect_v3/conditions/mod.rs`:

```rust
mod new_condition;
// ... existing modules

pub use new_condition::is_new_condition;
```

`is_my_condition` is `pub` so other systems can poll it directly without going through `evaluate_condition`. The `UntilEvaluateCommand` Shape 4 path is one such caller.

## 5. (Optional) Add tests

Unit tests in `new_condition.rs` typically build a tiny world, set up the condition state, and assert the predicate returns the expected value:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_false_when_state_absent() {
        let world = World::new();
        assert!(!is_new_condition(&world));
    }

    #[test]
    fn returns_true_when_state_set() {
        let mut world = World::new();
        world.insert_resource(MyResource::active());
        assert!(is_new_condition(&world));
    }
}
```

## What you do NOT need to do

- **No need to write a per-condition monitor system.** The single `evaluate_conditions` polling system in `EffectV3Systems::Conditions` handles every condition uniformly.
- **No need to manage `DuringActive` transitions.** `evaluate_conditions` does that for every During entry it finds, regardless of which condition is involved.
- **No need to register systems.** The polling system is registered once by `EffectV3Plugin::build` and iterates `Tree::During` entries every tick — your new variant is automatically picked up.
- **No need to schedule `.after()` constraints for transitions.** The poller runs in `EffectV3Systems::Conditions`, which is `.after(EffectV3Systems::Tick)`, which is `.after(EffectV3Systems::Bridge)`. Conditions therefore see the post-bridge / post-tick state of the world automatically.

## How conditions are used by chip authors

A new condition becomes available in chip RON immediately after step 4. For example:

```ron
RootNode::Stamp(StampTarget::Bolt, Tree::During(
    Condition::NewCondition,
    ScopedTree::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig(multiplier: 1.5))),
))
```

The condition poller will activate the SpeedBoost when `is_new_condition(world)` becomes true and reverse it when the predicate returns false. Cycling is automatic — every transition fires/reverses. See `conditions.md` for the four "shapes" the poller handles.

## Reversibility of During effects

Direct `Fire` inside a `During` requires the effect to be `Reversible` — this is enforced at the type level by `ScopedTree::Fire(ReversibleEffectType)`. A non-reversible effect like `Shockwave` simply cannot appear in a direct `Fire` position.

Nested `When` inside a `During` re-opens the gate to the full `Tree`, allowing any `EffectType`. The reasoning is documented in `node_types.md` — the listener (the armed When) is what gets removed at scope end, not the effect, so reversibility is not required.

## Naming convention

- The enum variant is `PascalCase` and named after the *state* (`ShieldActive`, `ComboActive`, `NodeActive`).
- The predicate function is `snake_case` prefixed with `is_` (`is_shield_active`, `is_combo_active`, `is_node_active`).
- The file name matches the predicate without the `is_` prefix (`shield_active.rs`, `combo_active.rs`, `node_active.rs`).
