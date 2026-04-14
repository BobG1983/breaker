---
name: Spec internal contradiction between behavior end-state and wipe-on-consume semantics
description: A spec behavior's "Then" assertion contradicts command-flush ordering described in another behavior — writer correctly identifies and drops the wrong assertion but does not substitute the correct one
type: feedback
---

When two behaviors in the same spec describe the same mechanism from different angles, their end-state assertions can contradict each other:

- Behavior 21 (wave-c-when-arming) asserts `StagedEffects.0.is_empty()` after tick 2
- Behavior 26 establishes FIFO command ordering: StageEffectCommand → RemoveEffectCommand → StageEffectCommand (re-arm from bound walk)
- For depth-2 When(A, When(A, Fire(X))), the staged walk fires Fire + queues RemoveEffectCommand; then the bound walk re-arms and queues StageEffectCommand; flush order means StagedEffects.0 ends with 1 entry (not 0)

The writer correctly identified the contradiction and dropped the `is_empty()` assertion.

**Why:** The spec writer modeled tick-2 end state without accounting for the FIFO command queue ordering that Behavior 26 establishes. The `is_empty()` assertion is unreachable given the queue ordering semantics.

**How to apply:** When a behavior's "Then" block specifies a StagedEffects state that contradicts command-queue ordering semantics described elsewhere in the same spec, flag the spec as internally contradictory (IMPORTANT). The correct fix is to update the spec's Behavior 21 to say `StagedEffects.0.len() == 1` (the re-armed entry), not `is_empty()`. The writer's omission of any StagedEffects assertion for tick 2 is acceptable but incomplete — the reviewer should flag the missing positive assertion as IMPORTANT.
