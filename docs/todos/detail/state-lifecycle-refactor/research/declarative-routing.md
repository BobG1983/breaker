# Idiom Research: Declarative State Routing in Bevy 0.18

## Context

The `rantzsoft_lifecycle` crate needs a routing system for state machines. Every screen
state enum has variants, and when a variant "completes", the crate must know where to go
next. Some routes are fixed at compile time (AnimateIn → Active always), some need runtime
decisions (Teardown → one of three targets depending on resources).

The additional wrinkle: some routes change a **different** state type (NodeState::Teardown
→ RunState::ChipSelect changes a parent `GameState`, not the `NodeState` itself).

This is a pure Rust idiom question. Bevy provides the mechanisms; the question is which
combination to reach for.

---

## The Seven Approaches

### Approach 1 — `Box<dyn Fn(&mut World)>` Stored in a Resource

Register closures during plugin build that perform the full routing action (read resources,
call `world.resource_mut::<NextState<S>>().set(...)`):

```rust
// In rantzsoft_lifecycle
pub struct RouteTable<S: States> {
    routes: HashMap<S, Box<dyn Fn(&mut World) + Send + Sync>>,
}

// Registration at plugin build time
impl<S: States + Eq + Hash> RouteTable<S> {
    pub fn add_route(
        &mut self,
        from: S,
        handler: impl Fn(&mut World) + Send + Sync + 'static,
    ) {
        self.routes.insert(from, Box::new(handler));
    }
}

// In breaker-game plugin:
let mut table = RouteTable::<NodeState>::default();
table.add_route(NodeState::AnimateIn, |world| {
    world.resource_mut::<NextState<NodeState>>().set(NodeState::Active);
});
table.add_route(NodeState::Teardown, |world| {
    let result = *world.resource::<NodeResult>();
    let is_final = world.resource::<RunState>().is_final_node();
    let next = match (result, is_final) {
        (NodeResult::Win, false) => GameState::ChipSelect,
        (NodeResult::Win, true) | (NodeResult::Lose, _) => GameState::RunEnd,
    };
    world.resource_mut::<NextState<GameState>>().set(next);
});
app.insert_resource(table);
```

The dispatch system runs as an exclusive system triggered `OnExit` each phase:

```rust
fn dispatch_route<S: States + Eq + Hash + Copy>(
    world: &mut World,
) {
    let current = *world.resource::<State<S>>().get();
    // Clone the handler out to avoid borrow conflict
    let handler = {
        let table = world.resource::<RouteTable<S>>();
        table.routes.get(&current).map(|f| {
            // We can't call f while borrowing table
            // This requires an Arc or a different ownership model
        })
    };
}
```

**The borrow problem**: `world.resource::<RouteTable<S>>()` borrows `world`, so you cannot
then call a closure that takes `&mut World`. You must either:
- Store handlers as `Arc<dyn Fn(&mut World)>` and clone the `Arc` before calling
- Use `world.resource_scope` to extract the table temporarily
- Store `SystemId` instead of closures (see Approach 3)

With `resource_scope`:
```rust
fn dispatch_route<S: States + Eq + Hash + Copy>(world: &mut World) {
    let current = *world.resource::<State<S>>().get();
    world.resource_scope(|world, table: Mut<RouteTable<S>>| {
        if let Some(handler) = table.routes.get(&current) {
            handler(world);
        }
    });
}
```

**Ergonomics**: Moderate. `Box<dyn Fn(&mut World)>` closures are verbose at call sites.
The closure captures nothing from the call site — all context comes from `world.resource::<T>()`.

**Type safety**: Low. `NextState<GameState>` and `NextState<NodeState>` are both available
to any closure. Nothing prevents setting the wrong state. No cross-state type safety.

**World access**: Full. The closure receives `&mut World` and can read any resource.

**Cross-level routing**: Natural. The closure sets `NextState<GameState>` or any other
`NextState<S>` — level is irrelevant.

**Testability**: Moderate. Must build a headless `App` with the resources under test.
Cannot call the closure with a mock world.

---

### Approach 2 — `(Condition, Target)` Pairs (run_if model)

Model each route as an ordered list of `(Box<dyn Fn(&World) -> bool>, RouteTarget)` pairs.
The first condition that returns `true` determines the target.

