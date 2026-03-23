---
name: chip-select-flow
description: End-to-end data flow for chip offering generation, UI display, player selection, and effect application
type: reference
---

## System Chain (OnEnter ChipSelect)

1. `generate_chip_offerings` -> ApplyDeferred -> `spawn_chip_select`  (chained, same frame via ApplyDeferred)

## System Chain (Update, run_if ChipSelect)

2. `handle_chip_input` -> `tick_chip_timer` -> `update_chip_display`  (chained)
3. `apply_chip_effect` (ChipsPlugin, Update, run_if ChipSelect) -- reads ChipSelected message

## Data Flow

ChipRegistry + ChipInventory + ChipSelectConfig + GameRng
  -> offering::generate_offerings() -> Vec<ChipDefinition>
  -> commands.insert_resource(ChipOffers)
  -> spawn_chip_select reads ChipOffers, spawns UI cards
  -> handle_chip_input: confirm sends ChipSelected message, records decay on non-selected, transitions to TransitionIn
  -> apply_chip_effect: reads ChipSelected, looks up ChipRegistry, triggers ChipEffectApplied per effect
  -> per-effect observers (handle_piercing, handle_overclock, etc.) apply components

## Key Types

- `ChipOffers(Vec<ChipDefinition>)` -- resource bridging generation to UI
- `ChipSelected { name: String }` -- message bridging UI to chips domain
- `ChipEffectApplied { effect, max_stacks }` -- triggered event for observer dispatch
- `ChipInventory` -- tracks held chips (stacks) and decay_weights (offering weight decay)

## Inventory Tracking

`ChipInventory::add_chip()` IS called in production code: `chips/systems/apply_chip_effect.rs:32` calls `inv.add_chip(&msg.name, chip)` when processing `ChipSelected`. The inventory tracks both stacks (via `add_chip`) and decay weights (via `record_offered`).
