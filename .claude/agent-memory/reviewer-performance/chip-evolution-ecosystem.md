---
name: chip-evolution-ecosystem performance review
description: Chip catalog, inventory, offering generation, evolution recipes, speed_boost recalculate_velocity — reviewed on feature/chip-evolution-ecosystem
type: project
---

## Review scope

Branch: feature/chip-evolution-ecosystem
Files with production changes:
- `breaker-game/src/effect/effects/speed_boost.rs` — added `recalculate_velocity` called by fire()/reverse()
- `breaker-game/src/chips/resources/data.rs` — ChipCatalog, ChipTemplateRegistry, EvolutionTemplateRegistry, Recipe, eligible_recipes
- `breaker-game/src/chips/inventory/data.rs` — ChipInventory, remove_by_template
- `breaker-game/src/chips/offering/system.rs` — generate_offerings, build_active_pool, per-draw Vec<f32> alloc
- `breaker-game/src/chips/systems/build_chip_catalog/system.rs` — build_chip_catalog (loading, one-shot)
- `breaker-game/src/chips/systems/dispatch_chip_effects/system.rs` — dispatch_chip_effects (Update, ChipSelect gate)
- `breaker-game/src/screen/chip_select/systems/generate_chip_offerings.rs` — generate_chip_offerings (OnEnter)
- `breaker-game/src/screen/chip_select/systems/handle_chip_input.rs` — handle_chip_input (Update, ChipSelect)
- `breaker-game/src/screen/chip_select/systems/update_chip_display.rs` — update_chip_display (Update, ChipSelect)
- `breaker-game/src/screen/chip_select/systems/tick_chip_timer.rs` — tick_chip_timer (Update, ChipSelect)

## Key findings

### world.query::<SpatialData>() in speed_boost::recalculate_velocity (Minor)
`speed_boost.rs:49` — `let mut query = world.query::<SpatialData>()` creates a fresh QueryState on
every call. In Bevy 0.18, World::query() registers the query type against all current archetypes
each time (no caching). Called once per fire() and once per reverse(). Both are episodic (chip
picked up or removed), not per-frame. Impact at current scale: zero. If speed_boost fire()/reverse()
were ever called in a hot loop (unlikely given design), this would be the first thing to fix by
caching the QueryState in a resource. Note: gravity_well uses the same pattern and was confirmed
clean in wave1-stat-boost-and-fifo-effects.md.

**Verdict**: Minor. Episodic trigger makes it a non-issue at all phases.

### Per-draw Vec<f32> allocation in generate_offerings (Minor)
`chips/offering/system.rs:105` — `let weights: Vec<f32> = pool.iter().map(|e| e.weight).collect()`
allocated inside the draw loop. Called up to `offers_per_node` times (default 3) per OnEnter(ChipSelect).
Total: 3 allocations per screen visit, pool size ~10-30 chips. This could be replaced with a
`weights` Vec preallocated outside the loop and truncated/refilled each iteration, but at 3 draws
and ~30 entries this is entirely negligible. NOT per-frame (OnEnter only).

**Verdict**: Minor. Note for future if offers_per_node grows significantly (>10).

### remove_by_template Vec<String> allocation (Minor)
`chips/inventory/data.rs:175` — collects held chip names into a Vec<String> before the removal
loop. Called once per evolution ingredient on chip selection. Player has <20 chips at any time;
allocation is a few dozen bytes. Could be done in-place with retain patterns, but no real impact.

**Verdict**: Minor. Bounded by player inventory size, called once per selection.

### update_chip_display format! in Update (Minor/Watch)
`screen/chip_select/systems/update_chip_display.rs:22` — `format!("{display_secs:.0}")` runs
every Update frame while in GameState::ChipSelect. This allocates a String per frame during the
chip select screen. The game is paused (no physics running) during ChipSelect, so the overall
frame cost is very low. But this is the only per-frame String allocation found.
Could be replaced with a `write!` into a pre-allocated buffer, but it's a UI-only state.

**Verdict**: Minor. Only active during ChipSelect (no physics), not during gameplay.

### Scheduling: all correct
- dispatch_chip_effects: run_if(in_state(GameState::ChipSelect)) — correct
- generate_chip_offerings: OnEnter(GameState::ChipSelect) — correct, one-shot
- build_chip_catalog: loading screen, Local<bool> guard prevents re-runs — correct
- propagate_chip_catalog: cfg(feature = "dev"), is_changed() guard — correct
- tick_chip_timer, handle_chip_input, update_chip_display: Update + run_if(ChipSelect) — correct

### Archetype impact: clean
New components from this branch:
- ActiveSpeedBoosts on bolt entity: already existed from prior phase. No new archetypes.
- ChipOffering/ChipOffers are resources, not components. No archetype impact.
- ChipInventory, ChipCatalog, ChipTemplateRegistry, EvolutionTemplateRegistry: all Resources.
- No new Component types that would fragment archetypes.

### eligible_recipes Vec<&Recipe> allocation (trivial)
`chips/resources/data.rs:71` — allocates Vec<&Recipe> called once per OnEnter(ChipSelect).
Recipe count will be O(10) at game scale. Not a concern.

## Summary
All new code is correctly gated to non-playing states (ChipSelect, Loading). No per-frame
allocations during gameplay. The world.query::<SpatialData>() pattern in speed_boost is the
one item that looks expensive at a glance but is episodic. All chip domain systems run in
Update with run_if guards, not in FixedUpdate. Clean overall.
