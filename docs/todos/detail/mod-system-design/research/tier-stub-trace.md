# Behavior Trace: Node Sequence — Tier Stub

## Trigger

The question to answer: how does the game track which node the player is on, how does it
advance through nodes, and where does a `current_tier: u32` stub need to live to be queryable
by the protocol/hazard system?

---

## System Chain

### 1. `generate_node_sequence_system` (RunPlugin, `OnExit(MenuState::Main)`)

Reads `DifficultyCurve` (tiers vec from RON) and `GameRng`, produces a flat `NodeSequence`
resource via `commands.insert_resource`. The sequence is a `Vec<NodeAssignment>` — one entry
per node the player will visit in the entire run.

Each `NodeAssignment` already carries:

```rust
pub struct NodeAssignment {
    pub node_type: NodeType,    // Passive | Active | Boss
    pub tier_index: u32,        // 0-indexed tier this node belongs to
    pub hp_mult: f32,
    pub timer_mult: f32,
}
```

`tier_index` is baked in at generation time. Nodes 0..N are assigned tier 0, the boss is tier 0,
next batch is tier 1, etc.

**File:** `breaker-game/src/state/run/loading/systems/generate_node_sequence/system.rs`

---

### 2. `reset_run_state` (RunPlugin, `OnExit(MenuState::Main)`)

Resets `NodeOutcome` to default (which sets `node_index = 0`). Runs before
`generate_node_sequence_system`.

**File:** `breaker-game/src/state/run/loading/systems/reset_run_state.rs`

---

### 3. `advance_node` (RunPlugin, `OnEnter(RunState::Node)`)

Increments `NodeOutcome.node_index` by 1 and clears `transition_queued`. This is the sole
write site for `node_index` after initialization. The index starts at 0 before the first node
enter, so entering `RunState::Node` the first time increments it to 1.

**Wait** — this has an off-by-one to be aware of:

- On `OnExit(MenuState::Main)`: `reset_run_state` sets `node_index = 0`.
- First `OnEnter(RunState::Node)`: `advance_node` sets `node_index = 1`.
- But `set_active_layout` and `init_node_timer` read `node_index` to index into the sequence.

So at the time `set_active_layout` runs (`OnEnter(NodeState::Loading)`), `node_index` is
already 1 for the first node. This is the live convention — the sequence is 1-indexed as
seen from these systems.

Actually — need to check ordering. `advance_node` runs `OnEnter(RunState::Node)`, and
`NodeState::Loading` is the default sub-state of `RunState::Node`, so `OnEnter(NodeState::Loading)`
also fires at the same time. Bevy runs `OnEnter(RunState::Node)` before sub-state enters,
so `advance_node` increments first, then `OnEnter(NodeState::Loading)` systems see the
already-incremented value.

**But `setup_run` also runs `OnEnter(NodeState::Loading)` for node 0** and guards with:

```rust
let serving = run_state.node_index == 0;
```

This guard fires correctly on the first node because at the moment `setup_run` checks, the
first `advance_node` hasn't run yet (the run goes `Loading → Setup → Node`). On `RunState::Setup`,
there is no `advance_node`. `advance_node` only runs on `OnEnter(RunState::Node)`.

So the actual flow for the first node:
1. `RunState::Loading` → `reset_run_state` → `node_index = 0`
2. `RunState::Setup` → `setup_run` runs — sees `node_index == 0`, spawns serving bolt
3. `RunState::Node` → `advance_node` → `node_index = 1`
4. `NodeState::Loading` → `set_active_layout` indexes at `1 % registry.len()` → skips layout 0?

Wait — this would mean the first node always plays layout at index 1. Let me re-check the test
in `set_active_layout`:

```rust
fn test_app(node_index: u32, layouts: Vec<NodeLayout>) -> App {
    app.insert_resource(NodeOutcome { node_index, ..default() })
    // ...
    fn index_zero_selects_first_layout() {
        let mut app = test_app(0, layouts);  // node_index=0 → "first"
```

