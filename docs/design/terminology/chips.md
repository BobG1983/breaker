# Chips & Effects

## Chip System

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Chip** | Any upgrade offered during chip selection — all effects are expressed as `TriggerChain` variants | `ChipDefinition`, `ChipRegistry`, `ChipSelected` |
| **ChipTemplate** | RON source type — one file per chip concept with per-rarity slots (`common`/`uncommon`/`rare`/`legendary`). Loader expands templates into individual `ChipDefinition`s at load time. `max_taken` is shared across all rarities. | `ChipTemplate`, `RaritySlot`, `seed_chip_registry` |
| **ChipInventory** | Runtime resource tracking the player's chip build during a run: which chips are held and at what stack level, and which chips have been seen in offerings | `ChipInventory`, `ChipEntry` |
| **ChipOffers** | Transient resource holding the `ChipDefinition`s offered on the chip selection screen for the current visit | `ChipOffers`, `generate_chip_offerings` |
| **ChipOffering** | Enum representing a single item on the chip selection screen. Either `Normal(ChipDefinition)` or `Evolution { ingredients, result }` | `ChipOffering::Normal`, `ChipOffering::Evolution` |

## Named Chips

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Amp** | Named chip — ramping damage bonus on cell hits, resets on non-bump breaker impact | `amp.chip.ron` |
| **Augment** | Named chip — breaker width increase + bump force boost | `augment.chip.ron` |
| **Overclock** | Named chip — timed speed burst after perfect bump | `overclock.chip.ron` |
| **Flux** | Named chip — randomness/instability themed; fires random effects from a weighted pool on bump | `flux.chip.ron`, `RandomEffect` |

## TriggerChain System

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **TriggerChain** | Recursive enum that encodes the full trigger-effect tree for ALL chip effects and archetype behaviors. Trigger wrapper variants (`OnPerfectBump`, `OnImpact`, `OnSelected`, etc.) nest around leaf effect variants (`Shockwave`, `LoseLife`, `SpawnBolt`, `Piercing`, `DamageBoost`, etc.). Replaces the old `ChipEffect`/`AmpEffect`/`AugmentEffect` enums. | `TriggerChain`, `ImpactTarget`, `Target` |
| **OnSelected** | TriggerChain variant for passive effects — evaluated immediately when a chip is selected, rather than waiting for a game event trigger | `TriggerChain::OnSelected`, `apply_chip_effect` |
| **OnBump** | TriggerChain variant that fires on any non-whiff bump (Early, Late, or Perfect) | `TriggerChain::OnBump` |
| **Target** | Enum discriminating which entity type an effect targets: `Bolt`, `Breaker`, or `AllBolts`. Used by `SpeedBoost(Target, val)` and `SizeBoost(Target, val)`. | `Target::Bolt`, `Target::Breaker`, `Target::AllBolts` |
| **RandomEffect** | TriggerChain leaf — weighted random selection from a pool of effects. Each entry is `(weight, TriggerChain)`. | `TriggerChain::RandomEffect` |
| **ActiveChains** | Runtime resource holding all `TriggerChain`s active for the current run. Populated from the archetype definition on entering Playing, and extended when any chip with non-`OnSelected` trigger chains is selected | `ActiveChains` |
| **ArmedTriggers** | Component attached to a bolt entity when a trigger chain matches a trigger node but the inner chain is not yet a leaf. Carries the remaining chain; evaluated by the next matching bridge system | `ArmedTriggers` |
| **EffectFired** | Observer event fired by bridge systems when a `TriggerChain` fully resolves to a leaf. Carries the leaf `TriggerChain` variant and an optional bolt entity | `EffectFired` |

## Evolutions

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **EvolutionRecipe** | A RON-loaded recipe combining chip ingredients into a new chip. Has `ingredients: Vec<EvolutionIngredient>` and `result_definition: ChipDefinition`. | `EvolutionRecipe`, `EvolutionIngredient` |
| **EvolutionRegistry** | Resource holding all loaded `EvolutionRecipe`s. Provides `eligible_evolutions(&ChipInventory)` to return recipes whose ingredient requirements are met. | `EvolutionRegistry`, `chips/resources.rs` |
| **EntropyEngine** | Evolution chip — counter-gated random effect (every Nth cell destroyed, roll from weighted pool). Combines Cascade + Flux ingredients. | `TriggerChain::EntropyEngine`, `entropy_engine.evolution.ron` |
