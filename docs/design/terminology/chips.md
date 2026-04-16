# Chips & Effects

## Chip System

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Chip** | Any upgrade offered during chip selection — all effects are expressed as `Tree` variants under `RootNode` entries | `ChipDefinition`, `ChipCatalog`, `ChipSelected` |
| **ChipTemplate** | RON source type — one file per chip concept with per-rarity slots (`common`/`uncommon`/`rare`/`legendary`). Loader expands templates into individual `ChipDefinition`s at load time. `max_taken` is shared across all rarities. | `ChipTemplate`, `RaritySlot`, `expand_chip_template` |
| **ChipTemplateRegistry** | `SeedableRegistry` loading `.chip.ron` files from `assets/chips/standard/`. Stores `(AssetId, ChipTemplate)` pairs keyed by name. Hot-reload triggers `update_single` on file change. | `ChipTemplateRegistry`, `SeedableRegistry`, `chips/resources.rs` |
| **ChipCatalog** | Runtime resource holding all expanded `ChipDefinition`s (built from templates) plus in-catalog `Recipe`s. Paired `Vec<String>` preserves insertion order for deterministic chip offers. NOT a `SeedableRegistry` — populated at load time by template expansion. | `ChipCatalog`, `ordered_values()`, `eligible_recipes()` |
| **SeedableRegistry** | Trait from `rantzsoft_defaults` — folder-based RON asset loading for registries. Implementors define `asset_dir`, `extensions`, `seed`, and `update_single`. `add_registry::<R>()` on `RantzDefaultsPluginBuilder` wires loading, seeding, and hot-reload. | `SeedableRegistry`, `RegistryHandles`, `rantzsoft_defaults::registry` |
| **ChipInventory** | Runtime resource tracking the player's chip build during a run: which chips are held and at what stack level, and which chips have been seen in offerings | `ChipInventory`, `ChipEntry` |
| **ChipOffers** | Transient resource holding the `ChipDefinition`s offered on the chip selection screen for the current visit | `ChipOffers`, `generate_chip_offerings` |
| **ChipOffering** | Enum representing a single item on the chip selection screen. Either `Normal(ChipDefinition)` or `Evolution { ingredients, result }` | `ChipOffering::Normal`, `ChipOffering::Evolution` |
| **Splinter** | Named triggered chip — spawns temporary, small bolts on cell destruction. No effect inheritance. Evolution ingredient for Chain Reaction. | `splinter.chip.ron`, `SpawnBolts` |
| **Flux** | Also the name of the meta-currency earned per run. Flux accumulates across runs for meta-progression spending. | `flux_earned`, `MenuState::MetaProgression` |

## Named Chips

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Amp** | Named chip — ramping damage bonus on cell hits, resets on non-bump breaker impact | `amp.chip.ron` |
| **Augment** | Named chip — breaker width increase + bump force boost | `augment.chip.ron` |
| **Overclock** | Named chip — timed speed burst after perfect bump | `overclock.chip.ron` |
| **Flux** | Named chip — randomness/instability themed; fires random effects from a weighted pool on bump | `flux.chip.ron`, `RandomEffect` |

