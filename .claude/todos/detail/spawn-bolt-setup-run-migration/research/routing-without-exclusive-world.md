# Research: Routing Without Exclusive World Access

**Bevy version**: 0.18.1  
**Verified against**: docs.rs/bevy/0.18.1, github.com/bevyengine/bevy/tree/v0.18.1

---

## Question 1: Can we avoid exclusive World access?

### Short answer: Yes — use `Commands::run_system` from a normal system.

The exclusive `&mut World` approach is not necessary. Here is what Bevy 0.18.1 provides:

### Option C is the best choice: `Commands::run_system`

**Verified API — `Commands::run_system`**

```rust
pub fn run_system(&mut self, id: SystemId)
// Generic form with input:
pub fn run_system_with<I>(&mut self, id: SystemId<I>, input: <I as SystemInput>::Inner<'static>)
```

`SystemId` is `Copy + Send + Sync`, so it can be stored in a `Resource`.

**How this enables non-exclusive routing:**

```rust
// Step 1: Register dynamic route systems at app build time
fn build(app: &mut App) {
    let id = app.world_mut().register_system(evaluate_node_result_route);
    app.insert_resource(RouteSystemIds { node_result: id, ... });
}

// Step 2: Normal system reads message, looks up route, dispatches via commands
fn route_phase_complete(
    mut reader: MessageReader<PhaseComplete<NodeState>>,
    routing_table: Res<RoutingTable<NodeState>>,
    route_ids: Res<RouteSystemIds>,
    mut commands: Commands,
) {
    for _msg in reader.read() {
        match routing_table.lookup(/* current state */) {
            Route::Static(target) => {
                commands.queue(move |world: &mut World| {
                    world.resource_mut::<NextState<NodeState>>()
                        .set_if_neq(target);
                });
            }
            Route::Dynamic => {
                commands.run_system(route_ids.node_result);
            }
        }
    }
}

// Step 3: Dynamic route system — normal system, reads what it needs
fn evaluate_node_result_route(
    node_result: Res<NodeResult>,
    mut next: ResMut<NextState<NodeState>>,
) {
    let target = match node_result.outcome {
        Outcome::Cleared => NodeState::ChipSelect,
        Outcome::Lost => NodeState::RunEnd,
    };
    next.set_if_neq(target);
}
```

**When does `Commands::run_system` execute?**

Deferred — at the next `ApplyDeferred` sync point after the calling system. Within the same schedule, this is the same frame. The documentation states: "There is no way to get the output of a system when run as a command, because the execution of the system happens later." The system runs exclusively (`&mut World` access) but only when dispatched, not every frame.

**Constraint**: You cannot get the return value of the one-shot system back into the calling system. This is fine for routing — the route system sets `NextState` directly.

---

### Option A: Split static and dynamic into separate systems

**Verdict: Workable but awkward.**

Static routes can use a normal system with `Res<RoutingTable<S>>` + `ResMut<NextState<S>>`. Dynamic routes need either:
- The one-shot system approach (Option C above — preferred), or
- A second exclusive system that only runs when a `PendingDynamicRoute` marker resource is present

The split approach creates two coordinating systems where one is cleaner. Use Option C instead.

---

### Option B: Observers

**Verdict: Not applicable to Message types.**

Observers in Bevy 0.18 respond to `Event` types triggered via `World::trigger()` or `Commands::trigger()`. They do **not** respond to `Messages<T>`. Observers use `On<E>` as their first system param — there is no `On<PhaseComplete<S>>` if `PhaseComplete<S>` is a `Message`.

Additionally, Observers receive standard `SystemParam` types (not `&mut World` directly), so they don't simplify resource access compared to normal systems.

**If** the routing trigger were converted from a `Message` to a Bevy `Event` and triggered via `Commands::trigger`, an Observer could work. But changing the message protocol is a larger refactor and not warranted here.

---

### Option D: `SystemParam` with dynamic dispatch

**Verdict: Not viable in Bevy 0.18.**

There is no `SystemParam` mechanism for "read whatever resources this closure needs." The `SystemParam` derive requires statically declared params. The only dynamic dispatch option is one-shot systems registered at startup (Option C).

---

### Recommendation for Question 1

Use `Commands::run_system`:

1. Register each dynamic route as a one-shot system during app build: `world.register_system(my_route_fn)` → returns `SystemId` (Copy)
2. Store `SystemId`s in a `Resource` (e.g., `DynamicRouteIds`)
3. Normal routing system reads messages, looks up routes, calls `commands.run_system(id)` for dynamic routes
4. Static routes can be applied inline via a `commands.queue(|w: &mut World| { ... })` closure or directly via `ResMut<NextState<S>>`
5. No per-frame exclusive system — only one-shot execution when a message arrives