This test confirms that `node_index == 0` → layout 0. But in the real flow, the first node
sees `node_index == 1` after `advance_node`. This means the first layout (index 0) is never
played — OR `advance_node` is registered on `OnEnter(RunState::Node)` but does NOT fire for
the very first `RunState::Node` entry, because the run enters `RunState::Node` via the
`RunState::Setup → RunState::Node` condition-triggered route (not a re-entry after ChipSelect).

Actually looking at the plugin registration again:

```rust
.add_systems(OnEnter(RunState::Node), (advance_node, show_gameplay_entities)),
```

`OnEnter(RunState::Node)` fires every time `RunState::Node` is entered, including the very
first time from `RunState::Setup`. So on the first node, `node_index` = 0 → 1 before
`set_active_layout`. The first layout played is at index 1 (i.e., the second layout in
the registry). This is either intentional or there is an off-by-one that has not been caught
because tests use `node_index` injected directly.

**This is a pre-existing design choice, not a bug to fix here.** The tier-stub work should
not touch this.

**File:** `breaker-game/src/state/run/systems/advance_node.rs`

---

### 4. `set_active_layout` (NodePlugin, `OnEnter(NodeState::Loading)`)

Reads `NodeOutcome.node_index`, picks the layout at `index % registry.len()`. Writes
`ActiveNodeLayout` resource via `commands.insert_resource`.

**File:** `breaker-game/src/state/run/node/systems/set_active_layout.rs`

---

### 5. `spawn_cells_from_layout` / `init_node_timer` (NodePlugin, `OnEnter(NodeState::Loading)`)

Both read `NodeOutcome.node_index` and `NodeSequence` to look up
`assignments[node_index]` for `hp_mult` and `timer_mult`. The `tier_index` field on that
`NodeAssignment` is available at this point but is not currently read.

**File:** `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/system.rs`
**File:** `breaker-game/src/state/run/node/systems/init_node_timer.rs`

---

### 6. `track_node_completion` → `handle_node_cleared` (FixedUpdate, NodeState::Playing)

`track_node_completion` reads `CellDestroyedAt` messages, decrements `ClearRemainingCount`,
and sends `NodeCleared` when it hits zero.

`handle_node_cleared` reads `NodeCleared`, checks whether `node_index >= final_index`,
sets `NodeResult::Won` or leaves `InProgress`, sets `transition_queued = true`, and sends
`ChangeState<NodeState>`.

**Neither system increments `node_index`.** `handle_node_cleared` only reads it.

**File:** `breaker-game/src/state/run/node/systems/track_node_completion.rs`
**File:** `breaker-game/src/state/run/node/lifecycle/systems/handle_node_cleared.rs`

---

### 7. State machine teardown → RunState routing

`ChangeState<NodeState>` triggers the `rantzsoft_stateflow` dispatch chain:

```
NodeState::Playing → NodeState::AnimateOut (message-triggered)
NodeState::AnimateOut → NodeState::Teardown (condition: when=|_| true, pass-through)
NodeState::Teardown → triggers RunState route (condition: NodeState == Teardown)
RunState::Node → resolve_node_next_state(world) → ChipSelect | RunEnd | Teardown
```

`resolve_node_next_state` reads `NodeOutcome.result`:
- `InProgress` → `RunState::ChipSelect`
- `Quit` → `RunState::Teardown`
- anything else → `RunState::RunEnd`

After ChipSelect, the `ChipSelectState::Teardown` condition triggers `RunState::ChipSelect →
RunState::Node`, which fires `OnEnter(RunState::Node)` again — and `advance_node` runs,
incrementing `node_index`.

**File:** `breaker-game/src/state/plugin/system.rs` — `register_node_routes`,
`register_run_routes`, `resolve_node_next_state`

---

## Data Flow Summary

