# Event-Driven Route Conditions — Bevy 0.18.1 Research

Verified against: `docs.rs/bevy/0.18.1`, `github.com/bevyengine/bevy/tree/v0.18.1`

---

## Summary Answer

Bevy 0.18 has **no native observer mechanism for resource changes**. The two viable approaches
are:

1. **Polling with `resource_exists_and_changed<R>`** — checks `Res<R>.is_changed()` every frame,
   runs the routing system only when the resource was mutated. Near-zero cost when nothing
   changes. This is the simplest and most idiomatic option for the `when()` pattern.

2. **Message-based (fully event-driven)** — child state teardown sends a custom message; the
   parent route's `when()` is `on_message::<ChildSignal>()`. True zero-cost when idle. Requires
   the child to explicitly emit the signal.

`condition_changed_to` can wrap any condition and fire only on the false→true edge, giving
edge-triggered semantics on top of polled conditions.

---

## 1. `Changed<R>` Detection for Resources

### `Res<R>.is_changed()` — YES, available

`Res<T>` implements `DetectChanges`, which provides:

```rust
fn is_changed(&self) -> bool   // true if mutably dereferenced since last system run
fn is_added(&self)   -> bool   // true if inserted since last system run
fn last_changed(&self) -> Tick
fn added(&self)      -> Tick
```

**Important caveat**: "Changed" means "mutably dereferenced" — even `ResMut<R>` access without
a value change sets the changed flag. Bevy does not compare values.

### `resource_changed<T>` run condition

```rust
// Module path:
use bevy::ecs::schedule::common_conditions::resource_changed;
// Also in bevy::prelude::*

pub fn resource_changed<T>(res: Res<'_, T>) -> bool
where T: Resource
```

- Returns `true` if resource was added or mutably accessed since the condition was last checked.
- **Panics if the resource does not exist.**
- Implementation: `res.is_changed()`

### `resource_exists_and_changed<T>` — the safer variant

```rust
pub fn resource_exists_and_changed<T>(res: Option<Res<'_, T>>) -> bool
where T: Resource
```

- Returns `false` (not panic) if the resource is absent.
- Identical behavior otherwise.
- **Use this** when the resource may not exist at the time the condition runs.

### `Ref<R>` — component change detection, NOT resources

`Ref<T>` is for **component** shared borrows with change detection. It is NOT a `SystemParam`
for resources. Use `Res<T>` (which implements `DetectChanges`) for resource change detection.
There is no `Ref<Res<T>>` or `Changed<Res<T>>` — those are component filter concepts.

### Verdict: Question 1

**YES** — `resource_exists_and_changed<R>` gives you polling that costs nearly nothing when
the resource is not mutated. Used as a `run_if`, the routing system body never executes unless
something touched the resource that frame.