## Effect Tree System

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Tree** | Recursive enum encoding the full effect tree for ALL chip effects and breaker behaviors. Seven node types: `Fire` (leaf effect), `When` (trigger gate), `Once` (one-shot gate), `During` (condition-scoped), `Until` (event-scoped), `Sequence` (ordered multi-fire), `On` (participant redirect). | `Tree`, `effect_v3/types/tree.rs` |
| **ScopedTree** | Restricted tree inside `During`/`Until` — direct `Fire` is reversible-only, nested `When` re-opens to full `Tree`. | `ScopedTree`, `effect_v3/types/scoped_tree.rs` |
| **Fire** | Tree variant — leaf effect. Queues a `FireEffectCommand` to execute the effect. | `Fire(SpeedBoost(SpeedBoostConfig(multiplier: 1.5)))` |
| **When** | Tree variant — repeating trigger gate. Fires inner tree when trigger matches. Re-arms each time. | `When(PerfectBumped, Fire(Shockwave(...)))` |
| **Once** | Tree variant — one-shot trigger gate. Self-removes after first match. | `Once(BoltLostOccurred, Fire(SecondWind(())))` |
| **During** | Tree variant — condition-scoped. Applies inner effects while condition is true, reverses when false. Can cycle. | `During(NodeActive, Fire(SpeedBoost(...)))` |
| **Until** | Tree variant — event-scoped. Applies effects immediately, reverses when trigger fires. Used for timed buffs. | `Until(TimeExpires(2.0), Fire(SpeedBoost(...)))` |
| **On** | Tree variant — participant redirect. Resolves a participant entity from the trigger context and redirects the terminal there. | `On(Bump(Bolt), Fire(DamageBoost(...)))` |
| **RootNode** | Top-level entry point for chip/breaker/cell effect definitions. Either `Stamp(StampTarget, Tree)` or `Spawn(EntityKind, Tree)`. | `RootNode::Stamp`, `RootNode::Spawn`, `effect_v3/types/root_node.rs` |
| **StampTarget** | Enum identifying which entities a `Stamp` root installs onto: `Bolt`, `Breaker`, `ActiveBolts`, `EveryBolt`, `PrimaryBolts`, `ExtraBolts`, `ActiveCells`, `EveryCell`, `ActiveWalls`, `EveryWall`, `ActiveBreakers`, `EveryBreaker`. | `StampTarget::Bolt`, `effect_v3/types/stamp_target.rs` |
| **BoundEffects** | Component (`BoundEffects(Vec<(String, Tree)>)`) on entities. Permanent trees that re-evaluate on every matching trigger. String key is the chip name. | `BoundEffects`, `effect_v3/storage/bound_effects.rs` |
| **StagedEffects** | Component (`StagedEffects(Vec<(String, Tree)>)`) on entities. One-shot trees consumed when matched. | `StagedEffects`, `effect_v3/storage/staged_effects.rs` |
| **EffectType** | Enum of all 30 effect variants, each wrapping a per-effect config struct. The dispatch layer — `fire_dispatch` matches on it. | `EffectType::SpeedBoost(SpeedBoostConfig)`, `effect_v3/types/effect_type.rs` |
| **ReversibleEffectType** | 16-variant subset of `EffectType` for effects that can be reversed. Used in `ScopedTree::Fire`. | `ReversibleEffectType`, `effect_v3/types/reversible_effect_type.rs` |
| **AttractionType** | Enum discriminating what entity type an `Attraction` effect pulls toward: `Cell`, `Wall`, or `Breaker`. Nearest wins. Type deactivates on hit, reactivates on bounce off non-attracted type. | `AttractionType::Cell`, `AttractionType::Wall`, `AttractionType::Breaker` |
| **SpawnBolts** | Effect — spawns additional bolts. Config: `count`, `lifespan` (Option), `inherit` (bool). | `Fire(SpawnBolts(SpawnBoltsConfig(count: 2, inherit: true)))` |
| **RandomEffect** | Effect — weighted random selection from a pool of effects. Config: `pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>`. | `Fire(RandomEffect(RandomEffectConfig(pool: [...])))` |
| **Pulse** | Effect — periodic shockwave emitter on a bolt. Config: `base_range`, `range_per_level`, `stacks`, `speed`, `interval`. | `Fire(Pulse(PulseConfig(...)))` |
| **SpawnPhantom** | Effect — temporary phantom bolt with limited lifespan. Config: `duration`, `max_active`. | `Fire(SpawnPhantom(SpawnPhantomConfig(duration: 2.0, max_active: 1)))` |
| **GravityWell** | Effect — attracts bolts within radius for a duration. Config: `strength`, `duration`, `radius`, `max`. | `Fire(GravityWell(GravityWellConfig(...)))` |

## Protocols

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **ProtocolKind** | C-style enum identifying each protocol. 15 variants. Used as `HashMap` key in registries and `HashSet` member in `ActiveProtocols`. | `ProtocolKind::Deadline`, `ProtocolKind::DebtCollector` |
| **ProtocolTuning** | Enum with struct variants — each variant carries the kind-specific tuning fields for one protocol. Effect-tree protocols carry `effects: Vec<RootNode>`. Custom-system protocols carry RON-tunable values. The variant IS the kind discriminant — `tuning.kind()` derives `ProtocolKind`. | `ProtocolTuning::DebtCollector { stack_per_bump: f32 }` |
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
| **EntropyEngine** | Evolution chip — counter-gated random effect (every Nth cell destroyed, roll from weighted pool). Combines Cascade + Flux ingredients. | `EntropyEngine(EntropyConfig(...))`, `entropy_engine.evolution.ron` |
| **Chain Reaction** | Legendary chip — recursive destruction triggers spawn bolts. Nested `DeathOccurred(Cell)` triggers. | `chain_reaction.chip.ron` |
| **Feedback Loop** | Legendary chip — deep trigger chain: perfect bump → cell impact → cell destruction → timed speed burst. | `feedback_loop.chip.ron` |
