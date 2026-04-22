# Bolt-loss behavior — `BoltLossBehavior` enum + centralized handler

## Problem

Today's `BoltLost` handling is diffuse and inconsistent:

- `BreakerDefinition` has a `bolt_lost: RootNode` field that carries an effect-tree (`Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife | TimePenalty)))`). Loss behavior is buried in the effect system.
- The three shipped breakers (Aegis, Prism, Chrono) use one of two fire shapes: `Fire(LoseLife(()))` or `Fire(TimePenalty((seconds: N)))`. Two simple variants hiding behind a generic effect-tree.
- `TimePenaltyConfig::fire` mutates `NodeTimer` directly (bypasses the `ApplyTimePenalty` message + `apply_time_penalty` applier). Two paths into the same resource.
- `BoltLost` message has no payload; the "what happens" is determined entirely by which effect tree the breaker has bound to `Trigger::BoltLostOccurred`.
- Godmode breaker (scenario runner) inherits `BreakerDefinition::default()` which defaults to `Fire(LoseLife)` — effectively god-mode only by accident of `life_pool: None` (no Hp component for LoseLife to decrement). No explicit god-mode declaration.
- Reckless Dash's "double penalty while dashing" mechanic has no clean hook today; the older remediation (`breaker-bolt-lost-effect-component.md`) proposed a `MessageMutator<BoltLost>` + BoltLifecycleSystems triplet. Cleaner: mutate the breaker's behavior component on dash-enter, restore on dash-exit.

Prism adds zero design value — its `TimePenalty(7.0)` on bolt-lost is functionally identical to Chrono's `TimePenalty(5.0)`, just tuned differently. Removing Prism simplifies the breaker roster.

## Prerequisite

Blocked on TODO #0–#3 landing first. Reasons:
- **#2 (mutators domain refactor)** moves Reckless Dash from `protocol/protocols/reckless_dash/` to `mutators/protocols/reckless_dash/`. Adding a transition-detection system to Reckless Dash before #2 means doing the work twice.
- **#1 (death pipeline crate)** is conceptually adjacent; keeping this remediation after the crate settles avoids entangling bolt-lifecycle changes with damage-pipeline changes.
- **#3 (Greed skip button)** is small and should land before this to keep the chip-select UI in a complete state during the rework.

## Design

### `BoltLossBehavior` component/enum

New enum field on `BreakerDefinition`, loaded from RON, attached as a component to every Breaker entity at spawn:

```rust
// breaker/components.rs
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum BoltLossBehavior {
    /// Decrements breaker Hp by `u32` on each bolt lost. Default for life-pool breakers.
    LifeLoss(u32),
    /// Writes ReduceNodeTimer with `f32` delta on each bolt lost. Used by Chrono.
    TimeLoss(f32),
    /// No penalty. Used by scenario-runner Godmode and anywhere else that wants
    /// "bolt drops are free" semantics.
    None,
}
```

Replaces the `bolt_lost: RootNode` field on `BreakerDefinition`:

```rust
pub struct BreakerDefinition {
    // ...
    pub bolt_loss_behavior: BoltLossBehavior,  // replaces `bolt_lost: RootNode`
    pub salvo_hit: RootNode,  // STAYS — salvo is a separate trigger, still effect-tree-driven
    // ...
}

impl Default for BreakerDefinition {
    fn default() -> Self {
        Self {
            bolt_loss_behavior: BoltLossBehavior::LifeLoss(1),
            // ... (salvo_hit default unchanged)
        }
    }
}
```

### Centralized handler in breaker domain

New system at `breaker/systems/handle_bolt_lost.rs`:

```rust
pub fn handle_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    mut breakers: Query<(&BoltLossBehavior, Option<&mut Hp>), With<Breaker>>,
    mut time_writer: MessageWriter<ReduceNodeTimer>,
) {
    for msg in reader.read() {
        let Ok((behavior, hp_opt)) = breakers.get_mut(msg.breaker) else { continue; };
        match behavior {
            BoltLossBehavior::LifeLoss(n) => {
                if let Some(mut hp) = hp_opt {
                    hp.current = (hp.current - *n as f32).max(0.0);
                }
            }
            BoltLossBehavior::TimeLoss(delta) => {
                time_writer.write(ReduceNodeTimer { delta: *delta });
            }
            BoltLossBehavior::None => {}
        }
    }
}
```