Cost model:
- Condition check: O(1) tick comparison (a single integer compare against the system's last-run tick)
- System body: zero cost when condition is false

---

## 2. Resource Mutation Observers

### NO — Observers do not support resource-level triggers

Bevy 0.18 Observers are a "push-based tool for responding to `Event`s." The built-in trigger
types are:

- `OnAdd<C>` — component added to an entity
- `OnInsert<C>` — component inserted into an entity
- `OnReplace<C>` — component replaced on an entity
- `OnRemove<C>` — component removed from an entity
- Custom `Event` types triggered via `world.trigger()` / `commands.trigger()`

Resources are singletons outside the entity-component archetype system. They do NOT fire
`OnAdd`, `OnRemove`, or any observer trigger when inserted, mutated, or removed. This is
confirmed by the source: `insert_resource()` in `world/mod.rs` contains no observer
notification, and `bevy_ecs/src/observer/mod.rs` has no resource-level trigger types.

### Workaround: `insert_resource` + custom Message

If you control the code that inserts/removes the resource, you can emit a custom message
immediately after: `writer.write(ResourceChangedSignal)`. This achieves true event-driven
behavior at the cost of one extra coupling point.

### Verdict: Question 2

**NO** — Observers cannot watch resource insertion or mutation in Bevy 0.18. Polling via
`resource_exists_and_changed` is the only built-in mechanism.

---

## 3. Message-Based Approach (Custom Signal Types)

This is fully supported and the recommended event-driven approach.

### How it works

```rust
// Define a signal message:
#[derive(Message, Clone)]
struct NodeExited;

// Child teardown sends it:
fn node_teardown_system(mut writer: MessageWriter<NodeExited>) {
    writer.write(NodeExited);
}

// Parent route condition:
.run_if(on_message::<NodeExited>())
```

`on_message::<M>()` is confirmed in Bevy 0.18:

```rust
// Source (bevy_ecs/src/schedule/condition.rs):
pub fn on_message<M: Message>(mut reader: MessageReader<M>) -> bool {
    reader.read().count() > 0
}
```

- Returns `true` only when at least one new message exists in the buffer.
- Uses its own `MessageReader` with an independent cursor — consuming the condition's cursor
  does NOT affect the routing system's own `MessageReader` cursor.
- Truly zero-cost when no message is written: the buffer is empty, the check is O(1).

### Verdict: Question 3

**YES** — `on_message` is fully event-driven with zero polling cost. The trade-off is that the
child state teardown system must explicitly send the message. This is consistent with the
project's message-driven architecture.

---

## 4. OnExit as a Trigger for Parent Routes

### Does `OnExit<S>` fire when a SubState is removed?

**YES.** Confirmed from `transitions.rs`:

When `State<S>` is removed (because the SubState's source condition is no longer met),
`internal_apply_state_transition` calls the `OnExit(S::variant)` schedule. The teardown order
is innermost-first:

```
NodeState::OnExit → RunState::OnExit → GameState::OnExit
```

Additionally, a `StateTransitionEvent<NodeState>` message is sent with:
```rust
StateTransitionEvent<S> {
    exited: Some(previous_node_state),
    entered: None,
    allow_same_state_transitions: bool,
}
```

### Can we use OnExit to trigger the parent route?

Sort of. `OnExit(NodeState::Teardown)` IS a schedule that runs when NodeState leaves Teardown.
Systems added to that schedule fire when the transition happens. However:

- `OnExit` schedules run **inside** `StateTransition` (the state machine's own internal
  schedule), not as normal `Update` or `FixedUpdate` systems.
- You cannot use `run_if(on_message::<StateTransitionEvent<NodeState>>())` to condition a
  route transition that itself causes a state change, because `NextState` queued during
  `StateTransition` takes effect the NEXT frame.

### The key insight about the scenario described in the question

The scenario is: NodeState::Teardown completes → RunState should change.

The clean way: a system in `OnExit(NodeState::Teardown)` sets `NextState<RunState>`. This is
direct and correct. The parent route "owns" this via a `when()` clause if and only if it
watches for `StateTransitionEvent<NodeState>` or a custom signal from child teardown.

Alternatively: a route on `NodeState::Teardown` with `to_dynamic(...)` using cross-level
routing (setting `NextState<RunState>`) already works per the existing routing system.

### Verdict: Question 4

`OnExit` fires correctly for SubState removal. A system placed in `OnExit(NodeState::Teardown)`
CAN set `NextState<RunState>`. But this is the existing "child route sets parent state" pattern.
The `when()` approach on the parent is a different ownership model where the parent monitors
a condition without requiring the child to own the transition.

---

## 5. SubState Existence as a Condition

### `state_exists<S>` run condition

```rust
// Module path:
use bevy::prelude::state_exists;  // re-exported in prelude

pub fn state_exists<S: States>(current_state: Option<Res<State<S>>>) -> bool {
    current_state.is_some()
}
```

- Returns `true` if `State<S>` resource exists.
- Returns `false` (does not panic) if the resource is absent.
- Works for both top-level `States` and `SubStates`.

### Detecting "NodeState no longer exists"

```rust
.run_if(not(state_exists::<NodeState>()))
```

This fires every frame that NodeState is absent. Not what you want for a route trigger —
you'd fire the route every frame after the SubState is gone.

### `state_changed<S>` — fires once on the transition frame

```rust
pub fn state_changed<S: States>(current_state: Option<Res<State<S>>>) -> bool {
    let Some(current_state) = current_state else {
        return false;
    };
    current_state.is_changed()
}
```

**Limitation**: Uses `Res<State<S>>.is_changed()`. This fires when the state transitions to a
new value, but it does NOT fire when the state resource is REMOVED (because the `Option` is
`None` when removed, and the function returns `false` in that case). So `state_changed<NodeState>`
does not detect NodeState being torn down entirely.

### `condition_changed` and `condition_changed_to` — edge detection wrappers

These wrap any condition and fire only when the condition's output changes:

```rust
// Fires on BOTH edges (false→true and true→false):
pub fn condition_changed<Marker, CIn, C>(condition: C) -> impl SystemCondition<(), CIn>

// Fires only when the wrapped condition transitions to the specified bool:
pub fn condition_changed_to<Marker, CIn, C>(to: bool, condition: C) -> impl SystemCondition<(), CIn>
```

To detect "NodeState went from existing to not-existing" (the removal edge):

```rust
.run_if(condition_changed_to(false, state_exists::<NodeState>()))
```

This fires exactly once on the frame when `state_exists::<NodeState>()` transitions from
`true` to `false`. This is an edge-triggered polling pattern — it effectively polls every
frame but the routing system body executes only once.

**Gotcha**: `condition_changed_to` uses `Local<bool>` to track the previous value. The initial
assumed previous value is `false`. If `NodeState` does not exist when the app starts, the
condition fires immediately on the first frame (because `false` → `false` is NOT a change).
If it exists on app start and then is removed, it fires once on removal. This is correct for
this use case.

### Verdict: Question 5

`condition_changed_to(false, state_exists::<NodeState>())` detects "NodeState ceased to exist"
as a one-shot edge trigger. Still polling (the condition runs every frame), but the system body
runs only once. Cost: one `Option<Res>` lookup per frame.

---

## 6. Custom Message as `when()` Trigger

This is the fully event-driven approach.

### Pattern

```rust
// Child state signals completion:
#[derive(Message, Clone)]
struct PhaseExited<S: States + Clone>(PhantomData<S>);
// Or a concrete type per state.

// In NodeState teardown:
fn emit_node_exited(mut writer: MessageWriter<NodeExited>) {
    writer.write(NodeExited);
}
app.add_systems(OnExit(NodeState::Teardown), emit_node_exited);

// Parent route when() condition:
.when(on_message::<NodeExited>())
```

The `when()` on the parent route becomes `run_if(on_message::<NodeExited>())`. When no
message exists, the condition returns `false` immediately (buffer empty check = O(1)).
The routing system body never executes.

### How `on_message` combines with state conditions

```rust
// run_if accepts a single condition — combine with .and():
.run_if(on_message::<NodeExited>().and(in_state(RunState::Node)))
```

The `.and()` combinator is available on any `SystemCondition` (the `Condition` trait provides
it). It short-circuits: if `on_message` returns false, `in_state` is never evaluated.

```rust
// Available combinators:
fn and<M, C: SystemCondition<M, In>>(self, and: C) -> And<Self::System, C::System>
fn or<M, C: SystemCondition<M, In>>(self, or: C) -> Or<Self::System, C::System>
fn nand<M, C: SystemCondition<M, In>>(self, nand: C) -> Nand<Self::System, C::System>
fn nor<M, C: SystemCondition<M, In>>(self, nor: C) -> Nor<Self::System, C::System>
```

All are methods on the `Condition` trait (named `SystemCondition` in the source). Available
in any Bevy 0.18 schedule.

### Verdict: Question 6

**YES** — a custom message is the cleanest event-driven approach. Requires the child to
explicitly send the signal. The `when()` becomes `on_message::<Signal>()`, which is zero-cost
when idle.

---

## 7. Combining `on_message` with State Checks in `run_if`

**YES, fully supported.**

```rust
.run_if(on_message::<NodeExited>().and(in_state(RunState::Node)))
```

Both `on_message` and `in_state` implement `SystemCondition` (which is `IntoSystem<(), bool,
Marker>` where the system is `ReadOnlySystem`). The `.and()` method is provided on any value
implementing `SystemCondition` and returns another `SystemCondition`.

Short-circuit behavior:
- `.and()`: evaluates left first, short-circuits to false if left is false
- `.or()`: evaluates left first, short-circuits to true if left is true

This means `on_message::<NodeExited>().and(in_state(RunState::Node))` skips the state check
entirely when no message has arrived — optimal ordering.

### Verified: `on_message` cursor independence

**Confirmed in memory**: `on_message` uses its own `MessageReader` with its own
`Local<MessageCursor<M>>`. The condition advancing its cursor does NOT affect the routing
system body's own `MessageReader` cursor. The system body can still read all messages via its
own `MessageReader<NodeExited>`.

### Verdict: Question 7

**YES** — `run_if(on_message::<M>().and(in_state(S::Variant)))` works correctly in Bevy 0.18.

---

## Approach Comparison

| Approach | Event-driven? | Cost when idle | Requires child to signal? | Notes |
|---|---|---|---|---|
| Polling `resource_exists_and_changed<R>` | No (polled) | ~O(1) tick compare | No | Clean; mutResMut touch triggers it |
| `on_message::<Signal>()` | **Yes** | O(1) buffer empty check | Yes | Best for architecture; zero overhead |
| `condition_changed_to(false, state_exists::<S>())` | No (polled) | ~O(1) per frame | No | Fires once on removal edge |
| `state_changed<S>` | No (polled) | ~O(1) | No | Does NOT detect state removal |
| Observer on resource | N/A | — | — | NOT SUPPORTED in Bevy 0.18 |
| `OnExit(NodeState::Variant)` schedule | **Yes** | Zero | Implicit (state machine fires it) | Only works if route is inside a schedule, not run_if |

---

## Recommendation for the `when()` Pattern

### For resource-flag conditions
Use `resource_exists_and_changed<R>` as the `when()` condition. Polling, but near-zero cost.
If you want edge-triggered behavior (fire once, not every frame the resource is changed),
wrap it: `condition_changed_to(true, resource_exists_and_changed::<R>())`.

### For child-state-exited conditions
Use a custom message. Child teardown sends `PhaseComplete<NodeState>` or a dedicated signal.
Parent route uses `on_message::<Signal>()` as its `when()`. This is already what
`PhaseComplete<S>` does — the `when()` is just `on_message::<PhaseComplete<NodeState>>()`.

### For "child SubState was removed" without child cooperation
Use `condition_changed_to(false, state_exists::<NodeState>())`. Fires once on the removal
frame. The child does not need to send anything — the state machine itself removes the
`State<NodeState>` resource.

### The actual scenario from the question

> NodeState::Teardown completes → RunState should change

The cleanest approach that keeps ownership at the RunState level:

```rust
// Route on RunState:
from(RunState::Node)
    .to_dynamic(|...| RunState::next(...))
    .when(condition_changed_to(false, state_exists::<NodeState>()))
```

This fires automatically when NodeState ceases to exist (because RunState left Node, which
removes the SubState). No child cooperation required.

Or with a message if you prefer the explicit handshake:

```rust
// In NodeState teardown:
.add_systems(OnExit(NodeState::Teardown), |mut w: MessageWriter<NodeTeardownComplete>| {
    w.write(NodeTeardownComplete);
})

// Route on RunState:
from(RunState::Node)
    .to_dynamic(|...| RunState::next(...))
    .when(on_message::<NodeTeardownComplete>())
```

---

## Key Facts for Implementation

1. **`resource_exists_and_changed<T>`** — safe (no panic), returns false when resource absent.
   Signature: `fn(Option<Res<'_, T>>) -> bool`. Use this over `resource_changed` for optional resources.

2. **`on_message<M>`** — zero-cost when idle. Has independent cursor; does not conflict with
   system body's `MessageReader`. In `bevy::ecs::schedule::common_conditions` and `bevy::prelude`.

3. **`state_exists<S>`** — `fn(Option<Res<State<S>>>) -> bool`. Safe when state absent.

4. **`condition_changed_to(to: bool, condition)`** — edge detector. Fires once when wrapped
   condition transitions to `to`. Uses `Local<bool>` for previous-value tracking. Initial assumed
   previous is `false`.

5. **`.and()` / `.or()`** — available on any `SystemCondition`. Short-circuit. Can chain:
   `cond_a.and(cond_b).or(cond_c)`.

6. **Observers cannot watch resources** — only entity/component lifecycle events.

7. **`state_changed<S>`** uses `is_changed()` on `Res<State<S>>` — returns false (not true)
   when the state resource is removed. Does NOT detect SubState removal.

8. **`StateTransitionEvent<S>`** fires when a state transitions (including removal with
   `entered: None`). This IS a `Message`, so `on_message::<StateTransitionEvent<NodeState>>()`
   works for detecting NodeState removal.
