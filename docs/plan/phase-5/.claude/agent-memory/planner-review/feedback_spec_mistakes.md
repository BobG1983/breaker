---
name: Common spec mistakes by system type
description: Recurring issues found in specs during review — patterns to check proactively in future reviews
type: feedback
---

## File Path Discrepancy Between Test and Impl Specs

When a feature adds a new test file, test spec Constraints and impl spec Failing Tests must point to the SAME path. A flat `tests.rs` and a directory `tests/fire_tests.rs` are not the same file. Writer-tests and writer-code will produce incompatible output if they read different paths.

**Why:** Found in wave1b-gravity-well-fifo review. The test spec said `tests.rs`, the impl spec said `tests/fire_tests.rs`. BLOCKING issue.

**How to apply:** Always cross-check the "Tests go in:" line in the test spec against the "Failing Tests:" line in the impl spec. They must be identical paths.

---

## Stub Types in Production Files — Requires "Shared Prerequisites" Framing

When a new feature requires types that must live in production source files (not in `#[cfg(test)]` blocks) for the tests to compile, calling them "stubs" is inaccurate and confusing. These are real production types defined early. The spec should call this section "Shared Prerequisites" and explicitly acknowledge this is an approved deviation from the pure writer-tests rule.

**Why:** Found in wave1b-gravity-well-fifo review. The Constraints section used "Stubs" language but asked writer-tests to define real production types in `effect.rs`. This conflates TDD stub (a non-implementing shell) with prerequisite type definition.

**How to apply:** Any time a new type must be visible to both tests and production code AND that type lives in a production file, flag it and recommend "Shared Prerequisites" framing.

---

## Internal Resource State Assertions Are Fragile and Often Wrong

When a spec asserts the internal state of a resource (e.g., "HashMap contains entry {owner: 0}"), verify the exact semantics of the counter. "Get 0, use it, store back" vs "get 0, increment to 1, store 1" produces different map values. Asserting the map value directly is also fragile if the type becomes opaque.

**Why:** Found in wave1b-gravity-well-fifo review. Behavior 1 asserted counter value 0 after a call that consumed 0 — but the impl spec said "increment then store," meaning the stored value would be 1.

**How to apply:** When a spec asserts internal counter/resource state, walk the impl spec's step-by-step logic and verify the post-call value matches the assertion. Prefer asserting the observable output (component on the spawned entity) over internal resource state.

---

## Owner Entity Lifecycle and Resource Cleanup Is Silently Out of Scope

When a feature adds a Resource keyed by Entity, the spec must explicitly exclude or include entity-despawn cleanup behavior. Entity keys in resources can become stale if the owner is despawned. Not testing it is fine, but the spec must say so in Do NOT test.

**Why:** Found in wave1b-gravity-well-fifo review. GravityWellSpawnCounter uses HashMap<Entity, u64>. If owner is despawned, the key is never cleaned up. The spec silently omitted this.

**How to apply:** Any spec that introduces a Resource with Entity keys should include "Do NOT test: owner entity lifecycle and counter cleanup" OR add a behavior for it explicitly.

---

## fire() Tests Need Explicit Stance on Duration Expiry in Bare World

When testing a `fire()` function for effects with duration-based expiry, the spec must state that the bare World has no time system running, so no expiry occurs during the test. Otherwise writer-tests may wonder why wells don't expire between calls.

**Why:** Found in wave1b-gravity-well-fifo review. Behavior 5 (6 calls, max=4) depends on all prior wells remaining alive between calls. Duration expiry in bare World is safe but must be explicit.

**How to apply:** Any test spec for an effect with time-based despawn should add this to the Test Approach: "Duration-based expiry does not run in a bare World. All wells remain alive for the duration of the test regardless of the duration parameter value."
