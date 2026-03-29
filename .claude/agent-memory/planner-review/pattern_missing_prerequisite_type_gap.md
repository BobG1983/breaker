---
name: Missing prerequisite type — Wave-gated component specs
description: Spec assumes Effective* components exist from a prior wave, but they were never defined — only Active* placeholders exist. Writer-tests will fail to compile.
type: feedback
---

When a spec says "Wave N-1 must create these types before Wave N writer-tests launch," always verify the types actually exist in the codebase. The recalculate_* systems may be present as no-op placeholders while the Effective* output components they're supposed to write are completely absent.

**Why:** In the Wave 3A/3B split, `ActiveDamageBoosts`, `ActiveSpeedBoosts`, `ActiveSizeBoosts`, and `ActivePiercings` all existed in `effect/effects/*.rs`. But `EffectiveDamageMultiplier`, `EffectiveSpeedMultiplier`, `EffectiveSizeMultiplier`, and `EffectivePiercing` did not exist at all. The recalculate systems were stubs that read Active* components but wrote nothing. Writer-tests for Wave 3B would compile against types that don't yet exist.

**How to apply:** For any spec with a "shared prerequisites" section pointing to another wave, grep for the type names before approving. If they're missing, flag as BLOCKING — the main agent must create them as actual shared prerequisites before launching writer-tests, not just as "assumed to exist."
