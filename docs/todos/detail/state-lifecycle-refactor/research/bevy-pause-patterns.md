# Research: Global Pause Patterns in Bevy 0.18.1

Verified against: Bevy 0.18.1 (docs.rs), GitHub v0.18.0 source, official examples.

## Context

The new hierarchical state machine is:
```
GameState: Loading | MainMenu | Run | MetaProgression
  RunState (sub-state of GameState::Run): Setup | Node | ChipSelect | RunEnd | Teardown
    NodeState (sub-state of RunState::Node): Setup | Playing | Teardown
```

The current `PlayingState::Active / Paused` sub-state is being removed. Pausing must work
during any `RunState` variant — not just `NodeState::Playing` — so the pause mechanism
cannot be a sub-state of `NodeState`.

---

## Approach 1: `Time<Virtual>::pause()` — Pause the Virtual Clock

### How it works

`Time<Virtual>` exposes:

```rust
fn my_system(mut virtual_time: ResMut<Time<Virtual>>) {
    virtual_time.pause();      // freeze
    virtual_time.unpause();    // resume
    let p: bool = virtual_time.is_paused();
    let p: bool = virtual_time.was_paused();  // paused at start of this update
}
```

Source: docs.rs/bevy/0.18.1/bevy/time/struct.Virtual.html

### Effect on FixedUpdate

**Confirmed from source (`bevy_time/src/fixed.rs`):** `run_fixed_main_schedule` reads
`world.resource::<Time<Virtual>>().delta()` and accumulates it. When virtual time is paused,
`delta()` returns zero. Zero delta means zero overstep accumulation. The `expend()` loop
exits immediately. `FixedUpdate` does not run.

This is the cleanest physics freeze — no conditional logic needed.

### Effect on timers

Game `Timer`s that tick with `time.delta()` (virtual time) stop advancing automatically.
Timers using `Time<Real>` (e.g., UI fade timers, ramp timers) continue unaffected.
This is the correct behavior: gameplay timers freeze, UI timers do not.

### UI/input while paused

`Time<Virtual>` affects only game logic. `Update` systems reading `ButtonInput<KeyCode>` or
`MouseButton` still run — input polling is not time-gated. UI systems also run normally.
You explicitly choose which systems use virtual vs real time.

### Implementation

```rust
// Toggle pause system (reads input, toggles virtual clock):
fn toggle_pause(
    actions: Res<InputActions>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    if actions.active(GameAction::TogglePause) {
        if virtual_time.is_paused() {
            virtual_time.unpause();
        } else {
            virtual_time.pause();
        }
    }
}
```

No state transitions needed. The pause menu open/close is a separate concern (UI layer).

### run_if guard (optional)

For systems that must NOT run while paused (e.g., physics input processing), you can add:

```rust
.run_if(|vt: Res<Time<Virtual>>| !vt.is_paused())
```

Or make this a named condition:

```rust
fn not_paused(vt: Res<Time<Virtual>>) -> bool {
    !vt.is_paused()
}
```

### Complexity: LOW

- No new state type
- No SubStates registration
- No `configure_sets` changes
- Works at any `RunState` or `NodeState` without coupling to state machine
- Pause menu can open independently of state

### Gotchas

- `pause()` does not affect the **current** frame's delta (it takes effect next frame).
  Source: "Calling `pause()` will not affect the `delta()` value for the update currently
  being processed."
- Systems using `Res<Time>` (the plain alias) in `FixedUpdate` automatically get
  `Time<Fixed>` behavior — they stop ticking naturally.
- Systems using `Res<Time>` in `Update` get `Time<Virtual>` behavior — they also stop.
- Systems that must continue during pause (UI, input) should use `Time<Real>`.

### Verdict: RECOMMENDED primary mechanism

This is the cleanest approach for this project. It requires zero state machine changes,
works across all `RunState` variants, and properly freezes `FixedUpdate` (physics).

---

## Approach 2: Resource Flag (`Res<Paused>`) + `run_if`

### How it works

Insert/remove a marker resource (or use a bool resource):

