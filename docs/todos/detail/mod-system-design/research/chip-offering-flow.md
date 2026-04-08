# Behavior Trace: Chip Offering and Selection Flow

Bevy version: **0.18.1** (confirmed from `breaker-game/Cargo.toml`).

---

## Trigger

Node is cleared → `NodeState` transitions to `Teardown` → `RunState` transitions to `ChipSelect` via the stateflow routing table → `ChipSelectState` sub-state advances through `Loading → AnimateIn → Selecting`.

`OnEnter(ChipSelectState::Selecting)` fires the offering and spawn chain.

---

## 1. ChipCatalog Structure

### `ChipDefinition` (runtime, never deserialized directly)

```
ChipDefinition {
    name: String,             // display name ("Basic Piercing Shot")
    description: String,      // flavor text
    rarity: Rarity,           // Common | Uncommon | Rare | Legendary | Evolution
    max_stacks: u32,          // max times the player may stack this specific chip
    effects: Vec<RootEffect>, // effect tree
    ingredients: Option<Vec<EvolutionIngredient>>, // None for non-evolutions
    template_name: Option<String>, // "Piercing Shot" (the source template name)
}
```

### `ChipCatalog` (Resource)

```
ChipCatalog {
    chips: HashMap<String, ChipDefinition>,  // keyed by display name
    order: Vec<String>,                       // insertion order for deterministic iteration
    recipes: Vec<Recipe>,                     // evolution combos
}
```

`ChipCatalog` is a `Resource` — it lives for the duration of the app, not the run. It is
built once during `AppState::Loading` by the `build_chip_catalog` system and hot-reloaded
in the `dev` feature via `propagate_chip_catalog`.

### `Rarity` enum

```rust
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
    Evolution,
}
```

### Load path

1. `ChipTemplateRegistry` is a `SeedableRegistry` — loads all `assets/chips/standard/*.chip.ron`
   files at startup as `ChipTemplate` assets.
2. `EvolutionTemplateRegistry` — loads all `assets/chips/evolutions/*.evolution.ron` files.
3. `build_chip_catalog` (iyes_progress tracked, runs in `AppState::Loading`) calls
   `expand_chip_template` on each `ChipTemplate`: iterates four optional rarity slots
   (`common`, `uncommon`, `rare`, `legendary`), builds one `ChipDefinition` per non-`None`
   slot. Template name and prefix determine the final display name.
4. Evolution templates are expanded via `expand_evolution_template` and added to the catalog
   alongside a `Recipe` entry.

### RON format (standard chip)

```ron
(
    name: "Piercing Shot",
    max_taken: 3,
    common:    (prefix: "Basic", effects: [...]),
    uncommon:  (prefix: "Keen",  effects: [...]),
    rare:      (prefix: "Brutal", effects: [...]),
    legendary: (prefix: "",      effects: [...]),  // optional — not all chips have this
)
```

`max_taken` is shared across ALL rarity variants of the template — it is a template-level cap.

### RON format (evolution)

```ron
(
    name: "Nova Lance",
    description: "...",
    effects: [...],
    ingredients: [
        (chip_name: "Impact", stacks_required: 2),
        (chip_name: "Bolt Speed", stacks_required: 3),
    ],
)
```

Evolution `max_stacks` defaults to 1 if omitted.

---

## 2. State Transition Chain Leading to ChipSelect

```
NodeState::Playing
  ↓  NodeCleared message sent (track_node_completion)
  ↓  handle_node_cleared sets NodeOutcome.transition_queued = true,
     sends ChangeState<NodeState>
NodeState::AnimateOut  (pass-through)
NodeState::Teardown    (triggers cleanup)
  ↓  stateflow routing: NodeState::Teardown → RunState fires to_dynamic(resolve_node_next_state)
     resolve_node_next_state reads NodeOutcome.result:
       InProgress → RunState::ChipSelect
       Won/Lost   → RunState::RunEnd
       Quit       → RunState::Teardown
RunState::ChipSelect
  → ChipSelectState::Loading (default sub-state, pass-through)
  → ChipSelectState::AnimateIn (pass-through)
  → ChipSelectState::Selecting  ← systems fire here
```

