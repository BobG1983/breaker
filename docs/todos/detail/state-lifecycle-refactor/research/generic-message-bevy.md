# Research: Generic `#[derive(Message)]` in Bevy 0.18.1

**Verified against**: Bevy 0.18.1 (`docs.rs/bevy/0.18.1`, `docs.rs/bevy_ecs_macros/0.18.1`,
Bevy source `v0.18.1` tag)

---

## Summary: All Four Questions Answered

**Short answer**: Yes to everything. Generic structs work with `#[derive(Message)]`, manual
`impl Message` also works, each instantiation registers a separate `Messages<T>` resource,
and `MessageReader`/`MessageWriter` have no extra bounds beyond `M: Message`.

---

## Question 1: Does `#[derive(Message)]` work on generic structs?

**Yes. Confirmed.**

The `derive_message` proc macro in `bevy_ecs_macros/src/message.rs` (lines 5–19):

1. Calls `ast.generics.split_for_impl()` to extract `impl_generics`, `type_generics`, and
   `where_clause` from the struct's existing generics.
2. Appends `Self: Send + Sync + 'static` to the where clause.
3. Emits: `impl #impl_generics Message for #struct_name #type_generics #where_clause {}`

The impl block is empty — `Message` is a marker trait with no methods.

**Real-world proof**: `StateTransitionEvent<S: States>` in `bevy_state/src/state/transitions.rs`
uses this exact pattern:

```rust
#[derive(Debug, Copy, Clone, PartialEq, Eq, Message)]
pub struct StateTransitionEvent<S: States> {
    pub exited: Option<S>,
    pub entered: Option<S>,
    pub allow_same_state_transitions: bool,
}
```

That is `#[derive(Message)]` on a generic struct with a bounded type parameter, and it is
part of Bevy's own codebase — so it definitely works.

### What the derive generates for your structs

For `ChangeState<S: States>(PhantomData<S>)`, the derive emits:

```rust
impl<S: States> Message for ChangeState<S>
where
    Self: Send + Sync + 'static,
{}
```

For `StateChanged<S: States> { from: S, to: S }`, it emits:

```rust
impl<S: States> Message for StateChanged<S>
where
    Self: Send + Sync + 'static,
{}
```

The `Self: Send + Sync + 'static` bound is automatically satisfied when `S: States`, because
`States` is defined as:

```rust
pub trait States: 'static + Send + Sync + Clone + PartialEq + Eq + Hash + Debug {
    const DEPENDENCY_DEPTH: usize = 1;
}
```

So `S: States` implies `S: Send + Sync + 'static`, which makes the entire struct `Send + Sync + 'static`.

**All four of your proposed structs will compile with `#[derive(Message, Clone)]`.**

---

## Question 2: Can you manually `impl Message for ChangeState<S> where S: States`?

**Yes. Also works.**

`AssetEvent<A>` and `AssetLoadFailedEvent<A>` use manual impls:

```rust
impl<A> Message for AssetEvent<A>
where
    A: Asset,
    AssetEvent<A>: Send + Sync + 'static,
{}
```

For your types, the manual impl would be:

```rust
impl<S: States> Message for ChangeState<S> {}
```

Because `S: States` already implies `Send + Sync + 'static` transitively, the additional
`Self: ...` bound is redundant — the compiler infers it. The derive macro adds it explicitly
as a safety belt, but you don't have to when writing a manual impl.

**Recommendation**: Use `#[derive(Message)]` — it's less code and behaves identically.

---

## Question 3: Does `app.add_message::<ChangeState<NodeState>>()` register a separate resource from `app.add_message::<ChangeState<RunState>>()`?

**Yes. Completely separate resources.**

`add_message::<M>()` signature:

```rust
pub fn add_message<M>(&mut self) -> &mut App
where M: Message
```

What it does internally:
1. Inserts `Messages::<M>` as a `Resource` (Bevy resources are keyed by `TypeId`)
2. Schedules `message_update_system` for that specific `M`

In Rust's type system, `ChangeState<NodeState>` and `ChangeState<RunState>` are **distinct
types** with distinct `TypeId` values. Therefore:

- `Messages::<ChangeState<NodeState>>` is one resource
- `Messages::<ChangeState<RunState>>` is a completely separate resource
- They do not share a buffer, cursor, or any state

You must call `add_message` once per concrete type you use:

```rust
app.add_message::<ChangeState<NodeState>>()
   .add_message::<ChangeState<RunState>>()
   .add_message::<StateChanged<NodeState>>()
   .add_message::<StateChanged<RunState>>();
   // etc.
```

Forgetting to register one will cause a runtime panic (or silent no-op depending on access
pattern) when `MessageWriter<T>` or `MessageReader<T>` tries to access the unregistered
`Messages<T>` resource.

---

## Question 4: Any issues with `MessageReader<ChangeState<NodeState>>` and `MessageWriter<ChangeState<NodeState>>`?

