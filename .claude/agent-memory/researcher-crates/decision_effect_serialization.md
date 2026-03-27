---
name: Effect serialization crate decision
description: Evaluated typetag, dyn-clone, erased-serde, serde_flexitos for the effect system — all rejected in favor of existing concrete enum approach
type: project
---

The effect system uses a concrete `Effect` enum (~25 variants) + `EffectNode` tree deserialized from RON.
No trait objects (`Box<dyn Effect>`) are used — the enum IS the dispatch mechanism.

**Conclusion**: typetag, dyn-clone, erased-serde, and serde_flexitos are all unnecessary and should NOT be added.

**Why**: The project already has the correct architecture. The `Effect` enum in `definition/types.rs` is a closed-world discriminated union — adding a new effect variant means adding an enum arm and a match arm, which is exactly right for a game with ~25 known effect types. Trait objects would add indirection, heap allocation, and serialization complexity for zero benefit.

**typetag specifically**: Would break with RON format. typetag's default "externally tagged" format has RON compatibility issues (`deserialize_identifier` conflicts). The internally tagged style (most natural for game data files) is explicitly unsupported by RON. Even if it worked today, it's a fragile dependency on undocumented RON behavior.

**How to apply**: If a future feature request asks to add typetag or dynamic dispatch to the effect system, decline — the enum approach scales to 50+ variants without architectural pain.