All `RunState` / `ChipSelectState` transitions use the `rantzsoft_stateflow` routing table
registered in `StatePlugin`. No `NextState` is set directly in game code for these
transitions — they go through `ChangeState<T>` messages and routing routes.

---

## 3. System Chain (OnEnter + Update)

### OnEnter(ChipSelectState::Selecting) — registered in `ChipSelectPlugin`

The three systems run as a chained sequence with `ApplyDeferred` between steps 1 and 2:

**Step 1: `generate_chip_offerings`** (`state/run/chip_select/systems/generate_chip_offerings.rs`)
- Reads: `ChipCatalog`, `ChipInventory`, `ChipSelectConfig`, `GameRng`, `ActiveNodeLayout`
- Writes: inserts `ChipOffers` resource via `Commands`
- Logic:
  1. Builds `rarity_weights` map from `ChipSelectConfig` fields:
     - Common: 100.0, Uncommon: 50.0, Rare: 15.0, Legendary: 3.0
  2. If `ActiveNodeLayout.pool == NodePool::Boss`, checks `ChipCatalog.eligible_recipes()`:
     - Recipes whose ingredients are all satisfied by `ChipInventory` become
       `ChipOffering::Evolution { ingredients, result }` entries, up to `offers_per_node`.
  3. Remaining slots filled by `generate_offerings()` (pure function, no ECS).
  4. Inserts `ChipOffers(Vec<ChipOffering>)`.

**ApplyDeferred** — flushes `Commands` so `ChipOffers` is visible to the next system.

**Step 2: `spawn_chip_select`** (`state/run/chip_select/systems/spawn_chip_select.rs`)
- Reads: `ChipSelectConfig`, `ChipOffers`
- Spawns: `ChipSelectScreen` entity tree (timer display, title, card row, prompt)
- Inserts: `ChipSelectTimer { remaining: config.timer_secs }`, `ChipSelectSelection { index: 0 }`
- Each card is spawned with a `ChipCard { index: i }` and `Button` component
- Card displays chip name (from `offering.definition().name`), rarity string, and description

---

### Update (while `in_state(ChipSelectState::Selecting)`) — `ChipSelectPlugin`

Three systems run **chained** each frame:

**1. `handle_chip_input`**
- Reads: `ButtonInput<KeyCode>`, `InputConfig`, `ChipOffers`, `ChipSelectSelection`, `ChipInventory`, `ChipSelectConfig`
- On left/right arrow: wraps `ChipSelectSelection.index`
- On confirm key:
  1. Sends `ChipSelected { name: offering.name().to_owned() }` message
  2. If offering is `Evolution`: calls `inventory.remove_by_template()` for each ingredient
  3. Records decay on all NON-selected offers: `inventory.record_offered(name, seen_decay_factor)`
  4. Sends `ChangeState<ChipSelectState>` → routes to `AnimateOut`
- If card count is 0: confirm key still sends `ChangeState` (skip with no chip)

**2. `tick_chip_timer`**
- Decrements `ChipSelectTimer.remaining` by `time.delta_secs()`
- On expiry (≤ 0.0):
  - Records decay on ALL normal offerings (not evolution offerings)
  - Sends `ChangeState<ChipSelectState>` → routes to `AnimateOut`

**3. `update_chip_display`**
- Updates `ChipTimerText` entity with `timer.remaining.ceil().max(0.0)`
- Updates `BorderColor` on all `ChipCard` entities: selected index gets `selected_color_rgb`,
  others get `normal_color_rgb`

---

### Update (while `in_state(ChipSelectState::Selecting)`) — `RunPlugin`

Three additional systems run in parallel with the chip select systems:

- **`track_chips_collected`**: reads `ChipSelected`, pushes `msg.name` to `RunStats.chips_collected`
- **`detect_first_evolution`**: reads `ChipSelected`, checks if chip name matches any recipe result;
  if so, increments `RunStats.evolutions_performed`, sets `HighlightTracker.first_evolution_recorded`,
  emits `HighlightTriggered { kind: FirstEvolution }`
- **`snapshot_node_highlights`**: drains `RunStats.highlights`, partitions by node, selects best,
  resets per-node `HighlightTracker` counters