```rust
pub enum RouteTarget<S: States> {
    SameLevel(S),
    // Cross-level routing needs type erasure or a separate mechanism
}

pub struct ConditionalRoute<S: States> {
    conditions: Vec<(Box<dyn Fn(&World) -> bool + Send + Sync>, RouteTarget<S>)>,
    default: RouteTarget<S>,
}
```

This mirrors Bevy's `run_if` condition pattern. Conditions are read-only (`&World`), which
solves the borrow conflict (no mutation while reading).

```rust
// Registration
table.add_conditional_route(
    NodeState::Teardown,
    vec![
        (
            Box::new(|w: &World| {
                *w.resource::<NodeResult>() == NodeResult::Win
                && !w.resource::<RunState>().is_final_node()
            }),
            RouteTarget::ParentState(GameState::ChipSelect),
        ),
    ],
    RouteTarget::ParentState(GameState::RunEnd), // default
);
```

The dispatch system reads conditions (`&World`) then applies the target (`&mut World`):
```rust
fn dispatch_conditional<S: States + Eq + Hash>(world: &mut World) {
    let current = *world.resource::<State<S>>().get();
    // Read phase — borrow &World immutably
    let target = world.resource_scope(|world, table: Mut<ConditionalRouteTable<S>>| {
        table.routes.get(&current).and_then(|route| {
            for (cond, target) in &route.conditions {
                if cond(world) {
                    return Some(target.clone());
                }
            }
            Some(route.default.clone())
        })
    });
    // Apply phase — now safe to use &mut World
    if let Some(target) = target {
        apply_route_target(world, target);
    }
}
```

**Ergonomics**: Worse than Approach 1. Two layers of `Box<dyn Fn>`. For simple static
routes (`AnimateIn → Active`) this is overkill — you'd write a condition that always
returns `true`, which is noise.

**Type safety**: Same as Approach 1 for the apply step. Conditions themselves are
read-only, which is safer.

**World access**: Read-only in conditions (correct), full in apply.

**Cross-level routing**: `RouteTarget` must be made type-aware or use type erasure. The
enum variant needs to hold `Box<dyn FnOnce(&mut World)>` anyway, which collapses this
back toward Approach 1 for the apply step.

**Testability**: Slightly better — conditions can be tested in isolation if they don't
require a full App. Still need resources in the world.

**Verdict**: More indirection than Approach 1 for no clear benefit. The separation of
read-only conditions from mutable apply is principled but adds boilerplate. Collapse into
Approach 1.

---

### Approach 3 — One-Shot Systems via `SystemId`

Bevy 0.18 fully supports one-shot systems. `world.register_system(f)` returns
`SystemId<(), O>`. The system can use any `SystemParam` (Res, ResMut, Query, Commands,
NextState).

```rust
#[derive(Resource)]
pub struct RouteTable<S: States + Eq + Hash> {
    routes: HashMap<S, SystemId>,
}

// Registration in plugin
fn build_route_table(world: &mut World) -> RouteTable<NodeState> {
    let mut table = RouteTable::default();
    table.routes.insert(
        NodeState::AnimateIn,
        world.register_system(route_animate_in_to_active),
    );
    table.routes.insert(
        NodeState::Teardown,
        world.register_system(route_teardown),
    );
    table
}

// Each route is a normal system with full SystemParam access
fn route_animate_in_to_active(mut next: ResMut<NextState<NodeState>>) {
    next.set(NodeState::Active);
}

fn route_teardown(
    result: Res<NodeResult>,
    run_state: Res<RunState>,
    mut next_game: ResMut<NextState<GameState>>,
) {
    let is_final = run_state.is_final_node();
    match (*result, is_final) {
        (NodeResult::Win, false) => next_game.set(GameState::ChipSelect),
        _ => next_game.set(GameState::RunEnd),
    }
}
```

Dispatch from an exclusive system:
```rust
fn dispatch_route<S: States + Eq + Hash + Copy>(world: &mut World) {
    let current = *world.resource::<State<S>>().get();
    let id = world.resource::<RouteTable<S>>()
        .routes
        .get(&current)
        .copied();
    if let Some(id) = id {
        world.run_system(id).expect("routing system failed");
    }
}
```

No borrow conflict: `SystemId` is `Copy`, so we copy it out of the table borrow before
calling `world.run_system`.

**Ergonomics**: Excellent. Route handlers are ordinary functions — the same form as every
other system in the codebase. No closure syntax, no Box. Registration is explicit and
readable.

