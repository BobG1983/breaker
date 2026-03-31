---
name: Railgun evolution chip missing from assets
description: evolution_railgun scenario expects "Railgun" to appear in ChipOffers after holding 3x Basic Piercing Shot, but no railgun.evolution.ron file exists in breaker-game/assets/chips/evolution/.
type: project
---

`breaker-scenario-runner/scenarios/mechanic/evolution_railgun.scenario.ron` sets `expected_offerings: Some(["Railgun"])` and seeds the player with 3x "Basic Piercing Shot" + 4x speed chips. The scenario expects the Railgun evolution to be unlocked and offered.

However, there is no `railgun.evolution.ron` in `breaker-game/assets/chips/evolution/`. The `ChipOfferExpected` invariant fires 29 times at frames 33..1545 because "Railgun" never appears in `ChipOffers`.

The `chip_offer_expected_self_test.scenario.ron` also references "Railgun" but is a **self-test** (with `expected_violations: Some([ChipOfferExpected])`), so it correctly fails as intended.

**Fix:** Either create `breaker-game/assets/chips/evolution/railgun.evolution.ron` with the correct ingredients (3x Piercing Shot), or rename the scenario to reference an existing evolution chip.

Note: `chip_catalog.rs` unit tests use "Railgun" as a test fixture name only — not evidence that the RON file exists.
