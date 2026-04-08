# Run Structure

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Tier** | A group of nodes in a run sequence sharing the same difficulty parameters (HP multiplier, timer multiplier, active ratio, introduced cell types) | `TierDefinition`, `tier_index` |
| **DifficultyCurve** | The ordered list of tier definitions loaded from `defaults.difficulty.ron`; drives procedural node sequence generation | `DifficultyCurve`, `DifficultyCurveDefaults` |
| **NodeType** | The category of a single node in the run sequence: Passive, Active, or Boss | `NodeType::Passive`, `NodeType::Active`, `NodeType::Boss` |
| **NodePool** | The pool a node layout belongs to — `Passive`, `Active`, or `Boss` — controls which layouts are eligible for each node type | `NodePool`, `NodeLayout.pool` |
| **NodeSequence** | The full ordered list of node assignments generated from the difficulty curve and run seed | `NodeSequence`, `NodeAssignment`, `generate_node_sequence` |
| **TransitionOut** | Game state representing an animated transition out of a completed node (clear animation) | `GameState::TransitionOut`, `TransitionDirection::Out` |
| **TransitionIn** | Game state representing an animated transition into the next node (load animation) | `GameState::TransitionIn`, `TransitionDirection::In` |
| **TransitionStyle** | Visual style of a node transition — `Flash` (full-screen overlay) or `Sweep` (rect sweeps across screen). Picked from seeded `GameRng`. | `TransitionStyle::Flash`, `TransitionStyle::Sweep` |
| **Protocol** | A positive run-altering upgrade that changes HOW you play (not just how strong you are). One offered per node on the chip select screen as an extra entry — picking it gives up the chip. Each protocol can be taken only once per run. Either an effect tree (dispatched through the effect system) or a custom system with code-implemented behavior. Pool grows via meta-progression. | `ProtocolKind`, `ProtocolDefinition`, `ProtocolTuning`, `ProtocolRegistry`, `ActiveProtocols`, `ProtocolSelected`, `ProtocolOffer` |
| **Hazard** | A negative stackable debuff chosen during infinite play (tier 9+). Three random hazards offered on a dedicated timed screen after chip/protocol selection — player must pick one. Hazards stack — same one can be picked multiple times, each stack intensifies the effect. Code-implemented systems with RON-tunable parameters. | `HazardKind`, `HazardDefinition`, `HazardTuning`, `HazardRegistry`, `ActiveHazards`, `HazardSelected`, `HazardOffers` |
| **HazardSelect** | Run state for the hazard choose-your-poison screen. Appears after ChipSelect, only when the completed node's tier >= 9. Timed — on expiry, a hazard is auto-picked at random from the 3 offers. | `RunState::HazardSelect`, `HazardSelectState`, `resolve_post_chip_state` |