**Type safety**: High for each route's internal logic (normal system type checking). The
`HashMap<S, SystemId>` does not enforce that the right `SystemId` type is stored, but
`SystemId` is opaque so you can't accidentally run a system with the wrong signature.

**World access**: Full `SystemParam` access via normal system parameters.

**Cross-level routing**: Natural. A route system for `NodeState` can inject
`ResMut<NextState<GameState>>` — state level is irrelevant.

**Testability**: Excellent. Each route is a normal function testable with a minimal `App`.
This is the same pattern as `handle_node_cleared` in the existing codebase
(`breaker-game/src/run/systems/handle_node_cleared.rs`).

**Limitation**: `world.run_system` runs the system immediately and serially. It cannot
run in parallel with other systems. This is fine for routing — routing is a rare, discrete
event, not a hot-path operation.

**Registration timing**: `world.register_system` requires `&mut World`. In Bevy 0.18,
you can call this from `Plugin::build` via `app.world_mut().register_system(...)`, or
from a `FromWorld` resource impl.

---

### Approach 4 — Bevy 0.18 Observers

Observers fire in response to `Trigger<E>` events and receive `&mut World` access through
their parameters. They can be registered globally or on specific entities.

```rust
// Register an observer during plugin build
app.add_observer(|trigger: Trigger<PhaseCompleted<NodeState>>,
                  result: Res<NodeResult>,
                  run_state: Res<RunState>,
                  mut next: ResMut<NextState<GameState>>| {
    if trigger.event().phase == NodeState::Teardown {
        // routing logic
    }
});
```

However, observers have a key property: **all observers for a trigger fire**, not just the
first matching one. For a routing table where exactly one target should be chosen per
state, this is problematic — you'd need guards in every observer to avoid multiple
`NextState::set` calls.

**Ergonomics**: Good for event-driven patterns. Awkward for exclusive routing (exactly one
target).

**Type safety**: Same as systems — normal Rust typing.

**World access**: Full via SystemParam in the observer closure.

**Cross-level routing**: Natural.

**Testability**: Good — observers can be triggered in test apps.

**Verdict**: Observers are the right tool for "broadcast when X happens" (e.g., notify
audio system when a phase changes). They are the wrong tool for "decide exactly one next
state" (routing). The exclusive-dispatch semantics of Approach 3 fit routing better.

---

### Approach 5 — Trait-Based Routing (`Routable`)

The state enum implements a trait that returns its own next state:

```rust
pub trait Routable: States {
    fn route(&self, world: &World) -> Option<RouteAction>;
}

pub enum RouteAction {
    ChangeState(Box<dyn AnyState>),  // type-erased state change
}
```

Implementation in the game crate:
```rust
impl Routable for NodeState {
    fn route(&self, world: &World) -> Option<RouteAction> {
        match self {
            NodeState::AnimateIn => Some(RouteAction::same_level(NodeState::Active)),
            NodeState::Teardown => {
                let result = *world.resource::<NodeResult>();
                let is_final = world.resource::<RunState>().is_final_node();
                let next: GameState = match (result, is_final) {
                    (NodeResult::Win, false) => GameState::ChipSelect,
                    _ => GameState::RunEnd,
                };
                Some(RouteAction::other_level(next))
            }
            _ => None,
        }
    }
}
```

**Ergonomics**: Moderate. The `match self` in `route()` is the cleanest expression of
routing logic — it reads like a state machine diagram. No external registration needed.
The downside is `RouteAction` needs type-erased state application for cross-level routing,
which requires `Box<dyn Any>` + `TypeId` bookkeeping or a macro.

**Type safety**: High within a single level. Cross-level routing (`RouteAction::other_level`)
loses type safety at the `GameState` boundary — the crate cannot know all states.

**World access**: `&World` (read-only). Sufficient for resource queries.

**Cross-level routing**: Hard. The crate cannot name `GameState` or any game-specific
type. The route action must be type-erased or use a callback.

**Testability**: Excellent. `route()` is a pure function of `&World` — no system
apparatus needed. Call directly.

**Verdict**: Best ergonomics for same-level routing. Cross-level routing breaks the
abstraction (the crate would need to know about `GameState`). This could be solved by
making `RouteAction` hold a `Box<dyn FnOnce(&mut World)>`, which brings it back to
Approach 1. As a hybrid: implement `Routable` for the trait that drives `Box<dyn FnOnce>`
routing internally.

---