Registered in `BreakerPlugin` in `FixedUpdate`, ordered after `bolt_lost` (the emitter in bolt domain) and before any system that could despawn the breaker.

### Message renames (node domain)

`state/run/node/messages.rs`:

```rust
// RENAMED from ApplyTimePenalty. Field `seconds` renamed to `delta`.
#[derive(Message, Clone, Debug)]
pub struct ReduceNodeTimer {
    pub delta: f32,
}

// RENAMED from ReverseTimePenalty. Field `seconds` renamed to `delta`.
// Subsumes `audit/remediations/rename-reverse-time-penalty-to-time-bonus.md`
// (destination changes from `TimeBonus` to `IncreaseNodeTimer`).
#[derive(Message, Clone, Debug)]
pub struct IncreaseNodeTimer {
    pub delta: f32,
}
```

Consumer systems rename:
- `apply_time_penalty` → `apply_reduce_node_timer`
- `reverse_time_penalty` → `apply_increase_node_timer`

SystemSet variant rename:
- `NodeSystems::ApplyTimePenalty` → `NodeSystems::ReduceNodeTimer`

Senders to sweep:
- `hazard/hazards/decay.rs` (Decay hazard per-stack speedup) — writes `ReduceNodeTimer`
- `effect_v3/effects/time_penalty/config.rs` — `TimePenaltyConfig::fire` currently bypasses the message and mutates `NodeTimer` directly. Change it to write `ReduceNodeTimer`. Single source of truth.
- New `breaker/systems/handle_bolt_lost.rs` — writes `ReduceNodeTimer` on `TimeLoss` variant
- `protocol/protocols/siphon/system.rs` — writes `IncreaseNodeTimer` on kill-streak continuation (was `ReverseTimePenalty`)

All 13 files in the sweep get updated mechanically (rename + field `seconds` → `delta`).

### Reckless Dash transition-detection

Today `DashState` is a 4-variant component enum (`Idle`, `Dashing`, `Braking`, `Settling`). State mutations happen in-place inside `breaker/systems/dash/system.rs`. No transition message is emitted.

Cleanest transition detection in Bevy 0.18: `Changed<DashState>` + a `PreviousDashState(DashState)` component maintained per-breaker.

New infrastructure in breaker domain:

```rust
// breaker/components.rs
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PreviousDashState(pub DashState);
```

```rust
// breaker/systems/update_previous_dash_state.rs
pub fn update_previous_dash_state(
    mut query: Query<(&DashState, &mut PreviousDashState)>,
) {
    for (state, mut prev) in &mut query {
        prev.0 = *state;
    }
}
```

Registered in `BreakerPlugin` `.after(dash_system)` so transitions are fully detected within one frame before the baseline updates for the next.

### Reckless Dash mutate-on-enter, restore-on-exit