```rust
#[derive(Resource)]
struct GamePaused;

// Toggle:
fn toggle_pause(
    mut commands: Commands,
    paused: Option<Res<GamePaused>>,
) {
    if paused.is_some() {
        commands.remove_resource::<GamePaused>();
    } else {
        commands.insert_resource(GamePaused);
    }
}

// Guard systems:
fn physics_system(...) { ... }
// registered as:
.add_systems(FixedUpdate, physics_system.run_if(not(resource_exists::<GamePaused>())))
```

Or with a bool resource:

```rust
#[derive(Resource, Default)]
struct Paused(bool);

fn physics_system(...) { ... }
// registered as:
.add_systems(FixedUpdate,
    physics_system.run_if(resource_equals(Paused(false)))
)
```

### Effect on FixedUpdate

**Partial.** The `run_if` condition prevents gameplay systems from executing, but
`FixedUpdate` still ticks — the schedule runs, just the guarded systems are skipped.
Time still accumulates in `Time<Fixed>`. This means: after unpausing, all the time that
elapsed during pause is instantly released as back-pressure, potentially running `FixedUpdate`
many times in one frame to "catch up."

This is **different** from Approach 1. For physics-heavy games, this can cause visible glitches.

To avoid catch-up: you would also need to reset the `Time<Fixed>` overstep accumulator on
unpause. There is no public API to do this directly in Bevy 0.18.1 without also pausing
virtual time.

### Effect on timers

Timers tick inside `FixedUpdate` or `Update` — if the system running them is guarded by
`run_if`, they freeze. But if you forget to guard a timer-ticking system, timers advance
during pause. This is a footgun: you must individually guard every gameplay timer system.

### UI/input while paused

Input and UI systems that are NOT guarded by `run_if` continue to run. This is correct and
explicit.

### Implementation

Requires adding `run_if(not_paused)` to every gameplay system set, in every schedule.
Note: **SystemSets are per-schedule**. A `run_if` applied to a set in `Update` does NOT
propagate to the same set label in `FixedUpdate`. You must call `configure_sets` separately
for each schedule.

### Complexity: HIGH

- Must annotate every gameplay system with the condition
- Must handle FixedUpdate time accumulation catch-up on unpause
- Per-schedule SystemSet configuration required
- Footgun: missed systems tick during pause

### Gotchas

- **`configure_sets` is per-schedule** (confirmed from `schedule.rs` source). If you define
  a `GameplaySystems` set and call `configure_sets(GameplaySystems.run_if(not_paused))` in
  the `Update` schedule, that condition does NOT apply in `FixedUpdate`. You must repeat the
  call in `FixedUpdate` as well.
- `resource_exists` panics at startup if the resource is not initialized. Use
  `Option<Res<GamePaused>>` in the condition closure to avoid panics.
- Does not freeze `Time<Fixed>` overstep — time catch-up on resume.

### Verdict: NOT RECOMMENDED as primary mechanism

Too much boilerplate, too many footguns. Best used as a supplementary guard for individual
systems that need special behavior around pause (e.g., a system that must skip processing
during pause regardless of virtual time state).

---

## Approach 3: Independent Orthogonal `PauseState` (a second `States`)

### How it works

Define a second top-level state type that is entirely independent of `GameState`:

```rust
#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum PauseState {
    #[default]
    Running,
    Paused,
}

// Register alongside GameState:
app.init_state::<GameState>()
   .init_state::<PauseState>();
```

The docs confirm: "Multiple states can be defined for the same world, allowing you to
classify the state of the world across orthogonal dimensions."

Systems are gated with `in_state(PauseState::Running)`:

```rust
.add_systems(FixedUpdate,
    physics_system.run_if(in_state(PauseState::Running))
)
```

Toggle:

```rust
fn toggle_pause(
    current: Res<State<PauseState>>,
    mut next: ResMut<NextState<PauseState>>,
) {
    match current.get() {
        PauseState::Running => next.set(PauseState::Paused),
        PauseState::Paused => next.set(PauseState::Running),
    }
}
```

### Effect on FixedUpdate

Same limitation as Approach 2. `in_state(PauseState::Running)` prevents guarded systems
from running, but `FixedUpdate` still ticks and time still accumulates. On unpause, catch-up
applies. Does not freeze `Time<Fixed>`.

### Effect on timers

Same as Approach 2. Only guarded systems freeze. Unguarded timer systems continue.

### UI/input while paused