### Approach 6 — `ComputedStates` for Route Derivation

`ComputedStates` derives a new state from existing states. Could model a routing decision:

```rust
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum PostTeardownRoute {
    ChipSelect,
    RunEnd,
}

impl ComputedStates for PostTeardownRoute {
    type SourceStates = (GameState, NodeState);
    fn compute((game, node): (Option<GameState>, Option<NodeState>)) -> Option<Self> {
        // Cannot access arbitrary resources here — only the source states
        None
    }
}
```

**Critical limitation**: `ComputedStates::compute` receives only the source state values,
NOT arbitrary resources. There is no `&World` access in `compute`. This means dynamic
routes that depend on `NodeResult` or `RunState` resources cannot be expressed this way.

`ComputedStates` is useful for "derive state X from the combination of states A and B" —
not for "query the world to decide where to go".

**Verdict**: Wrong tool for dynamic routing. Eliminates this approach entirely for the
stated use case.

---

### Approach 7 — Flat Dispatch System (No Routing Table)

The simplest approach: no crate-level routing abstraction. Each domain writes its own
`OnExit(SomeState)` system that sets `NextState`:

```rust
// In breaker-game's NodePlugin
app.add_systems(OnExit(NodeState::AnimateIn), |mut next: ResMut<NextState<NodeState>>| {
    next.set(NodeState::Active);
});
app.add_systems(OnExit(NodeState::Teardown), route_after_teardown);

fn route_after_teardown(
    result: Res<NodeResult>,
    run_state: Res<RunState>,
    mut next_game: ResMut<NextState<GameState>>,
) { ... }
```

This is exactly how `advance_node`, `handle_node_cleared`, and `handle_run_lost` work
today. The "declarative routing table registered at setup time" is just `app.add_systems`
calls, which IS declarative and IS setup-time.

**Ergonomics**: Excellent. No new abstractions. Same pattern as the entire codebase.

**Type safety**: Full — Bevy's type system handles everything.

**World access**: Full SystemParam.

**Cross-level routing**: Natural — any system can inject `ResMut<NextState<GameState>>`.

**Testability**: Excellent — same pattern as all existing systems.

**Verdict**: This is the existing approach and it works. The question is whether
`rantzsoft_lifecycle` needs to OWN the routing, or just provide the signals (phase
completion messages) that game-side systems respond to.

---

## Recommendation

### For static routes within a single state level

Use **Approach 7 (flat `OnExit` systems)** registered during plugin build. A static
route is a one-liner system:

```rust
app.add_systems(OnExit(NodeState::AnimateIn), |mut next: ResMut<NextState<NodeState>>| {
    next.set(NodeState::Active);
});
```

This is 3 lines total (with the closure). There is zero advantage to building a routing
table to express the same thing. The codebase already does this — `advance_node` in
`run/systems/advance_node.rs` is exactly this pattern.

### For dynamic routes (including cross-level)

Use **Approach 3 (one-shot systems)** when the routing logic is complex enough to need a
named function. Register the route handler as a named system function (same shape as
`handle_node_cleared.rs`), and the dispatch system runs it via `world.run_system`.

The dispatch system for `rantzsoft_lifecycle` would be:

```rust
fn dispatch_lifecycle_route<S: ScreenLifecycle>(world: &mut World) {
    let current = *world.resource::<State<S>>().get();
    let id = world.resource::<LifecycleRouteTable<S>>()
        .routes
        .get(&current)
        .copied();
    if let Some(id) = id {
        let _ = world.run_system(id);
    }
}
```

Registration in the game plugin:

```rust
let id = app.world_mut().register_system(route_after_node_teardown);
app.world_mut()
    .resource_mut::<LifecycleRouteTable<NodeState>>()
    .routes
    .insert(NodeState::Teardown, id);
```

Where `route_after_node_teardown` is a normal system function with full `SystemParam`
access — the same form as every routing system in the codebase today.

### The split: what `rantzsoft_lifecycle` owns vs. what the game owns

`rantzsoft_lifecycle` owns:
- The trigger mechanism: `PhaseComplete<S>` message
- The dispatch runner: `dispatch_lifecycle_route<S>` exclusive system (runs on `PhaseComplete<S>`)
- The table type: `LifecycleRouteTable<S>` resource (game populates it)

The game owns:
- Registering route handlers into the table
- The route handler functions themselves (they reference game types like `NodeResult`)