This gives you: normal scheduling, no per-frame exclusive lock, dynamic resource reads in the one-shot systems, and `set_if_neq` correctness.

---

## Question 2: Can we conditionally run only when `PhaseComplete<S>` messages exist?

### Short answer: Yes — use `on_message::<PhaseComplete<S>>()` as a run condition, but with a critical caveat.

### Verified API — `on_message` run condition

**Exact signature** (verified from source):

```rust
pub fn on_message<M: Message>(reader: MessageReader<'_, '_, M>) -> bool
```

**Module path**: `bevy::ecs::schedule::common_conditions::on_message`  
Re-exported in `bevy::prelude::*`.

**Usage**:

```rust
app.add_systems(
    Update,
    route_phase_complete.run_if(on_message::<PhaseComplete<NodeState>>()),
);
```

The system only runs when there are pending `PhaseComplete<NodeState>` messages. No per-frame exclusive world lock.

---

### CRITICAL CAVEAT: `on_message` consumes messages via its own cursor

**The `on_message` condition calls `reader.read().count() > 0` internally.** From the source:

```rust
pub fn on_message<M: Message>(mut reader: MessageReader<M>) -> bool {
    reader.read().count() > 0
    // Comment: "The messages need to be consumed, so that there are no false positives
    // on subsequent calls of the run condition. Simply checking is_empty would not be enough."
}
```

**Each `MessageReader` (including the one in the run condition) gets its own independent `MessageCursor` stored as `Local<MessageCursor<M>>`.**

This means: if `on_message` returns `true` and the system fires, the `MessageReader` inside the system body sees the SAME messages (because the condition's cursor is independent from the system body's cursor). The condition advancing its own cursor does NOT consume messages from the system body's reader.

**Verified**: MessageCursor is stored as `Local<>` per system param instance. Multiple readers for the same type are fully independent — each tracks its own position through the double-buffered message queue.

---

### Option A: `run_if(on_message::<T>())`

```rust
fn route_phase_complete(
    mut reader: MessageReader<PhaseComplete<NodeState>>,  // own cursor — independent
    // ...
) {
    for _msg in reader.read() { /* process */ }
}

// Correct usage:
.add_systems(Update, route_phase_complete.run_if(on_message::<PhaseComplete<NodeState>>()))
```

This works correctly. The system body's `MessageReader` sees the messages independently of the run condition's reader.

---

### Option B: Observer-based (skip if staying with Messages)

As established in Q1, Observers don't fire on `Message` types. Would require converting `PhaseComplete<S>` to a triggered `Event`. Only viable if you change the messaging protocol.

---

### Option C: Resource flag

```rust
#[derive(Resource, Default)]
struct PhaseCompletePending;

// Set flag when message is sent:
fn flag_setter(mut reader: MessageReader<PhaseComplete<NodeState>>, mut commands: Commands) {
    if reader.read().count() > 0 {
        commands.insert_resource(PhaseCompletePending);
    }
}

// Route system:
fn route_phase_complete(/* ... */) { /* ... */ }
app.add_systems(Update, route_phase_complete.run_if(resource_exists::<PhaseCompletePending>));
```

This is more complex than Option A and introduces an extra system. Use `on_message` directly (Option A).

---

### Recommendation for Question 2