Works correctly — only gated systems skip.

### Complexity: MEDIUM

- Clean API: `in_state(PauseState::Running)` is readable
- No `SubStates` complexity
- Still requires per-schedule `configure_sets` or per-system annotation
- Still has FixedUpdate catch-up problem

### `SubStates` vs `ComputedStates` note

`SubStates` CANNOT be used here because `SubStates` requires a source state — it is not
independent. A truly orthogonal `PauseState` must use `States`, not `SubStates`.

`ComputedStates` is relevant if you want `PauseState` to be derived from multiple source
states (e.g., auto-pause when `GameState != Run`). Example:

```rust
impl ComputedStates for PauseState {
    type SourceStates = Option<GameState>;

    fn compute(sources: Option<GameState>) -> Option<Self> {
        match sources {
            Some(GameState::Run) => Some(PauseState::Running), // default: not paused
            _ => None, // PauseState inactive outside of Run
        }
    }
}
```

But this makes `PauseState::Paused` unreachable via `compute()` (since compute is called on
every state change) — you'd need a second independent state variable to track "user requested
pause." This gets complicated fast.

### Verdict: VIABLE but unnecessary given Approach 1

Adds clarity of explicit state transitions at the cost of FixedUpdate catch-up problems.
Combine with `Time<Virtual>::pause()` to get the best of both worlds (see Hybrid below).

---

## Approach 4: SystemSet with `run_if` Across Schedules

### What you asked

Can a `GameplaySystems` set be defined once, added to both `Update` and `FixedUpdate`, and
have a single `configure_sets(GameplaySystems.run_if(not_paused))` call that covers both?

### Answer: NO

**SystemSets are per-schedule.** Confirmed from Bevy's `schedule.rs` source:

> "Another caveat is that if `GameSystem::B` is placed in a different schedule than
> `GameSystem::A`, any ordering calls between them — whether using `.before`, `.after`, or
> `.chain` — will be silently ignored."

Each schedule maintains its own `ScheduleGraph` with independent system sets and conditions.
`configure_sets` calls operate on a single schedule instance.

To apply the same `run_if` to a set in both schedules, you must call `configure_sets` in
each schedule:

```rust
// WRONG — only applies to Update:
app.configure_sets(Update, GameplaySystems.run_if(not_paused));

// CORRECT — must configure in each schedule separately:
app.configure_sets(Update, GameplaySystems.run_if(not_paused));
app.configure_sets(FixedUpdate, GameplaySystems.run_if(not_paused));
```

The same set *label* can appear in both schedules — the label is just a type, and
`configure_sets` applies conditions to that label within the specific schedule being
configured.

### Verdict: POSSIBLE but requires dual registration

Not a single-point-of-control solution. Still has the FixedUpdate catch-up problem.

---

## Approach 5: Built-in Bevy Pause Schedule

### What you asked

Does Bevy 0.18 have any built-in "Paused" schedule?

### Answer: NO

There is no built-in `Paused` schedule in Bevy 0.18.1. The built-in schedules are:

- `PreStartup`, `Startup`, `PostStartup`
- `First`, `PreUpdate`, `Update`, `PostUpdate`, `Last`
- `FixedFirst`, `FixedPreUpdate`, `FixedUpdate`, `FixedPostUpdate`, `FixedLast`
- `StateTransition`
- `OnEnter(S)`, `OnExit(S)`, `OnTransition { from, to }`

None of these are a pause schedule. The closest is using `OnEnter(PauseState::Paused)` and
`OnExit(PauseState::Paused)` for pause/resume side effects (show/hide UI, etc.), but these
are transition schedules, not ongoing schedules.

---

## Recommended Approach: `Time<Virtual>::pause()` + Optional State for Menu

### Summary

Use `Time<Virtual>::pause()` as the primary mechanism:

1. **Physics (FixedUpdate) freezes automatically** — zero delta, no overstep, no catch-up.
2. **Game timers freeze automatically** — they tick with virtual time.
3. **UI and input continue** — they use real time or are schedule-level, not time-gated.
4. **Works at any RunState** — no dependency on state machine hierarchy.
5. **Zero boilerplate** — no per-system run_if annotations needed for freeze behavior.

