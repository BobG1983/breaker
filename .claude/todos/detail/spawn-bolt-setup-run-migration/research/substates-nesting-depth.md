# Research: SubStates Nesting Depth — Bevy 0.18.1

**Verified against**: Bevy v0.18.0 source (applies to 0.18.1 — no state machinery changes between patch versions)

Sources consulted:
- `github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_state/src/state/sub_states.rs`
- `github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_state/src/state/state_set.rs`
- `github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_state/src/state/transitions.rs`
- `github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_state/src/app.rs`
- `docs.rs/bevy/0.18.1/bevy/state/state/trait.SubStates.html`
- Bevy 0.17→0.18 migration guide

---

## Short Answer

**Yes, the hierarchy is fully supported.** SubStates can source from other SubStates — 4-level-deep nesting works correctly in Bevy 0.18.1. There is no depth limit. `OnEnter`/`OnExit` fire at every level. Teardown cascades correctly.

---

## Question-by-Question Findings

### 1. Can a SubStates derive from another SubStates (not just a root States)?

**Yes.** Here is the chain of type constraints that makes this work:

```
SubStates: States + FreelyMutableState
  type SourceStates: StateSet

StateSet is implemented for S where S: InnerStateSet
InnerStateSet is implemented for S where S: States
SubStates: States  ← this is the key
```

Because `SubStates` requires `States` as a supertrait, any `SubStates` type is also a `States` type. Since `InnerStateSet` is implemented for all `S: States`, any SubStates type is a valid `InnerStateSet`. And since `StateSet` is implemented for single `S: InnerStateSet` types, any SubStates type is a valid `SourceStates`.

The exact `InnerStateSet` impl (verbatim from source):

```rust
impl<S: States> InnerStateSet for S {
    type RawState = Self;
    const DEPENDENCY_DEPTH: usize = S::DEPENDENCY_DEPTH;
    fn convert_to_usable_state(wrapped: Option<&State<Self::RawState>>) -> Option<Self> {
        wrapped.map(|v| v.0.clone())
    }
}
```

The bound is `S: States` — not `S: States + !SubStates`. SubStates types qualify.

The derive macro generates valid `DEPENDENCY_DEPTH` automatically:

```rust
// What the derive macro generates for GameState:
impl States for GameState {
    const DEPENDENCY_DEPTH: usize = <GameState as SubStates>::SourceStates::SET_DEPENDENCY_DEPTH + 1;
}
```

For your hierarchy, DEPENDENCY_DEPTH values are:
- `AppState`: 1 (root, default)
- `GameState`: `AppState::DEPENDENCY_DEPTH + 1` = 2
- `RunState`: `GameState::DEPENDENCY_DEPTH + 1` = 3
- `NodeState`: `RunState::DEPENDENCY_DEPTH + 1` = 4

### 2. Is there any depth limit on SubState nesting?

**No hard limit exists.** `DEPENDENCY_DEPTH` is a `usize` constant used only for system ordering. There is no assertion, panic, or compile-time check that enforces a maximum. The ordering constraint is expressed as:

```rust
.after(ApplyStateTransition::<ParentState>::default())
// and exit schedules:
.before(ExitSchedules::<ParentState>::default())
// and enter schedules:
.after(EnterSchedules::<ParentState>::default())
```

Each level simply adds another `.after()` / `.before()` constraint in the `StateTransition` schedule. 4 levels deep produces a chain of 3 ordering constraints — trivially handled by Bevy's scheduler.

The `SET_DEPENDENCY_DEPTH` for a single-state `StateSet` is just `S::DEPENDENCY_DEPTH`. For your deepest state:

```
NodeState::SET_DEPENDENCY_DEPTH = NodeState::DEPENDENCY_DEPTH = 4
```

### 3. Do `OnEnter`/`OnExit` work at all nesting levels?

**Yes.** Every SubStates level registers its own `OnEnter` and `OnExit` schedules independently. The `run_enter` and `run_exit` functions are wired up per type:

```rust
// When should_exist returns Some (state created or persisted):
// run_enter fires if State<S> did not previously exist
let _ = world.try_run_schedule(OnEnter(entered));

// When should_exist returns None (state removed):
// run_exit fires if State<S> previously existed
let Some(exited) = transition.exited else { return; };
let _ = world.try_run_schedule(OnExit(exited));
```

`try_run_schedule` is used (not `run_schedule`), so a missing `OnEnter`/`OnExit` schedule for a state variant does not panic — it silently succeeds.

### 4. When AppState transitions away from Game, do all nested SubStates get properly torn down?

**Yes, and in the correct order.** The teardown cascade works as follows:

**Ordering constraints** (registered per SubState level):
- Child's `ApplyStateTransition` runs `.after(ApplyStateTransition::<Parent>)`
- Child's `ExitSchedules` runs `.before(ExitSchedules::<Parent>)`
- Child's `EnterSchedules` runs `.after(EnterSchedules::<Parent>)`

