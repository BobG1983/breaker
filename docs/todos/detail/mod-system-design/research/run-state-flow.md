# Research: Run State Flow and Tier/Node Progression

## Scope

End-to-end trace of the run state machine, node completion flow, chip select
entry/exit, and tier/node tracking resources. This analysis supports deciding
where to insert a new `HazardSelect` state for the Protocol and Hazard system.

---

## 1. State Machine Hierarchy

The game uses Bevy 0.18 `SubStates` for a four-level hierarchy:

```
AppState (top level)
  Loading → Game → Teardown

  GameState (SubState of AppState::Game)
    Loading → Menu → Run → Teardown

    MenuState (SubState of GameState::Menu)
      Loading → Main → StartGame → Options → Meta → Teardown

    RunState (SubState of GameState::Run)
      Loading → Setup → Node → ChipSelect → RunEnd → Teardown

      NodeState (SubState of RunState::Node)
        Loading → AnimateIn → Playing → AnimateOut → Teardown

      ChipSelectState (SubState of RunState::ChipSelect)
        Loading → AnimateIn → Selecting → AnimateOut → Teardown

      RunEndState (SubState of RunState::RunEnd)
        Loading → AnimateIn → Active → AnimateOut → Teardown
```

All state types are declared in `breaker-game/src/state/types/`.

---

## 2. Transition Mechanism: `rantzsoft_stateflow`

Transitions are **not** driven by raw `NextState<S>` calls in game systems.
Instead, the project uses a custom crate (`rantzsoft_stateflow`) with:

- **`RoutingTable<S>`** — a resource holding one `Route` per source variant.
  Each `Route` declares: `from`, `to` (static or dynamic closure), an optional
  `when` condition closure, and an optional `transition` effect (fade, iris, etc.).
- **`dispatch_message_routes<S>`** — exclusive system that fires when a
  `ChangeState<S>` message is received. Looks up the current state in
  `RoutingTable<S>`, resolves the destination, and either calls
  `NextState<S>::set()` directly or begins a `TransitionEffect`.
- **`dispatch_condition_routes<S>`** — exclusive system that runs every frame
  in `Update`, polling `when` conditions. Used for parent-watches-child routing
  (e.g., `GameState::Run` → `GameState::Menu` when `RunState` reaches
  `Teardown`).
- **`ChangeState<S>`** — a Bevy message type. Systems that want to trigger a
  transition `write(ChangeState::new())` and the dispatch system handles the
  rest.

Routes are registered once at startup in
`breaker-game/src/state/plugin/system.rs::register_routing()`.

**Key property**: all routes are one-to-one per source variant. There is
exactly one registered route per state variant. Dynamic routing (`to_dynamic`)
lets a single route fan out to different destinations based on world state.

---

## 3. Full Routing Table (as registered)

### AppState
| From | To | Trigger |
|------|----|---------|
| Game | Teardown | condition: `GameState == Teardown` |

### GameState
| From | To | Trigger |
|------|----|---------|
| Loading | Menu | condition: always (with FadeOut transition) |
| Menu | dynamic (Run or Teardown) | condition: `MenuState == Teardown`; destination depends on `MainMenuSelection` |
| Run | Menu | condition: `RunState == Teardown` (with FadeOut) |

### MenuState
| From | To | Trigger |
|------|----|---------|
| Loading | Main | condition: always (with FadeIn) |
| Main | dynamic (StartGame/Options/Teardown) | message-triggered `ChangeState<MenuState>` |
| StartGame | Teardown | message-triggered `ChangeState<MenuState>` |

### RunState
| From | To | Trigger |
|------|----|---------|
| Loading | Setup | condition: always (pass-through) |
| Setup | Node | condition: always (with FadeIn) |
| Node | dynamic (ChipSelect/RunEnd/Teardown) | condition: `NodeState == Teardown`; destination resolved by `resolve_node_next_state()` |
| ChipSelect | Node | condition: `ChipSelectState == Teardown` (with FadeOut) |
| RunEnd | Teardown | condition: `RunEndState == Teardown` |

`resolve_node_next_state()` in `state/plugin/system.rs`:
```
NodeResult::InProgress  → RunState::ChipSelect
NodeResult::Quit        → RunState::Teardown
_ (Won/TimerExpired/LivesDepleted) → RunState::RunEnd
```

### NodeState
| From | To | Trigger |
|------|----|---------|
| Loading | AnimateIn | message-triggered (`check_spawn_complete` sends `ChangeState`) |
| AnimateIn | Playing | message-triggered (`all_animate_in_complete` sends `ChangeState`) |
| Playing | AnimateOut | message-triggered (`handle_node_cleared`, `handle_timer_expired`, or `handle_run_lost`) |
| AnimateOut | Teardown | condition: always (pass-through) |