This keeps `rantzsoft_lifecycle` game-vocabulary-free (required by
`.claude/rules/rantzsoft-crates.md`).

---

## Cross-Level Routing: How to Do It

Cross-level routing (NodeState::Teardown → GameState::ChipSelect) is handled by the route
handler function, which injects `ResMut<NextState<GameState>>` alongside any game
resources it needs. The handler function lives in the game crate, so it can freely
reference `GameState`. The `rantzsoft_lifecycle` crate sees only `SystemId` — opaque.

```rust
// In breaker-game — NOT in rantzsoft_lifecycle
fn route_after_node_teardown(
    result: Res<NodeResult>,
    run_state: Res<RunState>,
    mut next_game: ResMut<NextState<GameState>>,
    mut next_node: ResMut<NextState<NodeState>>,
) {
    let is_final = run_state.is_final_node();
    match (*result, is_final) {
        (NodeResult::Win, false) => next_game.set(GameState::ChipSelect),
        _ => next_game.set(GameState::RunEnd),
    }
    // NodeState is now exiting anyway — no need to set it explicitly
}
```

The route function sets the parent state directly. This is already how the codebase
routes: `handle_node_cleared.rs:38-41` sets `NextState<GameState>` from within a system
that runs under `PlayingState::Active`.

---

## Performance Notes

`world.run_system` runs serially. Routing happens at most once per phase transition
(rare events). This is never a hot path. No performance concern.

`SystemId` is `Copy` (it's a `u32` entity ID). Extracting it from a `HashMap` lookup and
copying before calling `world.run_system` is zero overhead.

---

## Alternatives Considered

| Approach | Why Not |
|---|---|
| `Box<dyn Fn(&mut World)>` stored directly | Borrow conflict when dispatching — must use `resource_scope` or `Arc`. Same power as one-shot systems but with inferior ergonomics (closures instead of named fns). |
| `(Condition, Target)` pairs | More indirection than needed. Static routes require a trivially-true condition. Cross-level targets still need type erasure. |
| `Routable` trait on the state enum | Clean for same-level routes, but breaks for cross-level because the crate can't name game state types. Requires `Box<dyn FnOnce(&mut World)>` for the apply step anyway. |
| `ComputedStates` | `compute()` has no `&World` access — cannot read resources. Eliminates dynamic routing. |
| Bevy Observers | "All matching observers fire" semantics are wrong for routing. Good for broadcast; bad for exclusive next-state selection. |
| Flat `OnExit` systems only | Correct, but puts routing logic directly in the game plugin rather than through the lifecycle crate. Works fine for simple cases; use this for static routes. |

---

## Codebase Precedent

- `breaker-game/src/run/systems/handle_node_cleared.rs:15-44` — Dynamic routing with
  resource reads and `NextState<GameState>` mutation. Exact same pattern as the
  recommended one-shot handler function.
- `breaker-game/src/run/systems/handle_run_lost.rs:15-26` — Cross-level route: reads
  `RunState` resource, sets `NextState<GameState>`. Demonstrates cross-level is trivial.
- `breaker-game/src/run/systems/advance_node.rs:12-15` — Static route: runs
  `OnEnter(GameState::TransitionIn)`, mutates `RunState`. Shows `OnEnter`/`OnExit` systems
  as declarative route declaration.
- `breaker-game/src/run/plugin.rs:84` — `app.add_systems(OnEnter(GameState::TransitionIn), advance_node)` shows setup-time route registration is just `add_systems`.

---

## Summary Table

| Property | Box Closure | Condition Pairs | One-Shot SystemId | Observers | Routable Trait | ComputedStates | Flat OnExit |
|---|---|---|---|---|---|---|---|
| Ergonomics | Fair | Poor | **Excellent** | Good | Good | Poor | **Excellent** |
| Type safety | Low | Low | Medium | Medium | High (same-level) | N/A | **High** |
| World access | Full | Read/Full | **Full** | Full | Read only | None | **Full** |
| Cross-level | Natural | Complex | **Natural** | Natural | Hard | No | **Natural** |
| Testability | Moderate | Moderate | **Excellent** | Good | **Excellent** | N/A | **Excellent** |
| Complexity | Medium | High | Low | Low | Medium | Low | **None** |

**Winner**: Flat `OnExit` systems for static routes. One-shot `SystemId` table for the
lifecycle crate's dispatch mechanism when the crate must own the routing trigger.