This means exit order for your hierarchy is:
```
NodeState::OnExit  (innermost first)
RunState::OnExit
GameState::OnExit
AppState::OnExit   (outermost last)
```

And enter order is:
```
AppState::OnEnter  (outermost first)
GameState::OnEnter
RunState::OnEnter
NodeState::OnEnter (innermost last)
```

The `should_exist` mechanism triggers automatically. When `AppState` transitions from `Game` to something else:
1. `GameState::should_exist(AppState::Teardown)` returns `None` → `State<GameState>` is removed
2. `RunState::should_exist(GameState::???)` — but `State<GameState>` no longer exists → `RunState::should_exist` receives `None` (via `Option<GameState>`) → returns `None` → `State<RunState>` is removed
3. Same cascades to `NodeState`

**Important**: For this cascade to work automatically, the deeper SubStates MUST use `Option<ParentState>` as their source type, not bare `ParentState`. If you use bare (non-optional) `GameState` as the source for `RunState`, the behavior when `State<GameState>` is absent is undefined/panics.

The `#[source(GameState = GameState::Run)]` derive macro **does handle this correctly** — it generates the appropriate `Option`-unwrapping logic in `should_exist`.

### 5. Known issues or performance concerns?

**No known bugs** specific to 4-level nesting as of 0.18.0/0.18.1.

**Performance**: Each SubState level adds:
- One `apply_state_transition` system per frame in `StateTransition` schedule
- One `StateTransitionEvent<S>` message buffer
- One `State<S>` + `NextState<S>` resource pair (when active)

For 3 additional SubState levels, this overhead is negligible. The `StateTransition` schedule runs once per frame in `PreUpdate` — it is not on a hot path.

**One genuine footgun**: The Bevy migration guide note that `next_state.set(S)` **always triggers OnEnter/OnExit in 0.18**, even when transitioning to the same variant. Use `next_state.set_if_neq(S)` when you want the old behavior. This applies at every nesting level.

---

## The Recommended Pattern (from official docs)

The official docs show a ComputedState intermediary as the "recommended" approach **for complex cases** — specifically when the source state uses struct variants (e.g., `AppState::InGame { paused: bool }`) and you need to match on fields. For simple enum variant matching (which your hierarchy uses), **direct chaining is simpler and fully supported**:

```rust
// This direct chain is valid and idiomatic for simple variant matching:
#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::Game)]
enum GameState { Loading, Menu, Run, Teardown }

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(GameState = GameState::Run)]
enum RunState { Loading, Setup, Node, ChipSelect, End, Teardown }

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(RunState = RunState::Node)]
enum NodeState { Loading, AnimateIn, Playing, AnimateOut, Teardown }
```

App registration order must follow dependency order:

```rust
app.init_state::<AppState>()
   .add_sub_state::<GameState>()   // depends on AppState
   .add_sub_state::<RunState>()    // depends on GameState
   .add_sub_state::<NodeState>();  // depends on RunState
```

---

## Exact Hierarchy Verification

The proposed hierarchy:

```rust
#[derive(States, ...)]
enum AppState { Loading, Game, Teardown }

#[derive(SubStates, ...)]
#[source(AppState = AppState::Game)]
enum GameState { Loading, Menu, Run, Teardown }

#[derive(SubStates, ...)]
#[source(GameState = GameState::Run)]
enum RunState { Loading, Setup, Node, ChipSelect, End, Teardown }

#[derive(SubStates, ...)]
#[source(RunState = RunState::Node)]
enum NodeState { Loading, AnimateIn, Playing, AnimateOut, Teardown }
```

**Is valid.** All four questions answered with Yes. The type system allows it, the scheduler handles it, teardown cascades correctly, and OnEnter/OnExit fire at every level.

---

## Summary Table

| Question | Answer |
|----------|--------|
| SubStates sourced from SubStates? | Yes — SubStates: States satisfies S: States bound |
| Depth limit? | None — DEPENDENCY_DEPTH is just a usize ordering hint |
| OnEnter/OnExit at all levels? | Yes — registered independently per type |
| Cascading teardown when AppState leaves Game? | Yes — in correct order (innermost exits first) |
| Performance concerns at 4 levels? | None — negligible overhead |
| Known bugs in 0.18.1? | None for this pattern |

---

## Caveats

1. `next_state.set()` always fires OnEnter/OnExit in Bevy 0.18 — use `set_if_neq()` to avoid spurious transitions
2. The `run_exit` function uses `try_run_schedule` — no OnExit handler is silently OK (no panic)
3. Official examples only show 1-level SubState hierarchies — the 4-level pattern is type-safe and mechanically correct, but it is not a documented/tested path in the Bevy examples repo
4. If any variant in the hierarchy is set to an intermediate value (e.g., `GameState::Teardown`) while a deeper SubState is active, that SubState's `should_exist` will return `None` and the state will be removed — teardown sequences must account for this
