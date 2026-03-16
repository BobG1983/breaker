---
name: review-feature-review-fixes-2-2026-03-16
description: Quality review of feature/review-fixes-2 branch — chip_select screen, chips domain, seed_chip_registry, seed_chip_select_config, vocabulary rename (upgrade→chip/rig), update_run_setup_colors tests added
type: project
---

Review of feature/review-fixes-2 branch (2026-03-16).

**Why:** Post-implementation review for upgrade vocabulary rename and chip_select screen.

**How to apply:** Use findings below to guide follow-up fixes.

## Issues Found

### Vocabulary
- Clean. The `upgrade` → `chip/rig` rename is complete across all identifiers and UI strings. "CHOOSE A CHIP" is the new title in `spawn_chip_select.rs` — correct.

### Idioms
1. `components.rs:9` — Article typo: "Identifies an chip card" should be "Identifies a chip card". Cosmetic but wrong English; fix to "a".
2. `spawn_chip_select.rs:23` — `.cloned().collect()` on a registry iter is required because `ChipDefinition` contains `String` fields (not `Copy`). This is correct and necessary — do not flag.
3. `handle_chip_input.rs:53` — `chip.name.clone()` in `ChipSelected { name: chip.name.clone(), kind: chip.kind }` — `kind` uses copy semantics (correct after rename), `name` must clone (String). All correct.

### Test Coverage
1. `seed_chip_select_config.rs` — Missing `only_seeds_once` test. The seeded guard is exercised in `seed_chip_registry.rs` but `seed_chip_select_config.rs` has no equivalent test proving the `Local<bool>` guard prevents a second seed. Low severity but the pattern is established across all other seeding systems.
2. `tick_chip_timer.rs:39` — `timer_decrements_after_update` uses a two-update approach to get a non-zero delta. The assertion `timer.remaining < 10.0` relies on real `Time` delta being non-zero. This is correct for the Bevy minimal plugin (uses real time), but the test cannot assert a specific decrement amount. Acceptable given the codebase pattern, but worth noting.

### Documentation
- `components.rs:9` — "Identifies an chip card" — grammatical error in doc comment. Should be "Identifies a chip card".

## Prior Issues Resolved in This Branch
- `spawn_upgrade_select.rs` player-facing string "CHOOSE A POWER-UP" → now "CHOOSE A CHIP" — FIXED
- `UpgradeKind` missing `Copy` → now `ChipKind` derives `Clone, Copy` — FIXED
- Unnecessary `UpgradeOffers(offers.clone())` clone → now `ChipOffers(offers)` after borrowing — FIXED
- `kind: upgrade.kind.clone()` redundant clone → now `kind: chip.kind` (copy) — FIXED
- `spawn_upgrade_select.rs` MAX_CARDS truncation test missing → `large_registry_caps_at_max_cards` test added — FIXED
- `seed_upgrade_registry.rs` weak `only_seeds_once` test → now uses a non-empty first seed and a later-added chip to prove the guard — FIXED
- `update_run_setup_colors.rs` zero tests → 4 tests added — FIXED
- `update_upgrade_display.rs` (now `update_chip_display.rs`) zero tests → 4 tests added — FIXED
