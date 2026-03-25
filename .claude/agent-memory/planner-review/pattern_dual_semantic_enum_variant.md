---
name: pattern_dual_semantic_enum_variant
description: Enum variant reused for both triggered and passive contexts has ambiguous field semantics (e.g., SpeedBoost.multiplier meaning multiplier in triggers but flat-additive in passives)
type: feedback
---

When a single enum variant (like `SpeedBoost { target, multiplier }`) serves two different dispatch paths, the field names must match both uses or the spec must explicitly document the dual interpretation.

**Why:** In the ChipEffect-to-TriggerChain flattening, `SpeedBoost` is used by archetype bridge systems (triggered: multiplier means velocity scale factor) and by chip observer handlers (passive: multiplier means flat additive per-stack value). The field name `multiplier` is misleading for the passive use.

**How to apply:** When reviewing enum unifications where a variant appears in multiple dispatch contexts, check: (1) Do all field names make sense in every context? (2) Does the spec explicitly document which handler interprets which field how? (3) Are there test behaviors covering both interpretations?