### ChipSelectState
| From | To | Trigger |
|------|----|---------|
| Loading | AnimateIn | condition: always (pass-through) |
| AnimateIn | Selecting | condition: always (pass-through) |
| Selecting | AnimateOut | message-triggered (`handle_chip_input` or `tick_chip_timer`) |
| AnimateOut | Teardown | condition: always (pass-through) |

### RunEndState
| From | To | Trigger |
|------|----|---------|
| Loading | AnimateIn | condition: always (pass-through) |
| AnimateIn | Active | condition: always (pass-through) |
| Active | AnimateOut | message-triggered (`handle_run_end_input`) |
| AnimateOut | Teardown | condition: always (pass-through) |

---

## 4. Node Completion Flow (Full Chain)

**Trigger**: A cell with `RequiredToClear` is destroyed.

1. `handle_cell_hit` (cells domain, FixedUpdate) destroys the cell and sends
   `CellDestroyedAt { was_required_to_clear: true }`.

2. `track_node_completion` (NodePlugin, FixedUpdate, `in_set(NodeSystems::TrackCompletion)`,
   `run_if(NodeState::Playing)`) reads `CellDestroyedAt`, decrements
   `ClearRemainingCount`. When `remaining == 0` (first time), sends `NodeCleared`.

3. `handle_node_cleared` (RunPlugin, FixedUpdate, after `NodeSystems::TrackCompletion`,
   `run_if(NodeState::Playing)`) reads `NodeCleared`:
   - Sets `NodeOutcome.transition_queued = true`
   - If `node_index >= final_node_index`: sets `NodeOutcome.result = NodeResult::Won`
   - Otherwise: `NodeOutcome.result` stays `NodeResult::InProgress`
   - Sends `ChangeState<NodeState>`

4. `dispatch_message_routes<NodeState>` (Update, from `RantzStateflowPlugin`)
   reads `ChangeState<NodeState>`, looks up current state `NodeState::Playing` in
   `RoutingTable<NodeState>`, finds route → `NodeState::AnimateOut`.
   Applies transition or sets `NextState<NodeState>::set(AnimateOut)`.
   Next frame: `NodeState` becomes `AnimateOut`.

5. `dispatch_condition_routes<NodeState>` picks up the `AnimateOut → Teardown`
   always-true condition. `NodeState` becomes `Teardown`.

6. `OnEnter(NodeState::Teardown)` runs `cleanup_on_exit::<NodeState>` —
   despawns all entities tagged with `CleanupOnExit<NodeState>`.

7. `dispatch_condition_routes<RunState>` now sees `NodeState == Teardown`,
   which satisfies the `RunState::Node` route's `when` condition.
   Calls `resolve_node_next_state()`:
   - `InProgress` → `RunState::ChipSelect`
   - `Won` → `RunState::RunEnd`
   - `Quit` → `RunState::Teardown`
   Transition begins (FadeOut, 0.6s); after it completes,
   `RunState` becomes `ChipSelect` (or `RunEnd`).

8. `OnExit(RunState::Node)` runs `hide_gameplay_entities` — hides Breaker and
   Bolt entities.

**Tie-frame rule**: If the timer fires on the same tick as the last cell is
destroyed, `handle_timer_expired` checks `run_state.transition_queued` and
yields to `handle_node_cleared`. Clear beats loss.

---

## 5. Chip Select Flow

**Entry**: `RunState` transitions to `ChipSelect`.

`ChipSelectState` substates automatically pass through `Loading → AnimateIn →
Selecting` on consecutive frames (all condition-triggered, all `when(|_| true)`).

On entering `ChipSelectState::Selecting`:
1. `generate_chip_offerings` runs (OnEnter, before `spawn_chip_select`).
   Reads `ActiveNodeLayout.pool` to detect Boss nodes and offer evolutions.
   Inserts `ChipOffers` resource.
2. `ApplyDeferred` flushes commands.
3. `spawn_chip_select` spawns the UI (card layout, timer display).

During `ChipSelectState::Selecting` (Update):
- `handle_chip_input` reads keyboard input:
  - Left/Right: navigate card selection
  - Confirm: sends `ChipSelected { name }` message, applies decay to non-selected
    chips, sends `ChangeState<ChipSelectState>`
- `tick_chip_timer`: decrements timer; on expiry sends `ChangeState<ChipSelectState>`
  (auto-advance, no chip selected, applies decay to all normal offerings)
- `track_chips_collected`, `detect_first_evolution`, `snapshot_node_highlights`
  also run

**Exit**: `ChangeState<ChipSelectState>` → `Selecting → AnimateOut → Teardown`
(pass-through chain). `OnExit(ChipSelectState::Selecting)` cleans up UI.

Then `dispatch_condition_routes<RunState>` detects `ChipSelectState == Teardown`
and transitions `RunState::ChipSelect → RunState::Node` (FadeOut, 0.6s).