---

### Update (while `in_state(ChipSelectState::Selecting)`) — `ChipsPlugin`

**`dispatch_chip_effects`** — the single most important consumer of `ChipSelected`:

- Reads: `ChipSelected` messages, `ChipCatalog`, `ChipInventory`
- Queries: Breaker entities, Bolt entities, Cell entities, Wall entities
- For each `ChipSelected` message:
  1. Looks up `ChipDefinition` in catalog by name; warns and skips if missing
  2. Calls `inventory.add_chip()` — enforces individual and template-level stack caps;
     warns and skips if already maxed
  3. For each `RootEffect::On { target, then }`:
     - If `target == Target::Breaker`: resolves Breaker entities immediately, dispatches
       child effects directly (Breaker exists during ChipSelect)
     - Otherwise: wraps children in `When(NodeStart, On(target, children))` and pushes to
       **Breaker's `BoundEffects`** — deferred until next node start (Bolt, Cell, Wall don't
       exist during ChipSelect)

`dispatch_chip_effects` runs only while `in_state(ChipSelectState::Selecting)` and only when
`ChipSelected` messages exist.

---

## 4. `generate_offerings` Algorithm (pure, no ECS)

Source: `src/chips/offering/system.rs`

### `build_active_pool`

1. Iterates `ChipCatalog.ordered_values()` (deterministic insertion order)
2. Skips `Rarity::Evolution` chips (they come from recipe matching, not the pool)
3. Skips any chip where `inventory.is_chip_available()` returns false:
   - Individual cap: `stacks >= max_stacks`
   - Template cap: `template_taken >= def.max_stacks`
4. For each remaining chip:
   - Looks up `base_weight = config.rarity_weights[chip.rarity]` (0.0 if not found)
   - Computes `effective_weight = base_weight * inventory.weight_decay(chip.name)` (1.0 if unseen)
   - Pushes `PoolEntry { name, weight, template_name }`

### `generate_offerings` (weighted draws with template deduplication)

1. `draws = min(offers_per_node, pool.len())`
2. Each draw:
   a. Builds `Vec<f32>` of weights from current pool
   b. `WeightedIndex::new()` — returns `Err` if all weights zero → breaks
   c. Samples index, `swap_remove` (avoids the drawn chip)
   d. If chosen chip has a `template_name`, removes ALL remaining pool entries sharing
      that `template_name` → no two offers from the same template
3. Returns resolved `ChipDefinition` values from catalog

---

## 5. Rarity Weights (from `defaults.chipselect.ron`)

| Rarity    | Weight |
|-----------|--------|
| Common    | 100.0  |
| Uncommon  |  50.0  |
| Rare      |  15.0  |
| Legendary |   3.0  |
| Evolution | excluded from pool entirely |

### Legendary: current status

13 chip templates currently have a `legendary:` rarity slot:
`parry`, `death_lightning`, `desperation`, `feedback_loop`, `powder_keg`, `gauntlet`,
`whiplash`, `ricochet_protocol`, `deadline`, `chain_reaction`, `glass_cannon`, `tempo`,
`singularity`.

Legendary chips function identically to Common/Uncommon/Rare — the rarity determines only
the base weight (3.0 vs 100.0). There is no special mechanic gating Legendary. They compete
in the same pool, get the same decay treatment, and are subject to the same template dedup.

The code is fully wired for Legendary: the `rarity_weights` map includes it, the RON files use
it, the `Rarity::Legendary` color config is defined. Removing it would require:
- Removing the `Legendary` variant from the `Rarity` enum
- Removing `rarity_weight_legendary` from `ChipSelectConfig`
- Removing the 13 `legendary:` slots from `.chip.ron` files
- Removing the color config entries

---

## 6. Weight Decay System

`ChipInventory.decay_weights: HashMap<String, f32>`

- Fresh (never offered): `weight_decay()` returns 1.0
- After being offered but NOT selected: `record_offered(name, 0.8)` → multiplies existing decay by 0.8
  - First decline: 1.0 × 0.8 = 0.80
  - Second decline: 0.80 × 0.8 = 0.64
  - etc.
