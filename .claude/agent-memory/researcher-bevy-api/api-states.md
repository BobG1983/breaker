---
name: Bevy 0.18.1 State System
description: States/SubStates/ComputedStates traits, StateTransitionEvent, in_state/state_changed conditions, condition_changed edge detectors, configure_sets per-schedule gotcha
type: reference
---

# State System (Bevy 0.18.1)

Verified against docs.rs/bevy/0.18.1, bevy v0.18.0 source, official examples.

## `States` trait (top-level, independent)

```rust
pub trait States: 'static + Send + Sync + Clone + PartialEq + Eq + Hash + Debug {
    const DEPENDENCY_DEPTH: usize = 1;
}
```

Multiple independent `States` can coexist in one app — "orthogonal dimensions."

```rust
app.init_state::<GameState>()
   .init_state::<PauseState>(); // completely independent
```

Both need `Default` (sets initial state), plus `Clone, Copy, PartialEq, Eq, Hash, Debug`.
Registered via `AppExtStates::init_state::<S>()` or `insert_state(S)`.

## `SubStates` trait (hierarchical, requires source)

```rust
pub trait SubStates: States {
    type SourceStates: StateSet;
    fn should_exist(sources: Self::SourceStates) -> Option<Self>;
}
```

Derive macro + `#[source(ParentState = ParentState::Variant)]` sets up the source.
Only exists when the source state condition is met; resource removed when condition fails.
SubStates CANNOT be independent (source-free) — that requires `States`.

Registration: `app.add_sub_state::<S>()` after the parent state is initialized.

## `ComputedStates` trait (derived, no manual transitions)

```rust
pub trait ComputedStates: 'static + Send + Sync + Clone + PartialEq + Eq + Hash + Debug {
    type SourceStates;
    fn compute(sources: Self::SourceStates) -> Option<Self>;
    const ALLOW_SAME_STATE_TRANSITIONS: bool = true;
}
```

Does NOT require `Default`. Automatically recomputed when any source changes.
`SourceStates` can be a single type, `Option<T>`, or tuple of multiple types.

Registration: `app.add_computed_state::<S>()`.

## State reading and transitioning

```rust
fn my_system(
    current: Res<State<GameState>>,          // read current state
    mut next: ResMut<NextState<GameState>>,  // queue transition
) {
    let s: &GameState = current.get();
    next.set(GameState::Playing);
}
```

## BREAKING CHANGE in Bevy 0.18

`next_state.set(S)` now ALWAYS fires `OnEnter`/`OnExit`, even when setting the same state value.
Use `next_state.set_if_neq(S)` for the old behavior (only transition if the state is different).

## SubStates nesting depth — verified 0.18.1

**SubStates can source from other SubStates** — no depth limit.

**Teardown order** (innermost exits first):
```
NodeState::OnExit → RunState::OnExit → GameState::OnExit → AppState::OnExit
```

**Enter order** (outermost enters first):
```
AppState::OnEnter → GameState::OnEnter → RunState::OnEnter → NodeState::OnEnter
```

**Valid 4-level hierarchy** (direct chaining, no ComputedState intermediary needed):
```rust
app.init_state::<AppState>()
   .add_sub_state::<GameState>()   // source: AppState = AppState::Game
   .add_sub_state::<RunState>()    // source: GameState = GameState::Run
   .add_sub_state::<NodeState>();  // source: RunState = RunState::Node
```

## `configure_sets` is per-schedule

**CRITICAL**: A `run_if` on a SystemSet in one schedule does NOT propagate to other schedules.

```rust
// MUST call configure_sets on EACH schedule independently:
app.configure_sets(Update, GameplaySystems.run_if(not_paused));
app.configure_sets(FixedUpdate, GameplaySystems.run_if(not_paused));
```

## `in_state` condition

```rust
pub fn in_state<S: States>(state: S) -> impl FnMut(Option<Res<'_, State<S>>>) + Clone;
```

Works in any schedule — `Update`, `FixedUpdate`, etc.
Returns `false` (not panic) if the state resource doesn't exist (e.g., SubStates not yet active).

## StateTransition schedule placement

`StateTransition` is inserted **after PreUpdate** in the main schedule:
- Frame order: `PreUpdate` → `StateTransition` → `FixedUpdate` → `Update` → `PostUpdate`
- A `NextState` queued during `FixedUpdate` or `Update` takes effect in the NEXT frame's `StateTransition`
- A `NextState` queued during `PreUpdate` takes effect in the SAME frame's `StateTransition`

## NextState has one slot — last set() wins

`NextState<S>` stores a single pending value. Multiple `set_if_neq()` calls in the same frame: only the last one takes effect.

---

## State-Related Run Conditions (Bevy 0.18.1)

Verified from `github.com/bevyengine/bevy/blob/v0.18.1/crates/bevy_state/src/condition.rs`.

```rust
pub fn state_exists<S: States>(current_state: Option<Res<State<S>>>) -> bool {
    current_state.is_some()
}

pub fn in_state<S: States>(state: S) -> impl FnMut(Option<Res<State<S>>>) -> bool + Clone {
    move |current_state| match current_state {
        Some(s) => *s == state,
        None => false,
    }
}

pub fn state_changed<S: States>(current_state: Option<Res<State<S>>>) -> bool {
    current_state.map_or(false, |s| s.is_changed())
}
```

**Critical gotcha**: `state_changed<S>` returns `false` when `State<S>` is REMOVED — the Option
is None so it returns false. It does NOT detect SubState removal (teardown). Use
`condition_changed_to(false, state_exists::<S>())` to detect removal.

---

## `condition_changed` / `condition_changed_to` (Bevy 0.18.1)

```rust
pub fn condition_changed<Marker, CIn, C>(condition: C) -> impl SystemCondition<(), CIn>
// Fires when wrapped condition output changes (either edge: false→true or true→false)
// Uses Local<bool> for previous-value tracking. Initial assumed previous = false.

pub fn condition_changed_to<Marker, CIn, C>(to: bool, condition: C) -> impl SystemCondition<(), CIn>
// Fires when wrapped condition transitions to `to`.
// Logic: *prev != new && new == to
// Initial assumed previous = false.
```

To detect "SubState S was removed" exactly once on the removal frame:

```rust
.run_if(condition_changed_to(false, state_exists::<S>()))
```

To detect "resource R was just created" exactly once:

```rust
.run_if(condition_changed_to(true, resource_exists::<R>()))
```

---

## StateTransitionEvent<S> as a Message (Bevy 0.18.1)

```rust
pub struct StateTransitionEvent<S: States> {
    pub exited: Option<S>,
    pub entered: Option<S>,
    pub allow_same_state_transitions: bool,
}
```

`StateTransitionEvent<S>` is a `Message` type (not a Bevy `Event`). It fires when any state
transition occurs, including SubState removal (where `entered: None`).

Fires AFTER the transition completes (after OnEnter/OnExit schedules run).

```rust
// Detect NodeState being removed (entered: None case):
.run_if(on_message::<StateTransitionEvent<NodeState>>())
// Then check entered.is_none() in system body to confirm it's a removal
```

This is truly event-driven (zero cost when no transition occurs) and does not require
child cooperation beyond the state machine itself firing the message.