On entering `RunState::Node`:
- `advance_node` increments `NodeOutcome.node_index` and resets
  `transition_queued`
- `show_gameplay_entities` makes Breaker and Bolt visible again

Then the `NodeState` sub-machine spins up again from `Loading`.

---

## 6. Tier and Node Tracking Resources

### `NodeOutcome` (`resources.rs`)
The primary runtime tracker. Lives for the entire run.
- `node_index: u32` — zero-indexed position in the run sequence, initialized to
  0 at run start. Incremented by `advance_node` on `OnEnter(RunState::Node)`.
  So the _first_ node played is index 0 (before `advance_node` runs — wait, see
  below for the off-by-one detail).
- `result: NodeResult` — `InProgress | Won | TimerExpired | LivesDepleted | Quit`
- `transition_queued: bool` — tie-frame guard

**Off-by-one detail**: `reset_run_state` sets `node_index = 0`. `setup_run`
runs on `OnEnter(NodeState::Loading)` (first node only — guarded by
`existing_breakers.is_empty()`). `advance_node` runs on `OnEnter(RunState::Node)`,
which fires when _returning_ from ChipSelect. So:
- Node 0 is played with `node_index == 0`
- `advance_node` increments to 1 after completing node 0
- Node 1 is played with `node_index == 1`

The current node being played is always `NodeOutcome.node_index`.

### `NodeSequence` (`resources.rs`)
Inserted by `generate_node_sequence_system` on `OnExit(MenuState::Main)`.
Contains a `Vec<NodeAssignment>` ordered from first node to last. Each
`NodeAssignment` has:
- `tier_index: u32` — which difficulty tier (0-indexed) this node belongs to
- `node_type: NodeType` — `Passive | Active | Boss`
- `hp_mult: f32`, `timer_mult: f32` — scaling factors

To find the current tier, read:
```rust
node_sequence.assignments[node_outcome.node_index as usize].tier_index
```

### `DifficultyCurve` (`resources.rs`)
Loaded from `assets/config/defaults.difficulty.ron`. Contains a `Vec<TierDefinition>`
(currently 5 tiers, indices 0–4). Each `TierDefinition` has node counts,
ratios, hp/timer multipliers.

**Current tier count**: 5 tiers (0–4), each with 4–6 nodes + 1 boss = 25–35
total nodes per run. Note: the design docs mention tier 9+, but the current
defaults only configure 5 tiers (0–4).

### `ActiveNodeLayout` (`resources.rs`)
Inserted by `set_active_layout` on `OnEnter(NodeState::Loading)`. Contains the
`NodeLayout` currently being played. The layout's `.pool` field
(`NodePool::Standard | NodePool::Boss`) comes from the node's RON file and is
independent of the `NodeAssignment.node_type`.

---

## 7. Where `NodeResult::InProgress` vs `Won` Is Decided

`handle_node_cleared` reads `NodeSequence.assignments.len()` as the total node
count. If `node_index >= final_index`, `NodeResult::Won` is set; otherwise,
`NodeResult::InProgress`. The `resolve_node_next_state` function then reads this
to route to `ChipSelect` or `RunEnd`.

---

## 8. Chip Select Is Always Offered After Non-Final Nodes

Every non-final node that ends with `NodeResult::InProgress` routes to
`RunState::ChipSelect`. The chip select screen appears regardless of tier. There
is no existing conditional gating based on tier number.

---

## 9. Where to Insert `HazardSelect`

### Design requirement
A `HazardSelect` screen should appear at tier 9+ AFTER the chip/protocol
selection screen. (The current defaults only have 5 tiers; tier indexing would
need to be expanded or the threshold adjusted.)

### Insertion point analysis

The canonical insertion point is between `RunState::ChipSelect` and
`RunState::Node`:

```
Current:    Node → [NodeState::Teardown] → ChipSelect → [ChipSelectState::Teardown] → Node

Proposed:   Node → [NodeState::Teardown] → ChipSelect → [ChipSelectState::Teardown]
                                                                              ↓
                                              (tier >= threshold?)
                                             /                    \
                                      HazardSelect              Node
                                [HazardSelectState::Teardown]
                                             ↓
                                           Node
```

### Concrete steps to wire this

**Step 1: Add `RunState::HazardSelect` variant**

Add `HazardSelect` to `RunState` between `ChipSelect` and `RunEnd`:
```rust
pub enum RunState {
    Loading,
    Setup,
    Node,
    ChipSelect,
    HazardSelect,  // NEW
    RunEnd,
    Teardown,
}
```

**Step 2: Add `HazardSelectState` substate (parallel to `ChipSelectState`)**

```rust
#[derive(SubStates, Default, ...)]
#[source(RunState = RunState::HazardSelect)]
pub enum HazardSelectState {
    #[default] Loading,
    AnimateIn,
    Selecting,
    AnimateOut,
    Teardown,
}
```

