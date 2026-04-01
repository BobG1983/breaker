---
name: Chain Reaction name collision — universal scenario failure (RESOLVED)
description: RESOLVED — chain_reaction.evolution.ron was renamed to "Shock Chain", eliminating the name collision with the chain_reaction.chip.ron template.
type: project
---

**RESOLVED**: `chain_reaction.evolution.ron` was renamed from `"Chain Reaction"` to `"Shock Chain"` (feature/chip-evolution-ecosystem). The name collision no longer exists. `validate_recipe_ingredients` no longer fires spurious WARNs for this reason.

Historical record below.

---

Both `breaker-game/assets/chips/templates/chain_reaction.chip.ron` and `breaker-game/assets/chips/evolution/chain_reaction.evolution.ron` previously had `name: "Chain Reaction"`.

In `populate_catalog` (`breaker-game/src/chips/systems/build_chip_catalog/system.rs`):
1. Template chips are inserted first: `catalog.insert(def)` with `name="Chain Reaction"`, `template_name: Some("Chain Reaction")`
2. Evolution chips are inserted second: `catalog.insert(def)` with `name="Chain Reaction"`, `template_name: None`
3. The HashMap insertion for the evolution **overwrites** the template chip entry keyed by "Chain Reaction"

After the overwrite, `validate_recipe_ingredients` collects `template_names` from all `def.template_name.as_deref()` — but the "Chain Reaction" entry now has `template_name: None` (the evolution). So "Chain Reaction" is absent from `template_names`, and both the Chain Reaction recipe (ingredient: "Chain Reaction") and the Voltchain recipe (ingredient: "Chain Reaction") fire a WARN.

The scenario runner captures all WARN logs from the `breaker` target and unconditionally fails any scenario where a WARN is captured. This causes ALL 111 scenarios to fail at frame 0.

**Fix:** Rename either the template or the evolution to avoid the name collision. The evolution `chain_reaction.evolution.ron` name should be distinct from the template `chain_reaction.chip.ron` name (e.g., "Catalyst" for the evolution, or rename the template's base name).

**Files:**
- `breaker-game/assets/chips/evolution/chain_reaction.evolution.ron` — name: "Chain Reaction" (conflicts)
- `breaker-game/assets/chips/templates/chain_reaction.chip.ron` — name: "Chain Reaction" (conflicts)
- `breaker-game/src/chips/systems/build_chip_catalog/system.rs` — `populate_catalog` and `validate_recipe_ingredients`
- `breaker-game/src/chips/resources/data.rs` — `ChipCatalog::insert()` (HashMap overwrites by name)