- `mark_seen(name)` is a convenience alias for `record_offered(name, 0.8)`

Decay accumulates multiplicatively across node visits. There is no floor — a chip offered
many times could approach zero weight. Evolution offerings are NOT decayed on timer expiry
(see `tick_chip_timer` — only `ChipOffering::Normal` entries receive decay on timeout).

---

## 7. After ChipSelected: Full Effect Application

```
ChipSelected { name }
    ↓
dispatch_chip_effects (ChipsPlugin, Update, ChipSelectState::Selecting)
    ├── catalog.get(name)       → ChipDefinition
    ├── inventory.add_chip()    → records to ChipInventory.held (stacks/max_stacks)
    └── for each RootEffect::On { target, then }:
            if target == Breaker:
                resolve entities → dispatch_children()
                    Do(effect) → commands.fire_effect()
                    other      → commands.transfer_effect() → BoundEffects
            else:
                wrap: When(NodeStart, On(target, then)) → push to Breaker BoundEffects
```

`BoundEffects` is a component on the Breaker entity. Effects bound to non-Breaker targets
accumulate there and are fired when `NodeStart` triggers on the next node.

---

## 8. After Selection: State Teardown

```
handle_chip_input sends ChangeState<ChipSelectState>
    ↓
ChipSelectState::Selecting → AnimateOut (pass-through)
    → Teardown
        → cleanup_on_exit::<ChipSelectState> (despawns all entities with StateScoped<ChipSelectState>)
        → ChipSelectPlugin: cleanup_entities::<ChipSelectScreen>
RunState watches ChipSelectState::Teardown:
    RunState::ChipSelect → RunState::Node (with FadeOut transition)
        → OnEnter(RunState::Node): advance_node + show_gameplay_entities
            → OnEnter(NodeState::Loading): set_active_layout, spawn_cells, init_timer
```

---

## 9. Key Messages

| Message | Sender | Consumers |
|---------|--------|-----------|
| `NodeCleared` | `track_node_completion` (FixedUpdate) | `handle_node_cleared` (routes NodeState forward) |
| `ChangeState<NodeState>` | `handle_node_cleared`, `handle_timer_expired`, `handle_run_lost` | `rantzsoft_stateflow` dispatcher |
| `ChangeState<ChipSelectState>` | `handle_chip_input`, `tick_chip_timer` | `rantzsoft_stateflow` dispatcher |
| `ChipSelected { name }` | `handle_chip_input` | `dispatch_chip_effects`, `track_chips_collected`, `detect_first_evolution` |
| `HighlightTriggered` | `detect_first_evolution` | run-end summary system |

---

## 10. Edge Cases

### Empty pool
If all chips are maxed/unavailable, `build_active_pool` returns empty, `generate_offerings`
returns empty, `ChipOffers(vec![])` is inserted. `spawn_chip_select` spawns zero cards.
`handle_chip_input` detects `card_count == 0` and skips directly to `ChangeState` on confirm.

### WeightedIndex failure (all weights zero)
`WeightedIndex::new(&weights)` returns `Err` if all weights are zero. The `let Ok(dist) = ...`
guard breaks the draw loop, returning however many chips were already drawn. This can produce
fewer offers than `offers_per_node`.

### Evolution offering selected
`handle_chip_input` detects `ChipOffering::Evolution { ingredients, .. }` and calls
`inventory.remove_by_template()` for each ingredient before sending `ChipSelected`.
`dispatch_chip_effects` then adds the evolution result to inventory normally.

### Chip not found in catalog
`dispatch_chip_effects` warns via `tracing::warn!` and continues — the chip name in
`ChipSelected` doesn't have to match the catalog. No panic, no crash.

### Chip already maxed
If `inventory.add_chip()` returns `false`, `dispatch_chip_effects` warns and skips effect
application for that chip. The chip is still "selected" from the UI perspective but produces
no effects.

### Boss node
`generate_chip_offerings` checks `active_layout.0.pool == NodePool::Boss`. Only Boss
nodes offer evolution chips. If not Boss, `evolution_offers` is always empty.