For pause menu visibility, optionally pair with an `IsPaused` marker (ZST resource or
a simple `ComputedStates` derived from virtual time state) to drive `OnEnter`/`OnExit`
schedules for showing/hiding pause menu UI.

### Implementation sketch

```rust
// In run plugin or shared:
fn toggle_pause(
    actions: Res<InputActions>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    if !actions.active(GameAction::TogglePause) { return; }
    if virtual_time.is_paused() {
        virtual_time.unpause();
    } else {
        virtual_time.pause();
    }
}

// Run condition for systems that should be skipped when paused
// (only needed for Update systems that don't use Time<Virtual>):
fn not_paused(vt: Res<Time<Virtual>>) -> bool {
    !vt.is_paused()
}

// Scheduling toggle_pause:
// - Run during any RunState (or scoped to GameState::Run)
// - Does NOT need to be limited to NodeState::Playing — user can pause during ChipSelect etc.
app.add_systems(
    Update,
    toggle_pause.run_if(in_state(GameState::Run)),
);
```

No `PlayingState` needed. No `SubStates` for pause. No `configure_sets` for pause.

### What replaces the current `PlayingState::Active` guard

Currently: `run_if(in_state(PlayingState::Active))`

After migration: `run_if(in_state(NodeState::Playing))`

Gameplay systems (physics, bolt movement, collision detection) only run during
`NodeState::Playing` — they are already gated by state, not by pause state. When the user
pauses, `Time<Virtual>::pause()` stops FixedUpdate from running, so those systems naturally
freeze even without an explicit `not_paused` condition.

The `run_if(in_state(NodeState::Playing))` gate handles:
- Non-node states (ChipSelect, RunEnd, Setup, Teardown)
- The NodeState::Setup and NodeState::Teardown phases

The `Time<Virtual>::pause()` handles:
- User-requested pause during any RunState

This is a clean separation of concerns.

---

## Key Facts Summary (Bevy 0.18.1)

| Question | Answer |
|---|---|
| Does `Time<Virtual>::pause()` exist? | Yes — `pause()`, `unpause()`, `is_paused()`, `was_paused()` |
| Does it freeze FixedUpdate? | Yes — zero delta stops overstep accumulation, loop exits |
| Does it freeze game timers? | Yes — timers ticking with virtual time stop |
| Does it affect UI/input? | No — input polling and `Time<Real>` systems continue |
| Can two independent States coexist? | Yes — "orthogonal dimensions" is explicitly supported |
| Can SubStates be independent (no source)? | No — SubStates always requires a source state |
| Is configure_sets per-schedule? | Yes — must call separately for Update and FixedUpdate |
| Is there a built-in Pause schedule? | No |
| Does `in_state()` work in FixedUpdate? | Yes — it is a SystemCondition, works in any schedule |

---

## Verified API Signatures

```rust
// Time<Virtual> pause methods (bevy::time::Virtual):
impl Time<Virtual> {
    pub fn pause(&mut self);
    pub fn unpause(&mut self);
    pub fn is_paused(&self) -> bool;
    pub fn was_paused(&self) -> bool;
    pub fn effective_speed(&self) -> f32;  // returns 0.0 when paused
}

// State conditions (bevy::state::condition):
pub fn in_state<S: States>(state: S) -> impl FnMut(Option<Res<'_, State<S>>>) + Clone;

// AppExtStates (bevy::state::app):
fn init_state<S: States + Default>(&mut self) -> &mut Self;
fn add_sub_state<S: SubStates>(&mut self) -> &mut Self;
fn add_computed_state<S: ComputedStates>(&mut self) -> &mut Self;
fn insert_state<S: States>(&mut self, state: S) -> &mut Self;

// SubStates trait:
pub trait SubStates: States {
    type SourceStates: StateSet;
    fn should_exist(sources: Self::SourceStates) -> Option<Self>;
}

// ComputedStates trait:
pub trait ComputedStates: 'static + Send + Sync + Clone + PartialEq + Eq + Hash + Debug {
    type SourceStates;
    fn compute(sources: Self::SourceStates) -> Option<Self>;
}
```

Note: `ComputedStates` does NOT require `Default` (unlike `States` / `SubStates`).
`SubStates` requires `Default` to specify the initial state when the source condition is met.
