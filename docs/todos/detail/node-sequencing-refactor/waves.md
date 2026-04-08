# Implementation Waves

This todo is large enough to warrant multiple feature branches. Each wave is a self-contained deliverable that can be committed and verified independently.

## Prerequisites

Must be complete before Wave 0 starts:
- **Cell builder pattern** (todo #1) — provides `Cell::builder()`, `CellBehavior` enum, `.with_behaviors()`
- **Protocol & hazard system** (todo #7) — protocols and hazards must exist as implemented systems (their RNG channels are planned but wire-up happens in Wave 0)

## Wave 0: RNG Migration

**Branch**: `feature/rng-domain-partitioning`
**Goal**: Replace flat single-stream `GameRng` with per-domain RNG. Zero gameplay changes — just rewiring existing randomness to deterministic, isolated streams.
**Depends on**: nothing (can start immediately, even before prerequisites)

Full implementation details in [rng-architecture.md](rng-architecture.md).

### Sub-wave 0a: FxRng + visual systems
- Add `FxRng(ChaCha8Rng)` resource to `shared/rng.rs`, seeded from OS entropy at app startup
- Migrate `spawn_highlight_text` → `FxRng`
- Migrate `spawn_transition_out` / `spawn_transition_in` → `FxRng`
- **Files**: `shared/rng.rs`, `state/run/node/lifecycle/systems/spawn_highlight_text/`, `state/transition/`
- **Risk**: zero — cosmetic-only changes

### Sub-wave 0b: BoltRng + ordering pins
- Add `BoltRng(ChaCha8Rng)` resource, seeded from `hash(run_seed, "bolt", node_index)` at `OnEnter(NodeState::Loading)`
- Add `seed_node_rng` system (initially just seeds `BoltRng`; later waves add `NodeGenRng`)
- Migrate `launch_bolt`, `bolt_lost`, `setup_run`, `reset_bolt` → `BoltRng`
- Pin ordering: `seed_node_rng` → `setup_run`, `seed_node_rng` → `reset_bolt`
- Add `derive_seed()` and `derive_seed_named()` hash functions to `shared/rng.rs`
- **Files**: `shared/rng.rs`, `bolt/systems/launch_bolt/`, `bolt/systems/bolt_lost/`, `state/run/systems/setup_run/`, `state/run/node/systems/reset_bolt/`, `bolt/plugin.rs`
- **Risk**: low — bolt angles may differ from previous seeds (acceptable, not a regression)

### Sub-wave 0c: ChipRng
- Add `ChipRng(ChaCha8Rng)` resource, seeded from `hash(run_seed, "chip", chip_select_count)`
- Add `ChipSelectCount(u32)` resource (or fold into `RunProgress`), incremented each chip select
- Migrate `generate_chip_offerings` → `ChipRng`
- **Files**: `shared/rng.rs`, `state/run/chip_select/systems/generate_chip_offerings.rs`, `chips/offering/`
- **Risk**: low — chip offerings may differ from previous seeds

### Sub-wave 0d: EffectEventCounter + ephemeral effect RNG
- Add `EffectEventCounter(u64)` resource, reset at `OnEnter(NodeState::Loading)` (in `seed_node_rng`)
- Add `EffectBaseSeed(u64)` resource, derived from `hash(run_seed, "effect")` at run start
- Change effect bridge dispatch to create ephemeral `ChaCha8Rng::seed_from_u64(hash(effect_base, counter))` per fire
- Change all effect `fire()` signatures: `ResMut<GameRng>` → `&mut ChaCha8Rng`
- Change `tick_chain_lightning`: derive per-chain-per-tick RNG
- **Files**: `shared/rng.rs`, `effect/core/` (bridge dispatch), all `effect/effects/*/effect.rs`, `effect/effects/chain_lightning/effect.rs`
- **Risk**: medium — touches all effect fire paths. Extensive existing test coverage mitigates. Effect outcomes will differ from previous seeds.
- **Note**: largest sub-wave. Consider splitting further if the effect `fire()` signature change touches too many files at once.

### Sub-wave 0e: NodeSequenceRng + seed capture refactor
- Add `NodeSequenceRng(ChaCha8Rng)` resource, seeded from `hash(run_seed, "node_sequence")`
- Move `capture_run_seed` to `OnExit(MenuState::Main)`, ordered before `generate_node_sequence_system`
- Migrate `generate_node_sequence_system` → `NodeSequenceRng`
- `GameRng` no longer drawn from directly during gameplay (still exists as root entropy for deriving sub-seeds for unseeded runs)
- **Files**: `shared/rng.rs`, `state/run/loading/systems/capture_run_seed.rs`, `state/run/loading/systems/generate_node_sequence/`, `state/run/plugin.rs`
- **Risk**: medium — changes run initialization ordering. Must verify scenarios still produce deterministic results.

### Sub-wave 0f: ProtocolRng + HazardRng
- Add `ProtocolRng` and `HazardRng` resources (if protocols/hazards exist by this point)
- Wire into protocol offering and hazard selection systems
- If protocols/hazards don't exist yet: define the resources and seed channels, wire when those systems land
- **Files**: `shared/rng.rs`, protocol/hazard system files (when they exist)
- **Risk**: low — new systems, no migration

### Wave 0 deliverables
- `docs/architecture/rng.md` — seed derivation tree, coverage guarantees
- All existing `GameRng` call sites migrated to domain-specific RNG
- `GameRng` retained only as root entropy source for unseeded runs
- All existing tests updated to use new RNG resource types

---

## Wave 1: Data Types & RON Loading

**Branch**: `feature/node-gen-data-types`
**Goal**: Define all Rust types and RON formats for frames, blocks, constraints, tier pools. Asset loading with hot-reload. No generation logic yet.
**Depends on**: Cell builder pattern (for `CellBehavior` enum)

### Sub-wave 1a: Core types
- `BlockSize` enum (S4x3, S4x4, S6x4, S6x6, S8x4, S8x6, S10x5, S10x8)
- `CellConstraint` enum (`Any`, `MustInclude(Vec<CellBehavior>)`, `MustNotInclude(Vec<CellBehavior>)`)
- `FrameDef` RON type (grid size, fixed cells, slot definitions)
- `BlockDef` RON type (size, min_tier, max_tier, cell positions with constraints)
- `TierModifierPool` RON type (weighted list of modifiers per tier)
- Modifier deny-list (incompatible pairs, in code)
- **Files**: new `cells/generation/` module (or `run/generation/`), `shared/` types
- **Domain**: this is node generation infrastructure, likely lives in `run/` or a new `generation/` domain

### Sub-wave 1b: Asset loading + hot-reload
- Frame asset loader (RON → `FrameDef`)
- Block asset loader (RON → `BlockDef`)
- Tier pool asset loader (RON → `TierModifierPool`)
- Hot-reload: on asset change, if the current node uses the changed frame/block, kill and respawn cell entities
- **Files**: asset loaders, `run/node/` lifecycle integration
- **Note**: frame format decision (visual grid vs inline list) must be finalized before this sub-wave

### Sub-wave 1c: Tier state resources
- `TierConfig` resource (tier level, modifier pool handle, hazard stack)
- `RunProgress` resource (current tier, node_in_tier, total_nodes_cleared)
- Lifecycle: `TierConfig` rebuilt when `RunProgress` crosses a tier boundary
- **Files**: `run/` resources, state transition systems

### Wave 1 deliverables
- All types compile and serialize/deserialize from RON
- A few test frame and block RON files load correctly
- Hot-reload proof of concept
- `TierConfig` / `RunProgress` resources exist (not yet wired to generation)

---

## Wave 2: Composition Engine

**Branch**: `feature/node-composition-engine`
**Goal**: The generator that composes nodes from frames + blocks + constraints. Given a tier and node type, produce a fully resolved grid of cells.
**Depends on**: Wave 1 (types + loading)

### Sub-wave 2a: Frame selection + block selection
- Frame selection: given node type + tier, pick a frame from the eligible pool (seeded)
- Block selection: given a slot size + tier, pick a block whose tier range includes current tier (seeded)
- Slot splitting: enumerate valid axis-aligned splits for a slot size, weighted random selection
- **Files**: `run/generation/` (or wherever the composition engine lives)

### Sub-wave 2b: Constraint resolution
- Given a `CellConstraint` + `TierModifierPool` + RNG → `Vec<CellBehavior>`
- Apply `MustInclude` (ensure required behaviors)
- Apply `MustNotInclude` (remove from pool)
- Apply deny-list (filter incompatible combos)
- Roll remaining behaviors from weighted pool
- **Files**: `run/generation/`

### Sub-wave 2c: Full composition pipeline
- Wire frame selection → slot splitting → block selection → constraint resolution → `Cell::builder().with_behaviors()`
- Sequence assignment: block-scoped explicit + generator cross-block assignment
- HP scaling per tier
- `NodeGenRng` integration (seeded from `hash(run_seed, tier_index, node_index)`)
- **Files**: `run/generation/`, integration with `seed_node_rng` from Wave 0

### Sub-wave 2d: Portal frame composition
- Portal frames selected from pool with `portal: true` flag
- Use tier N-2 modifier pool and frame/block pool
- Enforce zero nesting (portal frames cannot resolve Portal constraints)
- **Files**: `run/generation/` portal path

### Wave 2 deliverables
- Given a seed, tier, and node type: deterministically produces a fully resolved cell grid
- Unit tests for each sub-wave (frame selection, block selection, constraint resolution, full pipeline)
- Portal composition tested
- Not yet wired into the game — just the engine

---

## Wave 3: Tier System + Game Integration

**Branch**: `feature/tier-system`
**Goal**: Replace the current upfront `NodeSequence` with per-tier batching. Wire the composition engine into the run flow.
**Depends on**: Wave 2 (composition engine)

### Sub-wave 3a: Per-tier node generation
- When player reaches a new tier boundary, generate the next tier's 4 non-boss + 1 boss node
- `TierConfig` rebuilt with the new tier's modifier pool and hazard stack
- Node type ramp enforced (PPPA → VVVV based on tier index)
- `NodeSequence` becomes a buffer of upcoming nodes, refilled per tier instead of all upfront
- **Files**: `state/run/loading/systems/generate_node_sequence/`, `run/` state management

### Sub-wave 3b: Tier boundary transitions
- Detect tier boundary (last node of tier cleared)
- Trigger tier generation for next tier
- Hazard selection at tier boundary (using `HazardRng`)
- Protocol offering at appropriate points (using `ProtocolRng`)
- `RunProgress` updated
- **Files**: `state/run/` transition systems, integration with protocol/hazard systems

### Sub-wave 3c: Modifier introduction timeline
- Enforce modifier introduction: tier 1 only has Volatile, tier 3 adds Sequence, etc.
- `TierModifierPool` filtered by what's unlocked at the current tier
- Portal modifier only available in volatile nodes from tier 5+
- **Files**: `run/generation/` pool filtering, tier_pools.ron

### Wave 3 deliverables
- Runs use per-tier generation instead of upfront sequence
- Tier boundaries trigger generation, hazard selection, protocol offering
- Modifier availability respects introduction timeline
- Game is playable with the new system (even with placeholder content)

---

## Wave 4: NodeLayout Migration

**Branch**: `feature/nodelayout-migration`
**Goal**: Replace the old `NodeLayout` RON format with the new frame/block system. Clean break.
**Depends on**: Wave 3 (tier system working)

### Sub-wave 4a: Convert existing nodes to frames
- Each existing hand-designed `NodeLayout` becomes a frame with all fixed cells and no slots
- Boss nodes become boss layout files (separate format, not frame/block)
- Verify converted frames produce identical gameplay to old NodeLayout
- **Files**: `assets/nodes/` → `assets/frames/`, `assets/bosses/`

### Sub-wave 4b: Wire new loading into run flow
- Replace `NodeLayout` loading with frame-based composition
- Node loading path: `OnEnter(NodeState::Loading)` → compose from frame → spawn cells
- **Files**: `state/run/node/` loading systems

### Sub-wave 4c: Delete old code
- Remove `NodeLayout` type, loader, and all associated code
- Remove old `assets/nodes/` directory
- Remove `*.cell.ron` files that are replaced by tier pool resolution (e.g., `tough.cell.ron`)
- **Files**: cleanup across `cells/`, `state/run/node/`

### Wave 4 deliverables
- Old `NodeLayout` system fully removed
- All nodes composed through frame/block system
- Existing gameplay preserved (converted frames match old layouts)

---

## Wave 5: Content Authoring

**Branch**: `feature/node-gen-content`
**Goal**: Author the starter frame and block sets. Iterate through playtesting.
**Depends on**: Wave 4 (migration complete, new system is the only path)

### Sub-wave 5a: Starter frames
- ~12 passive frames (mixed fixed cells + slots)
- ~12 active frames
- ~24 volatile frames
- Portal frames (smaller, tagged `portal: true`)
- **Files**: `assets/frames/`

### Sub-wave 5b: Starter blocks (tiers 1-4)
- Blocks for all 8 standard sizes at tiers 1-4 (~108 blocks)
- Focus on early-game variety first
- **Files**: `assets/blocks/`

### Sub-wave 5c: Starter blocks (tiers 5-7)
- Blocks for tiers 5-7 (~121 blocks, more per tier for infinite run variety)
- More complex constraints, more modifier interactions
- **Files**: `assets/blocks/`

### Sub-wave 5d: Tier pool tuning
- `tier_pools.ron` with weights per tier
- Modifier deny-list finalized
- Playtesting iteration on weights, block distribution, frame sizes
- **Files**: `assets/tier_pools.ron`, code-level deny-list

### Wave 5 deliverables
- ~48 frames authored
- ~229 blocks authored across all tiers
- Tier pools tuned
- Game feels good through tier 8 and into infinite scaling

---

## Wave 6: Polish + Documentation

**Branch**: part of Wave 5 branch or standalone
**Goal**: Documentation, terminology, and post-landing tasks.

- `docs/architecture/rng.md` (if not written in Wave 0)
- Terminology additions to `docs/design/terminology/` (Volatile, Tier, Frame, Block, Portal Cell)
- Update `docs/plan/` with completed phase
- Fill Tier Regression stub (see main doc Post-Landing section)
- Scenario coverage for new generation system

---

## Wave Dependency Graph

```
Prerequisites (cell builder, protocols/hazards)
    │
    ├─ Wave 0: RNG Migration (can start before prerequisites)
    │   0a → 0b → 0c → 0d → 0e → 0f
    │
    └─ Wave 1: Data Types + Loading (needs cell builder)
        │
        Wave 2: Composition Engine
        │
        Wave 3: Tier System + Integration
        │
        Wave 4: NodeLayout Migration
        │
        Wave 5: Content Authoring
        │
        Wave 6: Polish + Docs
```

Wave 0 is fully independent — it can run in parallel with Waves 1-2 since it only touches existing RNG code, not generation code. Waves 1-5 are sequential.

## Estimated Scope

| Wave | Sub-waves | New files | Modified files | Content files |
|------|-----------|-----------|----------------|---------------|
| 0    | 6         | ~2        | ~20            | 0             |
| 1    | 3         | ~8        | ~5             | ~5 test RON   |
| 2    | 4         | ~6        | ~3             | 0             |
| 3    | 3         | ~3        | ~10            | 0             |
| 4    | 3         | 0         | ~15            | ~10 converted |
| 5    | 4         | 0         | ~3             | ~280 RON      |
| 6    | —         | ~3        | ~5             | 0             |
