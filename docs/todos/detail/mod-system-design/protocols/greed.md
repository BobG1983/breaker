# Protocol: Greed

## Category
`custom-system`

## Game Design
You WANT to skip chip offerings, gambling on better chips later.

- On chip offering screen: option to skip (take no chip).
- Each skip increases the chance of higher rarity chips in future offerings (e.g., +5% rarity boost per skip, tunable).
- No immediate power gain. Pure gamble — trading certain power now for uncertain better power later.
- Stacks per skip.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct GreedConfig {
    /// Percentage boost to higher-rarity chip probability per skip.
    /// e.g., 5.0 means +5% per skip.
    pub rarity_boost_per_skip: f32,
}
```

## Components
```rust
/// Tracks accumulated rarity boost from skipped chip offerings.
/// Global resource — persists for the entire run.
#[derive(Resource, Debug, Default)]
pub(crate) struct GreedStacks {
    /// Number of times the player has skipped a chip offering this run.
    pub skips: u32,
}

impl GreedStacks {
    /// Total rarity boost percentage based on accumulated skips.
    #[must_use]
    pub fn rarity_boost(&self, config: &GreedConfig) -> f32 {
        self.skips as f32 * config.rarity_boost_per_skip
    }
}
```

## Messages
**Reads**: A new `ChipOfferSkipped` message (or repurpose existing chip select flow to include a "skip" option)
**Sends**: None (modifies the rarity calculation input, not a direct message)

**Note on message design**: The chip select UI already sends `ChipSelected { name }` when a chip is picked. Greed adds a "skip" action to the chip select screen. This could be:
- Option A: A new `ChipOfferSkipped` message sent by the UI, consumed by the protocol domain.
- Option B: A sentinel value in `ChipSelected` (e.g., `ChipSelected { name: "__skip__" }`) — less clean.
- **Recommended**: Option A. New message `ChipOfferSkipped`, owned by the protocol domain (or chip_select subdomain). Clean separation.

```rust
#[derive(Message, Clone, Debug)]
pub struct ChipOfferSkipped;
```

## Systems

### `greed_on_skip`
- **Schedule**: `Update`
- **Run if**: `protocol_active(ProtocolKind::Greed)` + `in_state(ChipSelectState::Selecting)`
- **What it does**: Reads `ChipOfferSkipped` messages. Increments `GreedStacks.skips` by 1. Triggers chip select screen close (same state transition as picking a chip).
- **Ordering**: During chip select phase. Runs alongside `handle_chip_input`.

### `greed_modify_rarity_weights`
- **Schedule**: Not a runtime system — this is a **modifier** applied during chip offering generation.
- **What it does**: The `generate_chip_offerings` system (in `state/run/chip_select/`) reads `GreedStacks` and `GreedConfig` when rolling rarity for chip offers. The rarity boost shifts the probability distribution: for each skip, higher rarities become `rarity_boost_per_skip`% more likely.
- **Integration point**: `generate_chip_offerings` checks if `GreedStacks` resource exists, and if so, applies the boost to its rarity roll. This is a cross-domain read (protocol resource read by chip_select), which is allowed.

### Rarity Boost Application
The existing chip offering system uses weighted rarity probabilities (e.g., Common: 60%, Uncommon: 25%, Rare: 15%). Greed modifies these weights:

```
let boost = greed_stacks.rarity_boost(&greed_config);
// Shift weight from Common to higher rarities proportionally
// e.g., with 10% boost: Common drops from 60% to 50%, 
// Uncommon goes from 25% to 30%, Rare from 15% to 20%
```

The exact redistribution formula is a tuning decision. The simplest approach: subtract `boost` from Common weight and distribute evenly across Uncommon and Rare. Cap at reasonable limits (Common never drops below some floor, e.g., 10%).

### `greed_cleanup_run`
- **Schedule**: Run end cleanup (alongside `reset_run_state`)
- **What it does**: Resets `GreedStacks` to default. Skips do not carry across runs.

### UI Integration
- **Chip select screen**: When Greed is active, display a "Skip" button/option alongside the chip offerings. The UI sends `ChipOfferSkipped` when the player selects skip.
- **Visual feedback**: Display current skip count and rarity boost somewhere on the chip select screen (e.g., "Greed: 3 skips (+15% rarity)").

## Cross-Domain Dependencies
- **state/run/chip_select**: Greed modifies the chip offering generation. `generate_chip_offerings` reads `Res<GreedStacks>` and `Res<GreedConfig>` to adjust rarity weights. The chip select UI reads `Res<ActiveProtocols>` to know whether to show the skip option.
- **ui**: The chip select screen needs a skip button when Greed is active.
- **chips**: No direct interaction with the chips domain. Greed affects the rarity distribution of offerings, not the chips themselves.

## Expected Behaviors (for test specs)

1. **Skip increments greed stacks**
   - Given: Greed active, `GreedStacks { skips: 0 }`
   - When: `ChipOfferSkipped` is sent
   - Then: `GreedStacks { skips: 1 }`

2. **Multiple skips accumulate**
   - Given: `GreedStacks { skips: 2 }`
   - When: `ChipOfferSkipped` is sent
   - Then: `GreedStacks { skips: 3 }`

3. **Rarity boost calculated correctly**
   - Given: `GreedStacks { skips: 3 }`, `GreedConfig { rarity_boost_per_skip: 5.0 }`
   - When: `rarity_boost()` is called
   - Then: Returns `15.0` (3 * 5.0)

4. **Chip offering generation uses rarity boost**
   - Given: Base rarity weights: Common 60%, Uncommon 25%, Rare 15%. `GreedStacks { skips: 2 }`, `rarity_boost_per_skip: 5.0` (total boost: 10%)
   - When: `generate_chip_offerings` runs
   - Then: Rarity weights are shifted — Common probability reduced, Uncommon and Rare increased by the boost amount.

5. **Skip closes chip select screen**
   - Given: Greed active, chip select screen open
   - When: Player skips (sends `ChipOfferSkipped`)
   - Then: Chip select state transitions to done (same flow as selecting a chip). No chip is added to inventory.

6. **No skip option without Greed**
   - Given: Greed NOT active
   - When: Chip select screen opens
   - Then: No skip option displayed. Normal chip selection only.

7. **Stacks persist across nodes within a run**
   - Given: `GreedStacks { skips: 3 }`, node ends, new node starts
   - When: Next chip offering screen opens
   - Then: `GreedStacks` still has `skips: 3`. Rarity boost is cumulative across the whole run.

8. **Stacks cleared on run end**
   - Given: `GreedStacks { skips: 5 }`
   - When: Run ends (win or lose)
   - Then: `GreedStacks` reset to default (skips: 0).

9. **Zero skips means no rarity modification**
   - Given: `GreedStacks { skips: 0 }`, `rarity_boost_per_skip: 5.0`
   - When: `rarity_boost()` is called
   - Then: Returns `0.0`. Chip offerings use unmodified rarity weights.

## Edge Cases
- **Rarity boost exceeding 100%**: Cap the total boost so Common weight never drops below a floor (e.g., 10%). At `rarity_boost_per_skip: 5.0`, this means 10 skips maxes out the benefit. Beyond that, skipping still increments the counter but has diminishing/no returns on rarity.
- **Skipping when only Common chips available**: The boost makes higher rarities more likely, but if the chip pool has been exhausted for higher rarities, the boost has no effect. The offering system generates from available chips, not hypothetical ones.
- **Interaction with Evolution rarity**: Evolutions are a separate rarity tier. Greed's boost applies to Common/Uncommon/Rare only. Evolution availability is governed by chip prerequisites, not rarity rolls.
- **Skipping the protocol offering itself**: Greed only affects chip offerings. The protocol offering screen is a separate flow and has no skip mechanic (unless another protocol adds one).
- **Greed + Tier Regression**: Tier Regression gives extra chip offerings. Each offering can be skipped, compounding the rarity boost faster. Strong synergy.
