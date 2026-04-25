# Greed skip button — UI for the chip-offer skip action

## Problem

Greed is a protocol that rewards skipping chip offerings: each skip decays the offered chips (no chip that round) and stacks a rarity boost for future chip offers. The mechanic is live in code — `GreedConfig`, `GreedStacks`, `greed_on_skip`, and `apply_greed_boost` all exist in `breaker-game/src/mutators/protocols/protocols/greed/system.rs` — and the `ChipOfferSkipped` message is defined in `state/run/chip_select/messages.rs`. The chip-decay consequence of skipping is already wired in `handle_chip_input.rs:130`.

What's missing: the UI. There is no way for the player to actually trigger a skip. No button, no keybind, no indicator. The mechanic is unreachable from normal gameplay.

## Design

Greed gates the ability to skip. Without Greed active, the skip button is NOT shown — skipping isn't an option in the base game. With Greed active, the button is visible on every chip-select screen and the player can gamble: pass on the current chips (decay them to nothing, take no chip this round) in exchange for stacked rarity odds on the next offer.

### Scope

Lives entirely in `breaker-game/src/state/run/chip_select/`. No mutator-domain changes. No protocol-domain changes. The existing `greed_on_skip` consumer (protocol domain) and the chip-decay path (state domain) stay exactly as they are.

### Components

Add `SkipButton` component to `state/run/chip_select/components.rs`:

```rust
/// Marker for the Greed skip button on the chip-select screen.
/// Spawned conditionally by `spawn_chip_select` when ActiveProtocols
/// contains Greed; removed automatically with the rest of the screen
/// on `OnExit(ChipSelectState)`.
#[derive(Component, Debug)]
pub(crate) struct SkipButton;
```

If a skip indicator (counter / next-offer rarity boost) is implemented, add a matching marker:

```rust
/// Marker for the skip indicator label next to the SkipButton.
/// Shows current GreedStacks.skips and/or next-offer rarity boost.
#[derive(Component, Debug)]
pub(crate) struct SkipIndicator;
```

### Conditional spawn

In `state/run/chip_select/systems/spawn_chip_select/system.rs`, after spawning the chip row and protocol row, check `ActiveProtocols`:

```rust
if active_protocols.contains(ProtocolKind::Greed) {
    spawn_skip_button(parent, &config);
    spawn_skip_indicator(parent, &config, &greed_stacks);
}
```

`active_protocols: Res<ActiveProtocols>` is a new system param. `greed_stacks: Res<GreedStacks>` is needed only if the indicator renders the counter.

The button + indicator are children of `ChipSelectScreen` (same pattern as `ChipCard` and `ProtocolCard`), so they're torn down automatically by the existing `OnExit(ChipSelectState)` cleanup.

### Input handler

New system at `state/run/chip_select/systems/handle_skip_input.rs`:

```rust
pub(crate) fn handle_skip_input(
    mut reader: /* pointer / keyboard input reader */,
    skip_button: Query<Entity, With<SkipButton>>,
    mut writer: MessageWriter<ChipOfferSkipped>,
) {
    // On click or hotkey press targeting SkipButton entity:
    //   writer.write(ChipOfferSkipped);
    // ChipOfferSkipped is unit-payload per messages.rs doc.
}
```

Registered via `chip_select` plugin's system setup; runs in the same set as `handle_chip_input` (both are input handlers on the chip-select screen). The skip input does NOT transition out of `ChipSelectState` — that's driven by the existing chip-selection flow (which already handles the "no chip picked" case through chip decay).

Running alongside `handle_chip_input`: the two handlers consume separate input paths. If the player clicks a chip card, `handle_chip_input` emits `ChipSelected`; if they click the skip button, `handle_skip_input` emits `ChipOfferSkipped`. Both close the screen; the existing downstream flow distinguishes them.

### Indicator display

Text label rendered near the button. Content options:

- **`Skips: N`** — simplest. Shows the running skip counter from `GreedStacks.skips`.
- **`Next offer: +X% rarity`** — shows the computed rarity boost for the next offer (`GreedStacks.rarity_boost(config)`).
- **Both** — `Skips: 3 (+15% next)`.

