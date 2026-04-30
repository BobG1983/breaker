# Protocol: Reckless Dash

## Category
`custom-system`

## Game Design
You WANT to be out of position and stretch your dash to the limit for a risky catch.

- "Risky catch" = bolt contacts breaker in the last portion of dash distance (stretched to reach it — you weren't in position).
- Risky catch: next cell impact deals multiplied damage (default 4x).
- If a bolt is lost while dashing: the breaker's `BoltLossBehavior` is already doubled for the entire duration of the dash, so the standard loss handler applies the doubled penalty automatically.
- Non-risky catch (bolt contacts in early/mid dash, or while stationary): normal bump — no bonus, no penalty.
- Telegraph: visual indicator of the "risky zone" near the bottom (Phase 5 VFX).

## Config Resource
```rust
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub(crate) struct RecklessDashConfig {
    pub risky_zone_start: f32,      // 0.7 means last 30%
    pub damage_multiplier: f32,     // 4.0
    pub double_penalty: bool,       // true
}
```

## Components

Owned by Reckless Dash:

- **`RiskyDamageBoost { multiplier: f32 }`** — per-bolt single-shot marker. Inserted on risky catch by `reckless_dash_on_bump`; consumed and removed by `reckless_dash_amplify_damage` on the bolt's next cell impact (one-shot: only one amplified `DamageDealt<Cell>` is emitted per boost even on pierce-through frames).

- **`OriginalBoltLossBehavior(pub BoltLossBehavior)`** — per-breaker overlay storing the unmodified `BoltLossBehavior` from before dash entry. Inserted on `Idle → Dashing` transition; removed on `Dashing → {anything else}` or on `OnExit(NodeState::Playing)`. Allows restoration of the original penalty when the dash ends.

## Messages
**Reads**: `BumpPerformed { grade, bolt, breaker }` (breaker domain), `BoltImpactCell { bolt, cell }` (bolt domain).

**Sends**: `DamageDealt<Cell>` — emitted directly by `reckless_dash_amplify_damage` with `source: Some(SourceId::protocol(RecklessDash).build())` and `amount = base_damage * boost.multiplier`. This is NOT routed through `DamageBoostStack`.

## Systems

### `reckless_dash_on_bump`
- **Schedule**: `FixedUpdate`, `.after(BreakerSystems::GradeBump)`.
- **run_if**: Harness-safe drain — clears reader and returns if `ProtocolGate` is closed for `RecklessDash` OR `RecklessDashConfig` is absent. Does not use an external `run_if` gate (reader drain prevents message leaks across frames).
- **Behavior**: Consumes `BumpPerformed`. For each bump where the bolt exists, checks if the breaker's `DashState` is `Dashing` AND `progress > risky_zone_start` (strictly greater). Progress is `(duration - remaining) / duration`. If risky: inserts `RiskyDamageBoost { multiplier: config.damage_multiplier }` on the bolt entity.

### `reckless_dash_amplify_damage`
- **Schedule**: `FixedUpdate`, `.after(BoltSystems::CellCollision).in_set(DmgSystems::EmitDamage)`.
- **run_if**: Harness-safe drain — clears reader and returns if gate is closed or config absent. Boost is NOT consumed on the early-return drain path.
- **Behavior**: Consumes `BoltImpactCell`. For each bolt carrying `RiskyDamageBoost`, emits `DamageDealt<Cell>` with `amount = base_damage * boost.multiplier` (gated: only emits when `amount > 0.0`). Removes the `RiskyDamageBoost` component unconditionally (single-shot, even if amount guard suppresses emission). Pierce guard: tracks `amplified_this_frame` so only one amplified emit is produced per boost per invocation.

### `reckless_dash_on_dash_transition`
- **Schedule**: `FixedUpdate`, `.after(BreakerSystems::UpdateState).before(BreakerSystems::UpdatePreviousState).before(BreakerSystems::HandleBoltLost)`.
- **run_if**: `protocol_active(ProtocolKind::RecklessDash)` + `in_state(NodeState::Playing)`. (No reader to drain — gated externally.)
- **Query filter**: `Changed<DashState>` guards iteration cost; actual enter/exit detection uses `PreviousDashState` vs current `DashState` to avoid acting on same-value writes.
- **Behavior**:
  - `{not Dashing} → Dashing`: saves the live `BoltLossBehavior` as `OriginalBoltLossBehavior`, then doubles it (`LifeLoss(n) → LifeLoss(n.saturating_mul(2))`, `TimeLoss(d) → TimeLoss(d * 2.0)`, `None → None`). Skips when `config.double_penalty == false`.
  - `Dashing → {anything else}`: restores `BoltLossBehavior` from the overlay (when present) and removes `OriginalBoltLossBehavior`. Safe no-op when overlay is absent.
  - Same-value writes (`Idle → Idle`, `Dashing → Dashing`) are no-ops.

### `reckless_dash_cleanup_node`
- **Schedule**: `OnExit(NodeState::Playing)`.
- **run_if**: None — runs unconditionally.
- **Behavior**: For every breaker carrying `OriginalBoltLossBehavior` (i.e., exited `Playing` while still dashing), restores `BoltLossBehavior` to the saved original and removes the overlay. Idle breakers (no overlay) are naturally skipped by the query filter.

## Pipeline position (dmg crate)

- **Risky-catch trigger**: `BumpPerformed` consumed by `reckless_dash_on_bump`; `RiskyDamageBoost` inserted on the bolt.
- **Damage emission**: `reckless_dash_amplify_damage` emits `DamageDealt<Cell>` directly in `DmgSystems::EmitDamage`. The standard `DmgSystems` chain then applies and resolves it.
- **Bolt-loss double-penalty path**: `BoltLossBehavior` on the breaker is already doubled for the entire dash duration by `reckless_dash_on_dash_transition`. The standard `BoltLossBehavior` handler fires once, using the doubled value. No second invocation.
- **No `DamageBoostStack` usage** — Reckless Dash emits `DamageDealt<Cell>` directly rather than stacking a one-shot multiplier on the bolt.

## Cross-Domain Dependencies
- **breaker**: Reads `BumpPerformed`, `DashState`, `PreviousDashState`. Writes `BoltLossBehavior` (cross-domain write exception — see Site A comment in source). Inserts/removes `OriginalBoltLossBehavior` (owned by Reckless Dash).
- **bolt**: Reads `BoltImpactCell`. Reads `BoltBaseDamage`. Inserts/removes `RiskyDamageBoost` (owned by Reckless Dash).
- **damage crate (`rantzsoft_dmg`)**: `DmgSystems::EmitDamage` — `reckless_dash_amplify_damage` lives in this set so emissions accumulate before the chain flushes.

## Expected Behaviors (for test specs)

1. **Risky catch inserts `RiskyDamageBoost`** — `risky_zone_start: 0.7`, dash progress 80%: `BumpPerformed` → `RiskyDamageBoost { multiplier: 4.0 }` inserted on the bolt.
2. **Non-risky catch (early/mid dash) inserts nothing** — dash progress 50%: no `RiskyDamageBoost`.
3. **Stationary catch inserts nothing** — `DashState::Idle`: no `RiskyDamageBoost`.
4. **Boost consumed on first cell impact** — bolt carries `RiskyDamageBoost { multiplier: 4.0 }`, `BoltBaseDamage = 10`: `reckless_dash_amplify_damage` emits `DamageDealt<Cell> { amount: 40.0 }`; boost removed.
5. **Boost does NOT persist to second impact** — subsequent `BoltImpactCell` for same bolt: no amplified emit (boost already removed).
6. **Double-penalty active during dash** — `double_penalty: true`, `Idle → Dashing` transition: `BoltLossBehavior` doubled; `OriginalBoltLossBehavior` overlay inserted.
7. **Penalty restored on dash exit** — `Dashing → Idle`: `BoltLossBehavior` restored from overlay; overlay removed.
8. **Bolt-lost during dash uses doubled penalty** — bolt-loss handler reads the already-doubled `BoltLossBehavior`; standard single invocation.
9. **Bolt-lost while stationary uses original penalty** — `DashState::Idle`: `BoltLossBehavior` not doubled; single invocation with original value.
10. **`double_penalty: false` skips mutation** — `Idle → Dashing`: `BoltLossBehavior` unchanged; no overlay inserted.
11. **Node exit while dashing restores penalty** — `OnExit(NodeState::Playing)` with overlay present: `BoltLossBehavior` restored; overlay removed.

## Edge Cases
- **Risky zone boundary**: bolt contacts at exactly `risky_zone_start` (e.g. 70%) — does NOT trigger (strictly greater than). `progress <= risky_zone_start` is non-risky.
- **Multiple bolts**: each bolt tracks its own `RiskyDamageBoost`. Risky catch on bolt A doesn't affect bolt B.
- **Pierce-through**: multiple `BoltImpactCell` messages for the same bolt in one frame — only the first amplified emit is produced; subsequent ones are no-ops for that bolt.
- **`double_penalty: false`**: dash transitions do not mutate `BoltLossBehavior`; no overlay inserted.
- **Dash ends between bump and cell impact**: `RiskyDamageBoost` persists on the bolt regardless of dash state change — it was earned at bump time.
- **Bolt has `RiskyDamageBoost` but is lost before cell impact**: boost disappears with the bolt entity.
- **Same-value `DashState` write**: `Changed<DashState>` triggers but `PreviousDashState == current` — transition systems treat this as a no-op.
