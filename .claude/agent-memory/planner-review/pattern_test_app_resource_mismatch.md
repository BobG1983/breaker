---
name: pattern_test_app_resource_mismatch
description: When a system gains a new resource parameter (e.g., ResMut<ActiveChains>), the test_app() setup must also gain .init_resource() — specs must specify this
type: feedback
---

When a system signature is extended with a new `Res` or `ResMut` parameter, all test infrastructure functions (test_app, test helpers) that exercise that system must initialize the resource.

**Why:** The B1-B3 spec added `Option<ResMut<ActiveChains>>` to `apply_chip_effect` but the behavioral spec's test setup (behavior 30) didn't explicitly state the test needed `.init_resource::<ActiveChains>()`. The test would fail at runtime with a missing resource if the writer-tests agent doesn't realize the setup needs updating.

**How to apply:** When reviewing specs where a system gains new resource parameters: (1) Check that every behavior's Given clause mentions the resource, (2) Check that the impl spec's test infrastructure section mentions adding `.init_resource()`, (3) Look for existing test_app() patterns that would need updating.