Recommend BOTH — it tells the player what they've accumulated AND what it means. Updated on screen spawn (static for the duration of the chip-select screen; the value only changes after the player commits a skip and the screen closes).

No per-tick update system needed — the indicator is set at spawn time from the current `GreedStacks`, displays through the screen's lifetime, and is respawned with fresh values the next time the chip-select screen opens.

### Tests

Add `state/run/chip_select/systems/spawn_chip_select/tests.rs` cases (or a new dedicated test file):

1. **`skip_button_not_spawned_when_greed_inactive`**
   - Given: chip-select screen spawns, `ActiveProtocols` does NOT contain Greed.
   - Then: no entity with `SkipButton` exists.

2. **`skip_button_spawned_when_greed_active`**
   - Given: `ActiveProtocols` contains `ProtocolKind::Greed`.
   - When: chip-select screen spawns.
   - Then: exactly one `SkipButton` entity exists.

3. **`skip_indicator_reflects_current_stacks`**
   - Given: `GreedStacks { skips: 3 }`, `GreedConfig { rarity_boost_per_skip: 5.0 }`, Greed active.
   - When: chip-select screen spawns.
   - Then: the `SkipIndicator`'s text contains both `"3"` (or equivalent) and `"+15%"` (or computed value).

4. **`pressing_skip_emits_chip_offer_skipped`** (in `handle_skip_input/tests.rs`)
   - Given: chip-select screen with SkipButton spawned, Greed active.
   - When: simulated click / hotkey on the button.
   - Then: a `ChipOfferSkipped` message is emitted.

5. **`skip_input_noop_without_skip_button`** (regression)
   - Given: chip-select screen without SkipButton (Greed inactive).
   - When: simulated input that would otherwise target the button region.
   - Then: no `ChipOfferSkipped` is emitted.

Existing coverage already pins `greed_on_skip` incrementing `GreedStacks.skips` and `apply_greed_boost` re-weighting rarity; no new tests needed on the protocol side.

### Dependencies

- Reads `ActiveProtocols` resource from protocol domain.
- Reads `GreedStacks` and `GreedConfig` resources from protocol domain (for indicator).
- Emits `ChipOfferSkipped` message to protocol domain (via the existing greed consumer).

All three reads are legitimate cross-domain consumption by UI for display purposes — same pattern as `spawn_chip_select` reading `ProtocolOffer`. No architectural change needed.

### UX polish (optional, not blocking)

- Hover tooltip: "Skip the chips — lose power now for better rarity next offer."
- Disabled visual state if skip is somehow unavailable (e.g., first offer of the run — likely not needed; Greed doesn't restrict when skipping is allowed).
- Sound / haptic on press — deferred to audio phase.

## Migration plan

Single feature branch, single commit likely.

1. Add `SkipButton` + optional `SkipIndicator` components.
2. Update `spawn_chip_select/system.rs` with conditional spawn logic + `active_protocols: Res<ActiveProtocols>` + (if indicator) `greed_stacks: Res<GreedStacks>` + `greed_config: Res<GreedConfig>` params.
3. Add `handle_skip_input` system and wire in the chip_select plugin.
4. Write the 5 tests listed above.
5. Standard Verification Tier → commit → Full Verification Tier → merge.

## Scope boundary

In scope:
- `state/run/chip_select/` UI additions (button, indicator, input handler)
- Tests for conditional spawn + input emission

Out of scope:
- Changes to `greed_on_skip` or chip-decay behavior (already working)
- Changes to Greed's `GreedConfig` or `GreedStacks` shape
- Changes to `apply_greed_boost` or chip-offering generation
- Protocol-domain relocation (handled by TODO #1 mutators-domain-refactor)

## Ordering

Independent of TODO #0 and #2. Can land at any time. Doesn't block anything; nothing blocks it. Pure UI deliverable.

## TODO entry

> **[ready]** Greed skip button — conditional skip button + indicator on chip-select UI, visible only when Greed protocol is active. Lives entirely in `state/run/chip_select/`. Subsumes `audit/remediations/greed-skip-button.md`. Independent of #1 and #2. — [detail](detail/greed-skip-button.md)
