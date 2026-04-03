# Transition Lifecycle

Full lifecycle from "game says change state" to "landed in new state."

## Communication Model

Two directions, two mechanisms:

- **Resources** (`StartingTransition<T>`, `RunningTransition<T>`, `EndingTransition<T>`) flow **crate → effect systems**. Persistent — gate when systems run via run conditions. Carry effect params (color, easing, etc.). `RunningTransition<T>` stays around so the `run` system fires every frame.

- **Messages** (`TransitionReady`, `TransitionRunComplete`, `TransitionOver`) flow **effect systems → crate**. One-shot completion signals — "I'm done with this phase, advance me."

Resources = persistent activation + data. Messages = one-shot completion signal.

## Trigger

Two paths converge at the same dispatch point:

```
Path A: Message-triggered route
┌──────────────────────────────┐
│ Game sends ChangeState<S>    │
└──────────────┬───────────────┘
               │
               ▼
Routing system fires (gated by on_message::<ChangeState<S>>)
               │
               ▼
Look up route for current state in RoutingTable<S>
Resolve destination: .to(S) or .to_dynamic(fn(&World) → S)
Resolve transition: none, .with_transition(T), or .with_dynamic_transition(fn)
               │
               ▼
          DISPATCH

Path B: Condition-triggered route
┌──────────────────────────────────────────┐
│ Polling system runs every frame          │
│ Iterates condition-triggered routes only │
│ Calls when_fn(&World) for each           │
└──────────────┬───────────────────────────┘
               │ condition returns true
               ▼
Resolve destination + transition (same as Path A)
               │
               ▼
          DISPATCH
```

## Dispatch — branches on TransitionType

### BARE (no transition)

```
next_state.set(destination)
        │
        ▼
Send StateChanged<S> { from, to }
        │
        ▼
Done. No pause, no overlay, no TransitionStart/End messages.
```

### OUT

```
Pause Time<Virtual>
        │
        ▼
Send TransitionStart<S> { from, to }
        │
        ▼
Insert StartingTransition<T>
        │
        ▼
T start system runs (Time<Real>)
├─ Spawns overlay entity (GlobalZIndex(i32::MAX - 1))
└─ Sends TransitionReady
        │
        ▼
Remove Starting, insert RunningTransition<T>
        │
        ▼
T run system runs each frame (Time<Real>)
├─ Samples easing curve → animates overlay (transparent → opaque)
└─ When progress >= 1.0 → sends TransitionRunComplete
        │
        ▼
Remove Running, insert EndingTransition<T>
        │
        ▼
T end system runs
├─ Despawns overlay
└─ Sends TransitionOver
        │
        ▼
Remove EndingTransition
        │
        ▼
┌─────────── STATE CHANGE ───────────┐
│ next_state.set(destination)        │
│ Bevy fires OnExit / OnEnter        │
│ New state's systems run (behind    │
│ the cover — virtual time paused)   │
└────────────────────────────────────┘
        │
        ▼
Send StateChanged<S> { from, to }
        │
        ▼
Send TransitionEnd<S> { from, to }
        │
        ▼
Screen stays covered. Virtual time stays paused.
Waiting for a matching IN to reveal and unpause.
```

### IN

```
(Screen is already covered from a prior Out. Virtual time already paused.)
        │
        ▼
Send TransitionStart<S> { from, to }
        │
        ▼
Insert StartingTransition<T>
        │
        ▼
T start system runs (Time<Real>)
├─ Spawns overlay at full opacity
└─ Sends TransitionReady
        │
        ▼
Remove Starting, insert RunningTransition<T>
        │
        ▼
T run system runs each frame (Time<Real>)
├─ Samples easing curve → animates overlay (opaque → transparent)
└─ When progress >= 1.0 → sends TransitionRunComplete
        │
        ▼
Remove Running, insert EndingTransition<T>
        │
        ▼
T end system runs
├─ Despawns overlay entity
└─ Sends TransitionOver
        │
        ▼
Remove EndingTransition
        │
        ▼
Unpause Time<Virtual>
        │
        ▼
Send TransitionEnd<S> { from, to }
        │
        ▼
Done. Game is in the new state, virtual time running.
```

### INOUT (Out → handoff → change → In)

