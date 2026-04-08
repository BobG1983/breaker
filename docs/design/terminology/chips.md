# Chips & Effects

## Chip System

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Chip** | Any upgrade offered during chip selection — all effects are expressed as `EffectNode` variants | `ChipDefinition`, `ChipCatalog`, `ChipSelected` |
| **ChipTemplate** | RON source type — one file per chip concept with per-rarity slots (`common`/`uncommon`/`rare`/`legendary`). Loader expands templates into individual `ChipDefinition`s at load time. `max_taken` is shared across all rarities. | `ChipTemplate`, `RaritySlot`, `expand_chip_template` |
| **ChipTemplateRegistry** | `SeedableRegistry` loading `.chip.ron` files from `assets/chips/standard/`. Stores `(AssetId, ChipTemplate)` pairs keyed by name. Hot-reload triggers `update_single` on file change. | `ChipTemplateRegistry`, `SeedableRegistry`, `chips/resources.rs` |
| **ChipCatalog** | Runtime resource holding all expanded `ChipDefinition`s (built from templates) plus in-catalog `Recipe`s. Paired `Vec<String>` preserves insertion order for deterministic chip offers. NOT a `SeedableRegistry` — populated at load time by template expansion. | `ChipCatalog`, `ordered_values()`, `eligible_recipes()` |
| **SeedableRegistry** | Trait from `rantzsoft_defaults` — folder-based RON asset loading for registries. Implementors define `asset_dir`, `extensions`, `seed`, and `update_single`. `add_registry::<R>()` on `RantzDefaultsPluginBuilder` wires loading, seeding, and hot-reload. | `SeedableRegistry`, `RegistryHandles`, `rantzsoft_defaults::registry` |
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
| **EffectNode** | Recursive enum that encodes the full effect tree for ALL chip effects and breaker behaviors. Six node types: `When` (trigger gate), `Do` (leaf effect), `Until` (timed/triggered removal), `Once` (one-time fire), `On` (target scope), `Reverse` (internal — not authored in RON). Replaces the old `TriggerChain` enum. | `EffectNode`, `When`, `Do`, `Until`, `Once`, `On` |
| **When** | EffectNode variant — trigger gate. Fires children when the trigger condition is met. Re-fires on each activation. | `When { trigger: PerfectBump, then: [Do(Shockwave(...))] }` |
| **Do** | EffectNode variant — leaf effect. Terminal action in the tree. | `Do(Shockwave(...))`, `Do(DamageBoost(2.0))` |
| **Until** | EffectNode variant — applies children, auto-removes when the named trigger fires. Used for timed buffs (`TimeExpires(3.0)`), trigger-based removal (`Impact(Breaker)`). | `Until { trigger: TimeExpires(3.0), then: [Do(DamageBoost(2.0))] }` |
| **Once** | EffectNode variant — fires children once ever, then permanently consumed from the chain. Used for SecondWind-style one-time saves. | `Once([Do(SecondWind(...))])` |
| **BoundEffects** | Component (`BoundEffects(Vec<(String, EffectNode)>)`) on individual entities. Permanent chains that re-evaluate on every matching trigger. Populated at chip dispatch and by `On(permanent: true)` redirects. String key is the chip name for attribution. | `BoundEffects`, `effect/core/types/definitions/enums.rs` |
| **StagedEffects** | Component (`StagedEffects(Vec<(String, EffectNode)>)`) on individual entities. One-shot chains consumed when matched. Populated by `On(permanent: false)` redirects and `Once` wrappers. | `StagedEffects`, `effect/core/types/definitions/enums.rs` |
| **RootEffect** | Top-level enum wrapping an `On` node — constrains breaker definitions so every chain explicitly names its target entity before trigger matching. `BreakerDefinition.effects: Vec<RootEffect>`. | `RootEffect::On`, `effect/core/types/definitions/enums.rs` |
| **EffectNode::On** | EffectNode variant — target scope. Dispatches children against the entity identified by `target` (Bolt, Breaker, Cell, Wall, AllBolts, AllCells). Not a trigger gate; resolved at dispatch time. | `On { target: Bolt, then: [...] }` |
| **Bump** | `Trigger` variant — fires on any non-whiff bump (Early, Late, or Perfect) | `When { trigger: Bump, then: [...] }` |
| **Target** | Enum discriminating which entity type an effect targets: `Bolt`, `Breaker`, `AllBolts`, `Cell`, `AllCells`, `Wall`, or `AllWalls`. Used in `On { target, then }` nodes to scope effect dispatch. | `Target::Bolt`, `Target::Breaker`, `Target::AllBolts`, `Target::Cell`, `Target::AllCells`, `Target::Wall`, `Target::AllWalls` |
| **AttractionType** | Enum discriminating what entity type an `Attraction` effect pulls toward: `Cell`, `Wall`, or `Breaker`. Nearest wins. Type deactivates on hit, reactivates on bounce off non-attracted type. | `AttractionType::Cell`, `AttractionType::Wall`, `AttractionType::Breaker` |
| **SpawnBolts** | Effect leaf — spawns additional bolts. Has `count` (default 1), `lifespan` (default None = permanent), and `inherit` (default false = no effect inheritance). | `Do(SpawnBolts { count: 2, inherit: true })` |
| **RandomEffect** | Effect leaf — weighted random selection from a pool of effects. Each entry is `(weight, EffectNode)`. | `Do(RandomEffect([(0.5, Do(SpeedBoost(...))), ...]))` |
| **Pulse** | Triggered effect leaf — fires a shockwave at every active bolt position simultaneously. Parameters: `base_range`, `range_per_level`, `stacks`, `speed`. | `Do(Pulse { base_range: 32.0, ... })` |
| **SpawnPhantom** | Triggered effect leaf — spawns a temporary phantom bolt with infinite piercing and a lifespan timer. Parameters: `duration`, `max_active`. | `Do(SpawnPhantom { duration: 2.0, max_active: 1 })` |
| **GravityWell** | Triggered effect leaf — spawns a gravity well entity that attracts bolts within a radius for a given duration. Parameters: `strength`, `duration`, `radius`, `max`. | `Do(GravityWell { strength: 1.0, duration: 3.0, radius: 80.0, max: 1 })` |

