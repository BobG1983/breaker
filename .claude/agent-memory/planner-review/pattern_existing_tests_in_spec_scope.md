---
name: pattern_existing_tests_in_spec_scope
description: Test specs that say "write N tests per file" miss that some files already have those tests written — causes duplicate test name compile errors
type: feedback
---

When a spec says "write 7 tests" for an effect module, always read the actual source file first. Some modules (e.g., speed_boost.rs, damage_boost.rs) already have their tests written as part of an earlier wave. If writer-tests writes them again, the test module will have duplicate function names and fail to compile.

**Why:** Effect modules in breaker-game are built incrementally across waves. fire/reverse stubs and their tests may be written in a prior wave before the helper method (multiplier/total) is ready. The spec for the helper-method wave must acknowledge which tests already exist and which need to be added fresh.

**How to apply:** For any spec touching existing .rs files in effect/effects/, read each file before approving. Note which tests already exist. The spec must say "add tests X, Y, Z" not "write 7 tests" if some already exist. Flag as BLOCKING if the spec would cause writer-tests to overwrite or duplicate existing tests.