```
DifficultyCurve (RON)
    → generate_node_sequence_system
    → NodeSequence { assignments: Vec<NodeAssignment { tier_index, hp_mult, timer_mult, node_type }> }

OnEnter(RunState::Node)
    → advance_node → NodeOutcome.node_index += 1

OnEnter(NodeState::Loading)
    → set_active_layout   reads NodeOutcome.node_index → ActiveNodeLayout
    → init_node_timer     reads NodeOutcome.node_index + NodeSequence → NodeTimer
    → spawn_cells_from_layout  reads NodeOutcome.node_index + NodeSequence → cells spawned

NodeState::Playing (FixedUpdate)
    → CellDestroyedAt messages → track_node_completion → NodeCleared message
    → handle_node_cleared reads NodeCleared → NodeOutcome.result, ChangeState<NodeState>

ChangeState<NodeState>
    → stateflow dispatch → NodeState::AnimateOut → Teardown
    → RunState route → ChipSelect (InProgress) | RunEnd (won/lost)

RunState::ChipSelect teardown → OnEnter(RunState::Node) → advance_node
```

---

## State Machine (Relevant Nodes)

```
RunState::Loading  (reset_run_state: node_index=0)
    ↓
RunState::Setup    (setup_run: spawns breaker+bolt, node_index==0 guard)
    ↓ condition route
RunState::Node     (advance_node fires: node_index++)
    ↓ NodeState sub-states
    NodeState::Loading → AnimateIn → Playing → AnimateOut → Teardown
    ↓ Teardown condition route
    NodeOutcome.result == InProgress → RunState::ChipSelect
    NodeOutcome.result != InProgress → RunState::RunEnd
        ↓ ChipSelect teardown
    RunState::Node (advance_node fires again: node_index++)
```

---

## Existing Tier Concept

`tier_index: u32` already exists on `NodeAssignment`. It is populated by
`generate_node_sequence_system` and stored in `NodeSequence`. **It is never read after
generation.** No system currently surfaces the current tier to any other system.

No other "tier" or "difficulty level" concept exists in the codebase outside of:
- `NodeAssignment.tier_index` (in `NodeSequence`)
- `TierDefinition` (in `DifficultyCurve`)
- `DifficultyCurve.tiers` (the curve definition)

There is no `current_tier` resource, no component, and no query helper.

---

## What Needs to Change to Add a Tier Stub

The protocol/hazard system needs to query "what tier is the player currently on?"

### Option A: Compute on demand from resources (no new state)

Any system can compute the current tier by reading two existing resources:

```rust
fn get_current_tier(
    run_state: Res<NodeOutcome>,
    node_sequence: Res<NodeSequence>,
) -> u32 {
    node_sequence
        .assignments
        .get(run_state.node_index as usize)
        .map_or(0, |a| a.tier_index)
}
```

This is zero additional state. The values are already correct in `NodeSequence`. The only cost
is a read of two resources per system that needs the tier. Protocol/hazard systems can use this
pattern directly.

**Advantages**: no new resource, no new system, no synchronization risk.

**Disadvantage**: if many protocol systems all need the tier, they all repeat the same lookup.

### Option B: Add `current_tier: u32` to `NodeOutcome`

```rust
pub struct NodeOutcome {
    pub node_index: u32,
    pub result: NodeResult,
    pub transition_queued: bool,
    pub current_tier: u32,   // ← new field
}
```

Populated by a new system (or by modifying `advance_node`) on `OnEnter(NodeState::Loading)`:

```rust
fn sync_current_tier(
    mut outcome: ResMut<NodeOutcome>,
    node_sequence: Option<Res<NodeSequence>>,
) {
    if let Some(seq) = node_sequence {
        if let Some(assignment) = seq.assignments.get(outcome.node_index as usize) {
            outcome.current_tier = assignment.tier_index;
        }
    }
}
```

Or inline in `advance_node` — but `advance_node` runs before the sequence lookup systems,
and `NodeSequence` exists at that point, so it could be extended:

```rust
pub(crate) fn advance_node(
    mut run_state: ResMut<NodeOutcome>,
    node_sequence: Option<Res<NodeSequence>>,
) {
    run_state.node_index += 1;
    run_state.transition_queued = false;
    if let Some(seq) = node_sequence {
        run_state.current_tier = seq.assignments
            .get(run_state.node_index as usize)
            .map_or(0, |a| a.tier_index);
    }
}
```

This keeps `current_tier` on `NodeOutcome`, which is already the canonical run-progress
resource. Protocol systems read `NodeOutcome` and get the tier for free.

