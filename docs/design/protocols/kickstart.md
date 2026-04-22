# Protocol: Kickstart

## Category
`code-driven` (per TODO #7 — previously `effect-tree`; now dispatched from code).

## Game Design
You WANT to optimize explosive openers.

Each node opens with 3s of 2x bolt speed + 2x damage + Piercing(2). Timer starts on first bump.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct KickstartConfig {
    pub window_duration: f32,       // 3.0
    pub speed_multiplier: f32,      // 2.0
    pub damage_multiplier: f32,     // 2.0
    pub piercing_amount: u32,       // 2
}
```

Populated from `ProtocolTuning::Kickstart`. RON carries tuning only — no effect tree.

## Components
None owned by Kickstart. State is a per-run flag (`kickstart_armed`) + a per-node timer resource.

```rust
#[derive(Resource, Debug, Default)]
pub(crate) struct KickstartWindow {
    pub state: KickstartWindowState,
}

pub(crate) enum KickstartWindowState {
    #[default] NotActive,          // before node starts
    Armed,                         // node started, awaiting first bump
    Running(f32),                  // countdown active (seconds remaining)
    Expired,                       // countdown complete
}
```

## Messages
**Reads**: `OnEnter(NodeState::Playing)` state-change (drives arm). `BumpPerformed` (starts countdown on first bump).
**Sends**: None. Effects dispatched through `commands.fire_effect(bolt, ..., "protocol:kickstart")` when the window opens; `commands.reverse_effect(..., "protocol:kickstart")` when expires.

## Systems

### `kickstart_arm_on_node_enter`
- **Schedule**: `OnEnter(NodeState::Playing)`.
- **run_if**: `protocol_active(ProtocolKind::Kickstart)`.
- **Behavior**: For each active bolt: fires `SpeedBoost(speed_multiplier)`, `DamageBoost(damage_multiplier)`, `Piercing(piercing_amount)` via `commands.fire_effect` with source `"protocol:kickstart"`. Sets `KickstartWindow.state = Armed`. Registers with `SpawnStampRegistry` so bolts spawned before the first bump also get the effects.

### `kickstart_start_countdown_on_first_bump`
- **Schedule**: `FixedUpdate`, `.after(BreakerSystems::GradeBump)`.
- **run_if**: `protocol_active(ProtocolKind::Kickstart)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `BumpPerformed`. If `state == Armed`: transition to `Running(config.window_duration)`.

### `kickstart_tick_countdown`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::Kickstart)` + `in_state(NodeState::Playing)`.
- **Behavior**: If `state == Running(t)`: `t -= delta_secs`. If `t <= 0.0`: reverse all `"protocol:kickstart"`-sourced effects from every bolt (`commands.reverse_all_by_source(world, "protocol:kickstart")`); transition to `Expired`.

### `kickstart_cleanup_node`
- **Schedule**: `OnExit(NodeState::Playing)`.
- **Behavior**: Reverses any lingering `"protocol:kickstart"` entries; resets `KickstartWindow.state = NotActive`.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Kickstart does not participate in any `DeathPipelineSystems` set.
- **Trigger**: `OnEnter(NodeState::Playing)` (arm) + `BumpPerformed` (start countdown).
- **Writes**: source-tagged entries on bolts: `EffectStack<SpeedBoostConfig>` (speed), `DamageBoostStack::persistent` (damage), piercing-stack (piercing). All via `commands.fire_effect` / `commands.reverse_effect`.
- **No** direct `DamageDealt<T>` emission.

## Cross-Domain Dependencies
- **bolt**: Writes stack entries via fire/reverse commands on every active bolt. Piercing affects cell collision.
- **run/node**: Reads `NodeState` transitions.
- **breaker**: Reads `BumpPerformed`.
- **effect_v3**: Uses `fire_effect`, `reverse_effect`, `reverse_all_by_source`, `SpawnStampRegistry`.

## Expected Behaviors (for test specs)

1. **Bolt speed doubles at node start** — `speed_multiplier = 2.0`, bolt speed 400: at `OnEnter(Playing)`, bolt speed 800 (via stack aggregation).
2. **Bolt damage doubles at node start** — `damage_multiplier = 2.0`, bolt base damage 10: next `DamageDealt<Cell>` emits 20.
3. **Bolt gains Piercing(2) at node start** — bolt has `Piercing(2)` from `"protocol:kickstart"` source.
4. **Countdown starts on first bump** — state `Armed` + `BumpPerformed`: state `Running(window_duration)`.
5. **Effects removed after countdown** — `Running(3.0)`, 3.0s elapse: state `Expired`; all `"protocol:kickstart"` stack entries reversed.
6. **Effects persist indefinitely until first bump** — state `Armed`, no bump for 10s: effects still active.
7. **Each node gets a fresh Kickstart window** — `OnEnter(Playing)` re-arms and re-fires effects on the next node.
8. **Stacks multiplicatively with chip effects** — existing `DamageBoost(1.5)` + Kickstart `DamageBoost(2.0)`: effective `10 * 1.5 * 2.0 = 30`.

## Edge Cases
- **Multiple bolts**: all bolts receive effects at arm time. Countdown is shared (starts on first bump, not per-bolt). Expiry reverses for all.
- **Bolt spawned during Kickstart window** (e.g., Fission): `SpawnStampRegistry` applies Kickstart effects on spawn. New bolts also reverse on expiry.
- **Bolt-lost during Kickstart**: lost bolt's entries die with it; remaining bolts keep their effects.
- **Node-end before countdown expires**: all effects reversed at `OnExit(Playing)`.
- **Piercing stacks additively** with chip-granted piercing.
- **Bump whiff**: a whiff is not `BumpPerformed` (no grade) — does NOT start the countdown.
- **Interaction with Deadline**: both can be active; rarely overlap (Kickstart in first 3s; Deadline below 25% timer). If they overlap, effects stack multiplicatively.
