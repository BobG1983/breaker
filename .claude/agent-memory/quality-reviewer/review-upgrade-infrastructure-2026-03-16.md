---
name: review-upgrade-infrastructure-2026-03-16
description: Quality review of feature/upgrade-infrastructure branch — definition, registry, seed_upgrade_registry, spawn_upgrade_select, handle_upgrade_input, UpgradeOffers, UpgradeKind
type: project
---

Review of feature/upgrade-infrastructure branch (2026-03-16).

**Why:** Post-implementation review for upgrade infrastructure phase.

**How to apply:** Use findings below to guide follow-up fixes.

## Issues Found

### Vocabulary
- `spawn_upgrade_select.rs:70` — player-facing string "CHOOSE A POWER-UP" uses forbidden vocabulary. Should be "CHOOSE AN UPGRADE" or use the specific terms (AMP / AUGMENT / OVERCLOCK). This is the only vocabulary violation in this branch.

### Idioms
1. `definition.rs:7` — `UpgradeKind` is a fieldless enum deriving `Clone` but missing `Copy`. Fieldless enums should derive `Copy` to avoid `.clone()` calls at use sites.
2. `spawn_upgrade_select.rs:29` — `UpgradeOffers(offers.clone())` is an unnecessary clone. `offers` is owned; it's borrowed by `spawn_card_row` on line 47. Fix: move `offers` into `UpgradeOffers(offers)` after cloning for `spawn_card_row`, or pass `spawn_card_row` a clone and move the original into the resource.
3. `handle_upgrade_input.rs:54` — `kind: upgrade.kind.clone()` — redundant once `UpgradeKind` derives `Copy`. After fix #1, this becomes `kind: upgrade.kind` (copy semantics).

### Test Coverage
1. `spawn_upgrade_select.rs` — No test for the MAX_CARDS truncation branch. When the registry has more than 3 entries, `take(MAX_CARDS)` silently caps the offers at 3. A test `registry_larger_than_max_cards_caps_at_three` is missing.
2. `seed_upgrade_registry.rs:133` — `only_seeds_once` test seeds with an empty collection both times. It can't distinguish "seeded once and skipped second call" from "seeded twice with same result." The test should seed with a non-empty collection on first update and then add more assets before the second update, asserting the count doesn't grow. As written, the test would pass even if the guard were removed.

### Documentation
- Clean. All public types have `///` docs. `UpgradeRegistry` doc correctly explains consumers. `UpgradeOffers` doc correctly names its inserter and reader. Units on `timer_secs` fields are properly annotated. No stale comments found.

## Vocabulary Decision Recorded
- The `upgrade` module name, `UpgradeKind`, `UpgradeDefinition`, `UpgradeRegistry`, `UpgradeOffers`, `UpgradeSelected`, `UpgradeCard` — these are all infrastructure/wrapper names, not game nouns. The TERMINOLOGY.md rule targets identifiers for game entities (bolt, cell, etc.), not infrastructure that wraps the Amp/Augment/Overclock concepts. These names are acceptable. Only the player-facing UI string "POWER-UP" is a violation.
