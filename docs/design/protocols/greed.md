# Protocol: Greed

## Category
`custom-system`

## Game Design
You WANT to skip chip offerings, gambling on better chips later.

- Chip-select screen gains a "Skip" action while Greed is active.
- Each skip increases the probability of higher-rarity chips in subsequent offerings.
- No immediate power gain — pure gamble: trade certain power now for uncertain better power later.
- Stacks per skip.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct GreedConfig {
    pub rarity_boost_per_skip: f32,     // percent, e.g. 5.0 = +5% per skip
}
```

## Components
```rust
#[derive(Resource, Debug, Default)]
pub(crate) struct GreedStacks {
    pub skips: u32,
}

impl GreedStacks {
    #[must_use]
    pub fn rarity_boost(&self, config: &GreedConfig) -> f32 {
        self.skips as f32 * config.rarity_boost_per_skip
    }
}
```

`GreedStacks` is a resource (run-scoped state), not a per-entity component.

## Messages
**Reads**: `ChipOfferSkipped` (new message, owned by `state/run/chip_select`).
**Sends**: None.

## Systems

### `greed_on_skip`
- **Schedule**: `Update`. Wired ungated (no `run_if`); enforces the `ActiveProtocols` gate in-body via `reader.clear()` + return when Greed is inactive.
- **Behavior**: Reads `ChipOfferSkipped`. Increments `GreedStacks.skips` by 1. Does NOT close the chip-select screen — that is handled by `handle_chip_input`'s `SelectionRow::Skip` confirm arm, which calls `state_writer.write(ChangeState::new())` before emitting the message.

### `greed_modify_rarity_weights` (not a system — a read from chip-select)
- The chip-select domain's `generate_chip_offerings` system reads `Option<Res<GreedStacks>>` + `Option<Res<GreedConfig>>` when rolling rarity. If present, it shifts the distribution: for each skip, higher-rarity probabilities go up by `rarity_boost_per_skip`%. This is a **read** by chip-select of Greed's resource — allowed.

### Cleanup
- `GreedStacks` is cleared by `reset_run_state` as part of `RunInventories::clear_all`. Does not carry across runs. There is no separate `greed_cleanup_run` system.

### UI integration
- The chip-select screen renders a Skip row (button marker + indicator text) when Greed is active (conditional spawn reads `Res<ActiveProtocols>`).
- The Skip row is navigated via keyboard: `handle_chip_input` gains a `SelectionRow::Skip` arm; pressing confirm while focused on the Skip row emits `ChipOfferSkipped`. No separate click pipeline is used — the chip-select screen is keyboard-only.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Greed does not participate in any `DeathPipelineSystems` set.
- **Trigger**: `ChipOfferSkipped` message.
- **Affects**: chip-select rarity-weight rolls only.
- **No** `DamageDealt<T>` / `HealDealt<T>` / `Destroyed<T>` / `DamageBoostStack` involvement.

## Cross-Domain Dependencies
- **state/run/chip_select**: Reads `Res<GreedStacks>` + `Res<GreedConfig>` when rolling chip-offering rarity. UI reads `Res<ActiveProtocols>` to show/hide Skip button.
- **chips**: No direct interaction. Greed affects rarity rolls, not chip pools.

## Expected Behaviors (for test specs)

1. **Skip increments stacks** — `skips = 0`, `ChipOfferSkipped`: `skips = 1`.
2. **Multiple skips accumulate** — `skips = 2`, `ChipOfferSkipped`: `skips = 3`.
3. **Rarity boost computed correctly** — `skips = 3`, `rarity_boost_per_skip = 5.0`: `rarity_boost() = 15.0`.
4. **Chip offering uses boost** — base rarity Common 60 / Uncommon 25 / Rare 15; `skips = 2`, `rarity_boost_per_skip = 5.0`: Common shifts down, Uncommon + Rare up, by the boost amount (exact redistribution is a tuning decision — suggested: subtract from Common, distribute to higher tiers).
5. **Skip closes chip-select screen** — skip transitions chip-select state to done (no chip added to inventory).
6. **No Skip option without Greed** — `protocol_active(Greed) = false`: chip-select UI does not show Skip.
7. **Stacks persist across nodes within a run** — node ends, new node: `GreedStacks` unchanged.
8. **Stacks cleared on run end** — run ends: `GreedStacks::default()`.

## Edge Cases
- **Boost cap**: Common weight shouldn't drop below a floor (e.g., 10%). At `rarity_boost_per_skip = 5.0`, this caps benefit at ~10 skips.
- **Skipping when higher rarities have been exhausted**: the offering system draws from available chips; the boost has no effect if no higher-rarity chips remain in the pool.
- **Interaction with Evolution rarity**: Evolutions are gated by chip prerequisites, not rarity rolls. Greed's boost doesn't affect evolutions.
- **Protocol offering**: Greed only affects chip offerings, not protocol offerings (separate flow).
- **Greed + Tier Regression**: each regressed-tier chip offering can be skipped, compounding the rarity boost faster.