```
Pause Time<Virtual>
        │
        ▼
Send TransitionStart<S> { from, to }
        │
        ▼
┌─── OUT PHASE (duration / 2) ───────────────────────────┐
│                                                        │
│  Insert StartingTransition<OutEffect>                  │
│         │                                              │
│         ▼                                              │
│  out start → spawns overlay (transparent)              │
│         └─ Sends TransitionReady                       │
│         │                                              │
│         ▼                                              │
│  Remove Starting, insert RunningTransition<OutEffect>  │
│         │                                              │
│         ▼                                              │
│  out run each frame (Time<Real>)                       │
│         │  animate cover (transparent → opaque)        │
│         └─ TransitionRunComplete when done             │
│         │                                              │
│         ▼                                              │
│  Remove Running, insert EndingTransition<OutEffect>    │
│    .crate_owns_overlay = true                          │
│         │                                              │
│         ▼                                              │
│  out end → signal done, do NOT despawn overlay         │
│         └─ Sends TransitionOver                        │
│                                                        │
└────────┬───────────────────────────────────────────────┘
         │
         ▼
Screen is covered (Out overlay at full opacity)
Crate despawns Out overlay
         │
         ▼
┌─────────── STATE CHANGE ───────────┐
│ next_state.set(destination)        │
│ Bevy fires OnExit / OnEnter        │
│ New state loads behind the cover   │
└────────────────────────────────────┘
         │
         ▼
Send StateChanged<S> { from, to }
         │
         ▼
┌─── IN PHASE (duration / 2) ────────────────────────────┐
│                                                        │
│  Insert StartingTransition<InEffect>                   │
│         │                                              │
│         ▼                                              │
│  in start → spawns own overlay (full opacity)          │
│         └─ Sends TransitionReady                       │
│         │                                              │
│         ▼                                              │
│  Remove Starting, insert RunningTransition<InEffect>   │
│         │                                              │
│         ▼                                              │
│  in run each frame (Time<Real>)                        │
│         │  animate reveal (opaque → transparent)       │
│         └─ TransitionRunComplete when done             │
│         │                                              │
│         ▼                                              │
│  Remove Running, insert EndingTransition<InEffect>     │
│    .crate_owns_overlay = true                          │
│         │                                              │
│         ▼                                              │
│  in end → signal done, do NOT despawn overlay          │
│         └─ Sends TransitionOver                        │
│                                                        │
└────────┬───────────────────────────────────────────────┘
         │
         ▼
Crate despawns In overlay
         │
         ▼
Unpause Time<Virtual>
         │
         ▼
Send TransitionEnd<S> { from, to }
         │
         ▼
Done. Game is in the new state, virtual time running.
```

### ONESHOT (e.g. Slide)

```
Pause Time<Virtual>
        │
        ▼
Send TransitionStart<S> { from, to }
        │
        ▼
┌─────────── STATE CHANGE ───────────┐
│ next_state.set(destination)        │
│ Both old and new content coexist   │
│ (game's responsibility)            │
└────────────────────────────────────┘
        │
        ▼
Send StateChanged<S> { from, to }
        │
        ▼
Insert StartingTransition<T>
        │
        ▼
T start system runs (Time<Real>)
├─ Records camera position, calculates target
└─ Sends TransitionReady
        │
        ▼
Remove Starting, insert RunningTransition<T>
        │
        ▼
T run system runs each frame (Time<Real>)
├─ Samples easing curve → lerps camera position
└─ When progress >= 1.0 → sends TransitionRunComplete
        │
        ▼
Remove Running, insert EndingTransition<T>
        │
        ▼
T end system runs
└─ Sends TransitionOver (camera stays at target)
        │
        ▼
Remove EndingTransition
        │
        ▼
Unpause Time<Virtual>
        │
        ▼
Send TransitionEnd<S> { from, to }
        │
        ▼
Done. Game is in the new state, virtual time running.
```

## Deferred ChangeState

If a `ChangeState<S>` arrives while a transition is active:
- Queued, not processed
- Processed after current transition completes
- Prevents cascading state changes mid-animation

This is how child states work: `NodeState::Loading` completes, sends `ChangeState<NodeState>`, but a parent OutIn transition is still playing → deferred until the In phase finishes and `TransitionEnd` fires.