**Advantages**: single read site, protocol systems only need `Res<NodeOutcome>`.
**Disadvantage**: a new field on an existing resource — requires updating `reset_run_state`
(it does `*run_state = NodeOutcome::default()`, which would zero `current_tier` correctly
if it derives `Default`).

### Option C: Separate `CurrentTier(u32)` resource

A thin newtype resource inserted/updated by a dedicated system. Clean separation but more
boilerplate for minimal gain over Option B.

---

## Recommendation for Tier Stub

**Option B** is the right fit. `NodeOutcome` is already the resource everything reads for
run-progress queries. Adding `current_tier: u32` to it keeps all progression state in one
place and requires no new systems. The field gets updated inside `advance_node` (which already
mutates `NodeOutcome`) using the existing `NodeSequence` read.

Changes required:

1. **`NodeOutcome`** (`breaker-game/src/state/run/resources/definitions.rs`):
   - Add `pub current_tier: u32` with `#[doc]` comment.
   - `Default` already works (u32 defaults to 0).

2. **`advance_node`** (`breaker-game/src/state/run/systems/advance_node.rs`):
   - Add `Option<Res<NodeSequence>>` parameter.
   - After `node_index += 1`, set `current_tier = seq.assignments[node_index].tier_index`.
   - Falls back to 0 when `NodeSequence` is absent (tests, scenarios without sequence).

3. **`reset_run_state`** (`breaker-game/src/state/run/loading/systems/reset_run_state.rs`):
   - No change needed — `*run_state = NodeOutcome::default()` resets `current_tier` to 0.

4. **Tests**: `advance_node` tests will need a `NodeSequence` resource in the test app, or the
   new parameter must be `Option<Res<NodeSequence>>` so existing tests compile without it.

---

## Edge Cases

- **NodeSequence absent (tests, scenario runner without sequence)**: `Option<Res<NodeSequence>>`
  fallback to tier 0 is the correct behavior — no sequence means flat difficulty.
- **node_index out of bounds** (more nodes played than sequence length — e.g., scenario runner
  cycling layouts): `assignments.get(node_index)` returns `None`, falls back to 0. This is
  consistent with how `init_node_timer` and `spawn_cells_from_layout` already handle it.
- **Boss node**: `tier_index` correctly stays at the tier of the boss. A boss node at tier 1
  will report `current_tier = 1`, which is correct for hazard/protocol scaling.
- **First node off-by-one**: `advance_node` increments before `set_active_layout` reads. So
  when the first node is playing, `node_index == 1`. The `current_tier` will reflect
  `assignments[1].tier_index`. This matches the existing `hp_mult`/`timer_mult` behavior —
  the protocol/hazard system will see the same tier as the cell difficulty uses.

---

## Key Files

- `breaker-game/src/state/run/resources/definitions.rs` — `NodeOutcome`, `NodeSequence`,
  `NodeAssignment` (struct definitions; `tier_index` already on `NodeAssignment`)
- `breaker-game/src/state/run/systems/advance_node.rs` — sole write site for `node_index`;
  the recommended place to also set `current_tier`
- `breaker-game/src/state/run/loading/systems/generate_node_sequence/system.rs` — generates
  the flat sequence with `tier_index` baked into each `NodeAssignment`
- `breaker-game/src/state/run/loading/systems/reset_run_state.rs` — resets `NodeOutcome` at
  run start (no change needed if `current_tier` derives Default)
- `breaker-game/src/state/run/node/lifecycle/systems/handle_node_cleared.rs` — reads
  `node_index` to detect final node; does not touch `current_tier`
- `breaker-game/src/state/plugin/system.rs` — routing table; `resolve_node_next_state` reads
  `NodeOutcome.result` only — no change needed for tier stub
- `breaker-game/src/state/run/node/systems/init_node_timer.rs` — shows the pattern for reading
  `NodeSequence.assignments[node_index]` (exactly what `advance_node` extension should copy)
- `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/system.rs` —
  `resolve_hp_mult` shows the same `Option<Res<NodeSequence>>` fallback pattern to copy