## Protocols

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **ProtocolKind** | C-style enum identifying each protocol. 15 variants. Used as `HashMap` key in registries and `HashSet` member in `ActiveProtocols`. | `ProtocolKind::Deadline`, `ProtocolKind::DebtCollector` |
| **ProtocolTuning** | Enum with struct variants — each variant carries the kind-specific tuning fields for one protocol. Effect-tree protocols carry `effects: Vec<ValidDef>`. Custom-system protocols carry RON-tunable values. The variant IS the kind discriminant — `tuning.kind()` derives `ProtocolKind`. | `ProtocolTuning::DebtCollector { stack_per_bump: f32 }` |
| **ProtocolDefinition** | RON asset loaded from `assets/protocols/*.protocol.ron`. Common fields (`name`, `description`, `unlock_tier`) + a `ProtocolTuning` variant. | `ProtocolDefinition`, `Asset + TypePath + Deserialize` |
| **ProtocolRegistry** | `SeedableRegistry` resource keyed on `ProtocolKind`. One RON file per enum variant. | `ProtocolRegistry`, `protocols/*.protocol.ron` |
| **ActiveProtocols** | Per-run resource tracking which protocols the player has taken. `HashSet<ProtocolKind>`. Cleared on run reset. | `ActiveProtocols`, `protocol_active(kind)` |
| **ProtocolOffer** | Resource holding the protocol offered on the current chip select screen. `None` if no protocol available. Owned by the protocol domain. | `ProtocolOffer(Option<ProtocolDefinition>)` |

## Hazards

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **HazardKind** | C-style enum identifying each hazard. 16 variants. Used as `HashMap` key in `ActiveHazards`. | `HazardKind::Decay`, `HazardKind::Drift` |
| **HazardTuning** | Enum with struct variants — each variant carries the kind-specific tuning fields. The variant IS the kind discriminant. | `HazardTuning::Decay { base_percent: f32, per_level_percent: f32 }` |
| **HazardDefinition** | RON asset loaded from `assets/hazards/*.hazard.ron`. | `HazardDefinition`, `Asset + TypePath + Deserialize` |
| **HazardRegistry** | `SeedableRegistry` resource keyed on `HazardKind`. | `HazardRegistry`, `hazards/*.hazard.ron` |
| **ActiveHazards** | Per-run resource tracking active hazards and stack counts. `HashMap<HazardKind, u32>`. Stacks increment on each selection. Cleared on run reset. | `ActiveHazards`, `hazard_active(kind)` |

## Evolutions

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Recipe** | Runtime recipe record combining chip ingredients into a new chip. Has `ingredients: Vec<EvolutionIngredient>` and `result_name: String` (looks up the full `ChipDefinition` from `ChipCatalog`). Built at load time from `EvolutionTemplateRegistry` definitions; stored in `ChipCatalog`. | `Recipe`, `EvolutionIngredient`, `ChipCatalog::insert_recipe` |
| **EvolutionTemplate** | RON source type for evolutions (`.evolution.ron`). Has `name`, `description`, `effects`, `ingredients` (required), `max_stacks` (defaults to 1). Expanded into `ChipDefinition` with `rarity: Evolution` by `expand_evolution_template` at catalog-build time. | `EvolutionTemplate`, `expand_evolution_template` |
| **EvolutionTemplateRegistry** | `SeedableRegistry` loading `.evolution.ron` files from `assets/chips/evolutions/`. Each file is an `EvolutionTemplate`. At catalog-build time, templates are expanded into `ChipDefinition`s and `Recipe`s. | `EvolutionTemplateRegistry`, `chips/resources.rs` |
| **EntropyEngine** | Evolution chip — counter-gated random effect (every Nth cell destroyed, roll from weighted pool). Combines Cascade + Flux ingredients. | `EffectNode::Do(EntropyEngine(...))`, `entropy_engine.evolution.ron` |
| **Chain Reaction** | Legendary chip — recursive destruction triggers spawn bolts. Nested `DestroyedCell` triggers. | `chain_reaction.chip.ron` |
| **Feedback Loop** | Legendary chip — deep trigger chain: perfect bump → cell impact → cell destruction → timed speed burst. | `feedback_loop.chip.ron` |