**Step 3: Modify the `RunState::ChipSelect` route**

Currently:
```rust
Route::from(RunState::ChipSelect)
    .to(RunState::Node)
    .when(|world| ChipSelectState == Teardown)
```

Change the destination to be dynamic:
```rust
Route::from(RunState::ChipSelect)
    .to_dynamic(resolve_post_chip_state)  // NEW
    .when(|world| ChipSelectState == Teardown)
```

Where `resolve_post_chip_state` reads `NodeSequence` and `NodeOutcome` to
check tier:
```rust
fn resolve_post_chip_state(world: &World) -> RunState {
    let tier = current_tier(world);  // reads NodeSequence + NodeOutcome
    if tier >= HAZARD_THRESHOLD {
        RunState::HazardSelect
    } else {
        RunState::Node
    }
}
```

**Step 4: Add `RunState::HazardSelect` route**
```rust
Route::from(RunState::HazardSelect)
    .to(RunState::Node)
    .when(|world| HazardSelectState == Teardown)
```

**Step 5: Register `HazardSelectState` in `StatePlugin`**

In `register_routing()` add `register_hazard_select_routes(app)` following the
same pattern as `register_chip_select_routes`.

In `RantzStateflowPlugin::new()` call `.register_state::<HazardSelectState>()`.

**Step 6: Register `HazardSelectState` in `init_state` chain**

In `StatePlugin::build()`, add `.add_sub_state::<HazardSelectState>()`.

**Step 7: Add cleanup**

In `register_cleanup()`:
```rust
app.add_systems(OnEnter(HazardSelectState::Teardown), cleanup_on_exit::<HazardSelectState>);
```

### Alternative: reuse `ChipSelectState` as parent

Rather than a new `RunState` variant, the HazardSelect could be a sub-screen
within `RunState::ChipSelect`. However, this complicates the existing
`ChipSelectState` substate machine and the routing table (only one route per
variant). The separate `RunState::HazardSelect` variant is cleaner and follows
the existing pattern exactly.

---

## 10. Key Implementation Notes

### `NodeOutcome.node_index` vs tier

There is **no per-tier counter resource** — tier is derived at query time:
```rust
// In a system or closure:
let tier_index = node_sequence.assignments[node_outcome.node_index as usize].tier_index;
```

This means `resolve_post_chip_state` needs to read both `NodeSequence` and
`NodeOutcome` from the world. Both are always present during `RunState::ChipSelect`.

**Caution**: at `ChipSelectState::Teardown`, `advance_node` has NOT yet run —
`node_index` still points to the node that just completed. The next node will
have `node_index + 1`. The hazard threshold check should use the completed
node's tier, not the upcoming node's tier. If the intent is "show hazards
after completing a node in tier 9+", reading `node_index` directly is correct.
If the intent is "show hazards before starting a node in tier 9+", read
`node_index + 1`.

### Pass-through states and transition effects

`HazardSelectState::Loading → AnimateIn → Selecting` should follow the same
always-true condition pattern as `ChipSelectState`. The `Selecting → AnimateOut`
step is message-triggered. This makes the animate-in/out wiring symmetric
and ready for future visual transitions.

### `Time<Virtual>` pausing hazard

As documented in agent memory: `Out` transitions leave `Time<Virtual>` paused,
and `FixedUpdate` systems hang if reached while paused. The existing `FadeOut`
transition on `RunState::ChipSelect → RunState::Node` already navigates this
correctly. The new `HazardSelect` route should use the same `FadeOut` transition
pattern that the rest of the run-to-run transitions use.

---

## Key Files

- `breaker-game/src/state/types/run_state.rs` — `RunState` enum, add `HazardSelect` here
- `breaker-game/src/state/types/chip_select_state.rs` — reference pattern for new `HazardSelectState`
- `breaker-game/src/state/plugin/system.rs` — ALL route registrations live here; `resolve_node_next_state` and `resolve_post_chip_state` go here
- `breaker-game/src/state/run/chip_select/plugin.rs` — reference pattern for new `HazardSelectPlugin`
- `breaker-game/src/state/run/chip_select/systems/handle_chip_input.rs` — reference for message-triggered `ChangeState<ChipSelectState>` dispatch
- `breaker-game/src/state/run/resources/definitions.rs` — `NodeOutcome.node_index` and `NodeSequence` with `tier_index` per assignment
- `breaker-game/src/state/run/loading/systems/generate_node_sequence/system.rs` — how tier assignments are generated from `DifficultyCurve`
- `breaker-game/assets/config/defaults.difficulty.ron` — current 5-tier config (tiers 0–4); tier threshold for HazardSelect must be set against this scale
- `breaker-game/src/state/plugin/mod.rs` — `StatePlugin` that adds all sub-states and calls `register_routing`