### Timer expiry decay — evolution vs normal
On timeout, `tick_chip_timer` only records decay for `ChipOffering::Normal` offers — evolution
offers skip decay. This is intentional: evolutions are high-value and shouldn't be suppressed
by timer expiry.

---

## 11. Protocol Integration Point

The chip offering flow provides one natural integration seam for protocols:

**Integration point: `generate_chip_offerings`**

This system owns the `ChipOffers` resource construction. Protocols could be added as a new
variant alongside `ChipOffering::Normal` and `ChipOffering::Evolution`:

```rust
pub enum ChipOffering {
    Normal(ChipDefinition),
    Evolution { ingredients, result },
    Protocol(ProtocolDefinition),   // proposed new variant
}
```

The protocol would appear as one entry in `ChipOffers`. `spawn_chip_select` would render it
as a card (already iterates `offers` generically). `handle_chip_input` would need a new branch
for `ChipOffering::Protocol` — instead of calling `dispatch_chip_effects`, it would apply the
protocol.

The "opportunity cost" design (take protocol instead of a chip) is naturally satisfied by the
existing `offers_per_node` count: if one slot is occupied by a protocol, one fewer chip slot
is available. The generation logic in `generate_chip_offerings` would need to reserve one slot
for the protocol offering and fill the remainder with normal chips.

**What does NOT need to change**:
- `ChipCatalog` — protocols are not chips
- `dispatch_chip_effects` — only responds to `ChipSelected` with chip names in the catalog
- `ChipInventory` — protocols don't stack; they'd live in a separate `ProtocolInventory`
- The state machine — `ChipSelectState::Selecting` already handles the full offering screen

**What would need to change**:
- `ChipOffering` enum: add `Protocol(ProtocolDefinition)` variant
- `generate_chip_offerings`: add logic to include one protocol offering
- `spawn_chip_select` (or a new parallel system): render protocol cards distinctly
- `handle_chip_input`: add `ChipOffering::Protocol` match arm — apply protocol, not chip
- New `ProtocolCatalog` resource + `ProtocolDefinition` type
- New `ProtocolInventory` resource (per-run, tracks which protocols are held)

---

## Key Files

- `breaker-game/src/chips/definition/types.rs` — `Rarity` enum, `ChipTemplate`, `ChipDefinition`, `expand_chip_template`
- `breaker-game/src/chips/resources/data.rs` — `ChipCatalog`, `ChipTemplateRegistry`, `EvolutionTemplateRegistry`, `Recipe`
- `breaker-game/src/chips/inventory/data.rs` — `ChipInventory`, stacks/decay tracking
- `breaker-game/src/chips/offering/system.rs` — `build_active_pool`, `generate_offerings`, `OfferingConfig`
- `breaker-game/src/chips/systems/dispatch_chip_effects/system.rs` — chip effect application, BoundEffects dispatch
- `breaker-game/src/chips/plugin.rs` — `ChipsPlugin` registration, `dispatch_chip_effects` schedule placement
- `breaker-game/src/state/run/chip_select/resources.rs` — `ChipSelectConfig`, `ChipOffering`, `ChipOffers`
- `breaker-game/src/state/run/chip_select/systems/generate_chip_offerings.rs` — offering generation, boss evolution path
- `breaker-game/src/state/run/chip_select/systems/handle_chip_input.rs` — player input, `ChipSelected` send, decay recording
- `breaker-game/src/state/run/chip_select/systems/tick_chip_timer.rs` — timer expiry, auto-advance, decay on timeout
- `breaker-game/src/state/run/chip_select/systems/spawn_chip_select.rs` — UI card spawning
- `breaker-game/src/state/run/chip_select/plugin.rs` — `ChipSelectPlugin`, system schedule registration
- `breaker-game/src/state/plugin/system.rs` — routing table: `resolve_node_next_state`, `register_chip_select_routes`
- `breaker-game/src/state/types/chip_select_state.rs` — `ChipSelectState` sub-state enum
- `breaker-game/assets/chips/standard/*.chip.ron` — standard chip definitions (51 files)
- `breaker-game/assets/chips/evolutions/*.evolution.ron` — evolution definitions (16 files)
- `breaker-game/assets/config/defaults.chipselect.ron` — rarity weights, timer, visual config