New systems in `mutators/protocols/reckless_dash/` (post-#2 path):

```rust
// mutators/protocols/reckless_dash/system.rs
#[derive(Component, Clone, Copy, Debug)]
pub struct OriginalBoltLossBehavior(pub BoltLossBehavior);

pub fn reckless_dash_on_dash_transition(
    mut breakers: Query<(Entity, &DashState, &PreviousDashState, &mut BoltLossBehavior), Changed<DashState>>,
    originals: Query<&OriginalBoltLossBehavior>,
    mut commands: Commands,
) {
    for (entity, state, prev, mut behavior) in &mut breakers {
        let was_dashing = matches!(prev.0, DashState::Dashing);
        let now_dashing = matches!(state, DashState::Dashing);

        if !was_dashing && now_dashing {
            commands.entity(entity).insert(OriginalBoltLossBehavior(*behavior));
            *behavior = double_behavior(*behavior);
        } else if was_dashing && !now_dashing {
            if let Ok(orig) = originals.get(entity) {
                *behavior = orig.0;
                commands.entity(entity).remove::<OriginalBoltLossBehavior>();
            }
        }
    }
}

fn double_behavior(behavior: BoltLossBehavior) -> BoltLossBehavior {
    match behavior {
        BoltLossBehavior::LifeLoss(n) => BoltLossBehavior::LifeLoss(n.saturating_mul(2)),
        BoltLossBehavior::TimeLoss(n) => BoltLossBehavior::TimeLoss(n * 2.0),
        BoltLossBehavior::None => BoltLossBehavior::None,
    }
}
```

Registered via Reckless Dash's `wire(app)`:
- `.run_if(protocol_active(ProtocolKind::RecklessDash))`
- `.before(update_previous_dash_state)` so the transition detection reads the OLD `PreviousDashState` value before it's overwritten this tick.

Restoration on protocol deactivation / run end: the `OriginalBoltLossBehavior` overlay is automatically cleaned up because breakers are despawned at run end and respawned fresh. Within a run, the only way the overlay exists is "currently dashing" — the dash-exit branch always restores. If the player somehow dies mid-dash, the breaker despawns with the overlay; no leak.

### Prism removal

Prism breaker is retired as part of this remediation. Five files to update:

1. `breaker-game/assets/breakers/prism.breaker.ron` — DELETE
2. `breaker-game/src/state/menu/start_game/systems/spawn_run_setup.rs` — remove Prism from breaker-selection UI
3. `breaker-game/src/state/menu/start_game/systems/handle_run_setup_input.rs` — remove Prism handling (likely index-based)
4. `breaker-game/src/breaker/definition/tests.rs` — remove any test loading Prism
5. `breaker-game/src/breaker/builder/tests/ron_tests.rs` — remove Prism RON-load test
6. (sixth from grep) `breaker-game/src/bolt/components/definitions.rs` — check for a Prism-specific bolt default, remove if present

Breaker-selection UI reduces from `[Aegis, Prism, Chrono]` to `[Aegis, Chrono]`. Menu layout adjusts.

### RON migrations

Each shipped breaker's RON becomes a single enum variant instead of an effect-tree stamp:

```ron
// Aegis (before):
bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),

// Aegis (after):
bolt_loss_behavior: LifeLoss(1),
```

```ron
// Chrono (before):
bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(TimePenalty((seconds: 5.0))))),

// Chrono (after):
bolt_loss_behavior: TimeLoss(5.0),
```

`salvo_hit` fields unchanged on both (salvo is a separate trigger and still uses effect-tree).

### Godmode scenario-runner breaker

`breaker-scenario-runner/src/lifecycle/systems/menu_bypass.rs:59` currently builds the Godmode breaker via:

```rust
BreakerDefinition {
    name: "Godmode".to_owned(),
    bolt: "Bolt".to_owned(),
    life_pool: None,
    effects: vec![],
    ..BreakerDefinition::default()  // inherits the default bolt_lost effect tree
}
```

After the refactor:

```rust
BreakerDefinition {
    name: "Godmode".to_owned(),
    bolt: "Bolt".to_owned(),
    life_pool: None,
    bolt_loss_behavior: BoltLossBehavior::None,   // EXPLICIT god-mode
    effects: vec![],
    ..BreakerDefinition::default()
}
```

Documents the intent explicitly; no longer relies on "no Hp → LoseLife is a no-op" accident.

### Effect system retention

- **`LoseLifeConfig` stays.** Still used by `salvo_hit` trigger on breakers, and available for any future chip/hazard that wants to drain life via the effect system. Not routed through `handle_bolt_lost`.
- **`TimePenaltyConfig` stays.** Gets a small refactor: `fire` writes `ReduceNodeTimer` instead of mutating `NodeTimer` directly. Still used by `salvo_hit` on Chrono (5s on salvo impact), and anywhere else that wants a time-penalty effect.
- **`on_bolt_lost_occurred` bridge stays.** Still dispatches `Trigger::BoltLostOccurred` for any entity-bound effects (protocols, chips, hazards). Default breaker-life-loss just no longer routes through it (moved to `handle_bolt_lost` in breaker domain).

### System ordering

`FixedUpdate` sequence:
1. `bolt_lost` (bolt domain) — detects fallen bolts, writes `BoltLost` messages
2. `handle_bolt_lost` (breaker domain) — reads `BoltLost`, applies `BoltLossBehavior`, writes `ReduceNodeTimer` if `TimeLoss`
3. `on_bolt_lost_occurred` (effect_v3 bridge) — dispatches `Trigger::BoltLostOccurred` for entity-bound effects
4. `apply_reduce_node_timer` (node domain) — consumes `ReduceNodeTimer`
5. `reckless_dash_on_dash_transition` (protocol domain) — independent path; runs in relation to dash system
6. `update_previous_dash_state` (breaker domain) — runs last, after all transition consumers

Steps 2 and 3 operate on the same message (`BoltLost`) — independent, can run in either order.

## Tests

New tests:

- `breaker/systems/handle_bolt_lost/tests/life_loss.rs` — `BoltLossBehavior::LifeLoss(1)` + `Hp` decrements correctly; clamps at zero.
- `breaker/systems/handle_bolt_lost/tests/life_loss_no_hp.rs` — `LifeLoss(1)` on a breaker without `Hp` is a no-op.
- `breaker/systems/handle_bolt_lost/tests/time_loss.rs` — `BoltLossBehavior::TimeLoss(5.0)` writes `ReduceNodeTimer { delta: 5.0 }`.
- `breaker/systems/handle_bolt_lost/tests/none_noop.rs` — `BoltLossBehavior::None` writes nothing, modifies nothing.
- `mutators/protocols/reckless_dash/tests/double_penalty_on_dash.rs` — enter `DashState::Dashing` → behavior becomes doubled, `OriginalBoltLossBehavior` overlay inserted.
- `mutators/protocols/reckless_dash/tests/restore_on_dash_exit.rs` — transition from `Dashing` → `Settling` → behavior restored, overlay removed.
- `mutators/protocols/reckless_dash/tests/double_only_while_dashing.rs` — BoltLost during Idle → base penalty. BoltLost during Dashing → doubled penalty. BoltLost during Settling (post-dash) → base penalty.
- `mutators/protocols/reckless_dash/tests/time_loss_doubles_correctly.rs` — breaker with `TimeLoss(5.0)` → during dash it's `TimeLoss(10.0)`; `handle_bolt_lost` writes `ReduceNodeTimer { delta: 10.0 }`.
- `breaker/systems/update_previous_dash_state/tests/updates_each_tick.rs` — `PreviousDashState` tracks previous tick's `DashState`.

Existing tests to update:
- `state/run/node/systems/apply_time_penalty.rs` tests → renamed files, renamed type, renamed field.
- `state/run/node/systems/reverse_time_penalty.rs` tests → renamed files, renamed type, renamed field.
- `hazard/hazards/decay.rs` tests → rename `ApplyTimePenalty` → `ReduceNodeTimer` + field rename.
- `protocol/protocols/siphon/` tests → rename `ReverseTimePenalty` → `IncreaseNodeTimer` + field rename.
- `effect_v3/effects/time_penalty/config.rs` tests → updated to verify `fire` writes `ReduceNodeTimer` message instead of direct mutation.

Existing tests to remove:
- Any test that loads or exercises Prism.

## Migration plan

Land as a feature branch; each step is a TDD-style commit.

1. **Message renames sweep.** `ApplyTimePenalty` → `ReduceNodeTimer`, `ReverseTimePenalty` → `IncreaseNodeTimer`, field `seconds` → `delta`, system renames, SystemSet variant rename. Mechanical rename across 13 files. Tests still pass (pure rename).
2. **`TimePenaltyConfig::fire` unification.** Change `fire` to write `ReduceNodeTimer` message instead of direct `NodeTimer` mutation. Test that `fire` writes the message; removal of direct mutation is a behavior change — verify `apply_reduce_node_timer` still runs and that the final `NodeTimer` state matches.
3. **Introduce `BoltLossBehavior` enum + component.** Add to `breaker/components.rs`. Add `bolt_loss_behavior: BoltLossBehavior` field to `BreakerDefinition`. Default `LifeLoss(1)`.
4. **Write `handle_bolt_lost` system.** Register in `BreakerPlugin`. Tests for each variant.
5. **Migrate Aegis and Chrono RON files.** Drop `bolt_lost: RootNode`. Add `bolt_loss_behavior: LifeLoss(1)` / `TimeLoss(5.0)`. Keep `salvo_hit` as-is.
6. **Delete Prism.** Remove RON, UI references, tests, any bolt-default.
7. **Update Godmode scenario-runner breaker** — explicit `BoltLossBehavior::None`.
8. **Delete the `bolt_lost: RootNode` field** from `BreakerDefinition`. Remove the default-tree code that used `LoseLifeConfig` for bolt-loss. `on_bolt_lost_occurred` bridge still dispatches trigger for other entity-bound effects; no change there.
9. **Introduce `PreviousDashState` component and `update_previous_dash_state` system.** Register in BreakerPlugin.
10. **Write Reckless Dash transition system** — `reckless_dash_on_dash_transition`. Register under its `wire(app)` function per #2's mutators-domain pattern.
11. **Remove the old Reckless Dash double-penalty implementation** (the duplicate-emit / anti-feedback guard / `RecklessDashDoubledBolts` resource if present).
12. **Full Verification Tier** — scenarios pass, no Hp/timer drift, run-lifecycle invariants intact.

## Scope boundary

In scope:
- `BoltLossBehavior` enum + component + RON field
- `handle_bolt_lost` system in breaker domain
- Message rename (`ReduceNodeTimer`, `IncreaseNodeTimer`) + field rename
- `TimePenaltyConfig::fire` unification (write message instead of direct mutation)
- Reckless Dash transition-detection + mutate/restore
- Prism breaker removal
- Godmode breaker explicit declaration

Out of scope:
- Salvo-hit changes (remains effect-tree-driven)
- `LoseLifeConfig` retirement (stays for salvo_hit + future uses)
- Effect-system trigger bridge changes
- Changes to `BoltLost` message payload (stays `{ bolt, breaker }`)
- Any mutator chain or SystemSet triplet for BoltLost (rejected earlier — component-mutation is simpler)

## Subsumed remediations

This remediation fully subsumes:
- `rename-reverse-time-penalty-to-time-bonus.md` — the rename destination changes (`IncreaseNodeTimer` not `TimeBonus`) and the sweep scope expands to include `ApplyTimePenalty` → `ReduceNodeTimer`.
- Part of `breaker-bolt-lost-effect-component.md` — the "new component on breaker for bolt-lost penalty" idea is realized here via `BoltLossBehavior`, but the `MessageMutator<BoltLost>` approach and BoltLifecycleSystems triplet are rejected in favor of the simpler component-mutation pattern. That remediation file can be deleted.

Counts two more toward the "fully subsumed" tally (previously 7 → after this lands, 9 total, and a partial scope adjustment for the rename remediation).

## TODO entry

> **[BLOCKED by #1, #2, #3]** Bolt-loss behavior — `BoltLossBehavior` enum component replacing the `bolt_lost` effect-tree field on `BreakerDefinition`; centralized `handle_bolt_lost` system in breaker domain; message renames (`ApplyTimePenalty` → `ReduceNodeTimer`, `ReverseTimePenalty` → `IncreaseNodeTimer`, field `seconds` → `delta`); Reckless Dash mutate-on-dash-enter / restore-on-dash-exit via `PreviousDashState` + `Changed<DashState>`; Prism breaker retired; Godmode scenario-runner breaker gets explicit `None` variant. Subsumes `rename-reverse-time-penalty-to-time-bonus.md` and the component-on-breaker portion of `breaker-bolt-lost-effect-component.md`. — [detail](detail/bolt-loss-behavior.md)
