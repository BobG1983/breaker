# Chips & Effects

## Chip System

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Chip** | Any upgrade offered during chip selection — all effects are expressed as `EffectNode` variants | `ChipDefinition`, `ChipRegistry`, `ChipSelected` |
| **ChipTemplate** | RON source type — one file per chip concept with per-rarity slots (`common`/`uncommon`/`rare`/`legendary`). Loader expands templates into individual `ChipDefinition`s at load time. `max_taken` is shared across all rarities. | `ChipTemplate`, `RaritySlot`, `seed_chip_registry` |
| **ChipInventory** | Runtime resource tracking the player's chip build during a run: which chips are held and at what stack level, and which chips have been seen in offerings | `ChipInventory`, `ChipEntry` |
| **ChipOffers** | Transient resource holding the `ChipDefinition`s offered on the chip selection screen for the current visit | `ChipOffers`, `generate_chip_offerings` |
| **ChipOffering** | Enum representing a single item on the chip selection screen. Either `Normal(ChipDefinition)` or `Evolution { ingredients, result }` | `ChipOffering::Normal`, `ChipOffering::Evolution` |
| **Splinter** | Named triggered chip — spawns temporary, small bolts on cell destruction. No effect inheritance. Evolution ingredient for Chain Reaction. | `splinter.chip.ron`, `SpawnBolts` |

## Named Chips

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Amp** | Named chip — ramping damage bonus on cell hits, resets on non-bump breaker impact | `amp.chip.ron` |
| **Augment** | Named chip — breaker width increase + bump force boost | `augment.chip.ron` |
| **Overclock** | Named chip — timed speed burst after perfect bump | `overclock.chip.ron` |
| **Flux** | Named chip — randomness/instability themed; fires random effects from a weighted pool on bump | `flux.chip.ron`, `RandomEffect` |

## EffectNode System

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **EffectNode** | Recursive enum that encodes the full effect tree for ALL chip effects and breaker behaviors. Four node types: `When` (trigger gate), `Do` (leaf effect), `Until` (conditional removal), `Once` (one-time fire). Replaces the old `TriggerChain` enum. | `EffectNode`, `When`, `Do`, `Until`, `Once` |
| **When** | EffectNode variant — trigger gate. Fires children when the trigger condition is met. Re-fires on each activation. | `When { trigger: OnPerfectBump, then: [Do(Shockwave(...))] }` |
| **Do** | EffectNode variant — leaf effect. Terminal action in the tree. | `Do(Shockwave(...))`, `Do(DamageBoost(2.0))` |
| **Until** | EffectNode variant — applies children, auto-removes when `until` trigger fires. Used for timed buffs (`TimeExpires(3.0)`), trigger-based removal (`OnImpact(Breaker)`). | `Until { until: TimeExpires(3.0), then: [Do(DamageBoost(2.0))] }` |
| **Once** | EffectNode variant — fires children once ever, then permanently consumed from the chain. Used for SecondWind-style one-time saves. | `Once([Do(SecondWind(...))])` |
| **EffectChains** | Component (`EffectChains(Vec<EffectNode>)`) on individual entities (bolts, cells). Entity-local chains, evaluated by bridge systems based on triggers. | `EffectChains`, `effect/definition.rs` |
| **ActiveEffects** | Global resource (`Vec<(Option<String>, EffectNode)>`) holding all breaker-definition and triggered-chip chains. Bridge helpers sweep it for global and breaker-owned triggers. | `ActiveEffects`, `effect/active.rs` |
| **ArmedEffects** | Component on bolt entities holding partially-resolved `When` trees waiting for a deeper trigger. Consumed on Fire, replaced on re-Arm. | `ArmedEffects`, `effect/armed.rs` |
| **OnSelected** | Trigger variant for passive effects — evaluated immediately when a chip is selected, rather than waiting for a game event trigger | `When { trigger: OnSelected, then: [...] }` |
| **OnBump** | Trigger variant that fires on any non-whiff bump (Early, Late, or Perfect) | `When { trigger: OnBump, then: [...] }` |
| **Target** | Enum discriminating which entity type an effect targets: `Bolt`, `Breaker`, or `AllBolts`. Used by `SpeedBoost(Target, val)` and `SizeBoost(Target, val)`. | `Target::Bolt`, `Target::Breaker`, `Target::AllBolts` |
| **AttractionType** | Enum discriminating what entity type an `Attraction` effect pulls toward: `Cell`, `Wall`, or `Breaker`. Nearest wins. Type deactivates on hit, reactivates on bounce off non-attracted type. | `AttractionType::Cell`, `AttractionType::Wall`, `AttractionType::Breaker` |
| **SpawnBolts** | Effect leaf — spawns additional bolts. Has `count` (default 1), `lifespan` (default None = permanent), and `inherit` (default false = no effect inheritance). | `Do(SpawnBolts { count: 2, inherit: true })` |
| **RandomEffect** | Effect leaf — weighted random selection from a pool of effects. Each entry is `(weight, EffectNode)`. | `Do(RandomEffect([(0.5, Do(SpeedBoost(...))), ...]))` |

## Evolutions

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **EvolutionRecipe** | A RON-loaded recipe combining chip ingredients into a new chip. Has `ingredients: Vec<EvolutionIngredient>` and `result_definition: ChipDefinition`. | `EvolutionRecipe`, `EvolutionIngredient` |
| **EvolutionRegistry** | Resource holding all loaded `EvolutionRecipe`s. Provides `eligible_evolutions(&ChipInventory)` to return recipes whose ingredient requirements are met. | `EvolutionRegistry`, `chips/resources.rs` |
| **EntropyEngine** | Evolution chip — counter-gated random effect (every Nth cell destroyed, roll from weighted pool). Combines Cascade + Flux ingredients. | `EffectNode::Do(EntropyEngine(...))`, `entropy_engine.evolution.ron` |
| **Chain Reaction** | Evolution chip — recursive bolt spawning with effect inheritance on cell destruction. Combines Cascade + Splinter + Piercing ingredients. | `chain_reaction.evolution.ron` |
| **Feedback Loop** | Evolution chip — counter-gated burst (every 3rd perfect bump fires bolts + shockwave). Ingredients TBD. | `feedback_loop.evolution.ron` |