**No issues.**

Both types are defined as:

```rust
pub struct MessageReader<'w, 's, E> where E: Message { /* private fields */ }
pub struct MessageWriter<'w, E> where E: Message { /* private fields */ }
```

The only bound is `E: Message`. No `Reflect`, `TypePath`, or any other bound.

Since `ChangeState<NodeState>` satisfies `Message` (via the derive), it satisfies the only
required bound. `MessageReader<ChangeState<NodeState>>` and `MessageWriter<ChangeState<NodeState>>`
will work as `SystemParam`s in any system.

---

## Concrete Code That Will Compile

```rust
use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Message, Clone)]
struct ChangeState<S: States>(PhantomData<S>);

#[derive(Message, Clone)]
struct StateChanged<S: States> {
    from: S,
    to: S,
}

#[derive(Message, Clone)]
struct TransitionStart<S: States> {
    from: S,
    to: S,
}

#[derive(Message, Clone)]
struct TransitionEnd<S: States> {
    from: S,
    to: S,
}
```

Registration (once per concrete type in use):

```rust
app.add_message::<ChangeState<NodeState>>()
   .add_message::<ChangeState<RunState>>()
   .add_message::<StateChanged<NodeState>>()
   .add_message::<StateChanged<RunState>>()
   .add_message::<TransitionStart<NodeState>>()
   .add_message::<TransitionStart<RunState>>()
   .add_message::<TransitionEnd<NodeState>>()
   .add_message::<TransitionEnd<RunState>>();
```

System usage:

```rust
fn handle_change_state(
    mut reader: MessageReader<ChangeState<NodeState>>,
    mut next: ResMut<NextState<NodeState>>,
) {
    for msg in reader.read() {
        // PhantomData<NodeState> — need to encode the target state differently
        // (see Gotcha #1 below)
    }
}
```

---

## Gotchas

### Gotcha 1: `ChangeState<S>(PhantomData<S>)` carries no data

`PhantomData<S>` is a zero-sized type — it carries no runtime value. A `ChangeState<NodeState>`
message tells the receiver "transition to some NodeState" but cannot specify *which* NodeState
variant to transition to.

If you need to convey the target state value, the struct must carry it:

```rust
#[derive(Message, Clone)]
struct ChangeState<S: States> {
    target: S,
}
```

This still derives `Message` correctly — the field type `S` is `Clone + Send + Sync + 'static`
(from `States` bounds), so the struct satisfies all requirements.

### Gotcha 2: Registration must happen before any system accesses the message type

`app.add_message::<T>()` must be called during app build (in a plugin's `build()` method)
before any system that reads or writes `Messages<T>` runs. The system will panic at runtime
if the resource is missing.

### Gotcha 3: `#[derive(Message)]` requires `bevy_ecs_macros` in scope

In Bevy 0.18.1, `Message` is re-exported from `bevy::prelude::*`. The derive macro comes
from `bevy_ecs_macros` which is wired in when you use the `bevy` umbrella crate. With the
project's current setup (`bevy = { version = "0.18.1", ... }`), `#[derive(Message)]` works
out of the box with `use bevy::prelude::*`.

### Gotcha 4: Each state in the hierarchy needs separate registrations

With a 4-level state hierarchy (AppState → GameState → RunState → NodeState), each
`ChangeState<X>` instantiation is its own type. If you have N state types and want to drive
all of them via generic messages, that is `N × (number of message types)` registrations.
Consider whether this many registrations is worth the generic abstraction, or whether
type aliases and explicit per-state message types are clearer.

---

## Source References

- `Message` trait: `docs.rs/bevy/0.18.1/bevy/ecs/message/trait.Message.html`
  — bounds: `Send + Sync + 'static`
- `Messages<M>` struct: `docs.rs/bevy/0.18.1/bevy/ecs/message/struct.Messages.html`
  — bound: `M: Message`
- `MessageReader<E>` struct: `docs.rs/bevy/0.18.1/bevy/ecs/message/struct.MessageReader.html`
  — bound: `E: Message`
- `derive_message` macro: `docs.rs/bevy_ecs_macros/0.18.1/src/bevy_ecs_macros/message.rs.html`
  — uses `split_for_impl()`, appends `Self: Send + Sync + 'static`
- `StateTransitionEvent<S>` source: `docs.rs/bevy_state/0.18.1/src/bevy_state/state/transitions.rs.html`
  — `#[derive(Debug, Copy, Clone, PartialEq, Eq, Message)]` on a generic struct, confirmed
- `AssetEvent<A>` — manual `impl<A> Message for AssetEvent<A>` (alternative pattern)
- `add_message` method: `docs.rs/bevy/0.18.1/bevy/prelude/struct.App.html`
  — `pub fn add_message<M: Message>(&mut self) -> &mut App`