Use `run_if(on_message::<PhaseComplete<S>>())`. It is idiomatic, has no false-positive risk (the condition's cursor advances independently), and the system body's `MessageReader` sees all messages normally.

**No exclusive system needed per frame.** The one-shot dynamic route systems dispatched via `Commands::run_system` only execute when a message actually arrives.

---

## Question 3: What if multiple `PhaseComplete<S>` messages arrive in the same frame?

### Sub-question A: Is this a real concern?

**For a single state type, yes — handle it explicitly.**

Two code paths could both send `PhaseComplete<NodeState>` in the same frame if:
- A phase completion and an immediate re-entry both fire in the same `FixedUpdate` tick
- Two different systems in the same schedule independently decide the node is complete
- A cascading message (bolt lost → phase ends → node ends) arrives in quick succession

**For different state type parameters** (`PhaseComplete<NodeState>` vs `PhaseComplete<RunState>`), these are entirely different types with separate `Messages<T>` resources and separate routing systems — no cross-contamination.

---

### Sub-question B: If we process multiple, does order matter?

**Yes, but only the last `NextState::set_if_neq()` call wins within the same frame.**

`NextState<S>` is a `ResMut<>` with a single pending value. From the verified API:

```
NextState::Pending(S)     — set by set()
NextState::PendingIfNeq(S) — set by set_if_neq()
```

There is only one slot. Calling `set_if_neq(A)` then `set_if_neq(B)` leaves `PendingIfNeq(B)`. The `StateTransition` schedule runs **after `PreUpdate`** (verified: `schedule.insert_after(PreUpdate, StateTransition)`), processes the single pending value, and applies the transition.

**Implication**: If two `PhaseComplete<NodeState>` messages arrive in the same frame and the routing function would produce different targets for each, only the second routing call's target takes effect. This is almost certainly a bug.

---

### Sub-question C: Could cascading cause issues within the same frame?

**Partial cascade — not a full same-frame cascade.**

The `StateTransition` schedule runs after `PreUpdate`. Systems in `Update` and `FixedUpdate` run after `StateTransition`. So:

- Frame N: `PhaseComplete<NodeState>` sent in `FixedUpdate` → routing system fires → `NextState<NodeState>::set_if_neq(ChipSelect)` queued
- Frame N: `StateTransition` runs (after PreUpdate of the SAME frame? No — see below)

**Correction**: `StateTransition` runs after `PreUpdate` each frame. A message sent during `FixedUpdate` in frame N is processed by the routing system in `Update` of frame N, setting `NextState`. `StateTransition` for frame N+1 applies the transition.

**Cascade scenario**: `PhaseComplete<NodeState>` → NodeState changes to `ChipSelect` → could this trigger `PhaseComplete<RunState>` in the same frame? No — because NodeState's `OnEnter(ChipSelect)` runs in frame N+1's `StateTransition`. By the time `RunState`'s routing system could see a `PhaseComplete<RunState>`, it's a different frame.

**Conclusion**: Cross-state cascades across different state hierarchy levels are naturally frame-separated. Same-state-type cascades within the same frame are the real risk.

---

### Recommendation for Question 3

**Process only the first message per frame. Assert/warn if more than one arrives.**

```rust
fn route_phase_complete(
    mut reader: MessageReader<PhaseComplete<NodeState>>,
    routing_table: Res<RoutingTable<NodeState>>,
    current: Res<State<NodeState>>,
    mut commands: Commands,
    route_ids: Res<DynamicRouteIds>,
) {
    let messages: Vec<_> = reader.read().collect();

    // Warn on unexpected multiple messages
    if messages.len() > 1 {
        tracing::warn!(
            count = messages.len(),
            "Multiple PhaseComplete<NodeState> in same frame — only first processed"
        );
    }

    if messages.is_empty() { return; }

    // Process first message only
    match routing_table.lookup(current.get()) {
        Route::Static(target) => {
            commands.queue(move |world: &mut World| {
                world.resource_mut::<NextState<NodeState>>().set_if_neq(target);
            });
        }
        Route::Dynamic(id) => {
            commands.run_system(route_ids.get(id));
        }
    }
}
```

This is safe because:
1. `StateTransition` runs after `PreUpdate` — same frame's `NextState` changes are applied next frame
2. Cross-hierarchy cascades (`NodeState` → `RunState`) are naturally frame-separated
3. Same-type duplicate messages indicate a real logic error — warn loudly rather than silently picking a winner

---

## Summary Table

| Question | Answer | Key API |
|---|---|---|
| Q1: Avoid exclusive World? | Yes — use `Commands::run_system` with registered one-shot systems | `world.register_system(fn)` → `SystemId`; `commands.run_system(id)` |
| Q2: Conditional execution? | Yes — `run_if(on_message::<T>())` | `on_message::<M>()` in `bevy::prelude`; cursor is independent from system body's reader |
| Q3: Multiple messages same frame? | Process first, warn on duplicates; cross-state cascades are frame-separated | `NextState` has one slot; `StateTransition` runs after `PreUpdate` |

---

## Key Gotchas

1. **`on_message` advances its own cursor** — but this does NOT consume messages from the system body's `MessageReader`. They are independent. The condition fires the system; the system body reads the same messages independently.

2. **`Commands::run_system` is deferred** — executes at `ApplyDeferred` sync point, same frame. No return value available in the calling system. The one-shot system sets `NextState` directly.

3. **`StateTransition` runs after `PreUpdate`** — a `NextState` set in `Update` or `FixedUpdate` takes effect in the NEXT frame's `StateTransition` pass (since `StateTransition` already ran for the current frame by the time `Update` starts). 

   **Correction**: `StateTransition` is inserted after `PreUpdate` — it runs before `Update` in the same frame. So: `PreUpdate` → `StateTransition` → `FixedUpdate` → `Update`. A `NextState` queued during `FixedUpdate` or `Update` is applied in the NEXT frame's `StateTransition`.

4. **`SystemId` is `Copy`** — safe to store in resources and clone into closures.

5. **`set()` always triggers `OnEnter`/`OnExit` in 0.18** — use `set_if_neq()` to prevent spurious transitions when routing to the current state.

6. **Observers don't fire on `Message` types** — only on triggered `Event` types via `World::trigger` / `Commands::trigger`.
