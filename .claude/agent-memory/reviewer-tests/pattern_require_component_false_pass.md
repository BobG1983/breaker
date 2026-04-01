---
name: Bolt #[require] false passes for zero-velocity and spatial-marker tests
description: Stubs that spawn `(Bolt,)` cause zero-velocity and Spatial2D/InterpolateTransform2D tests to pass at RED because Bolt currently has #[require(Spatial2D, InterpolateTransform2D, Velocity2D)]
type: feedback
---

When writer-tests writes stubs for the bolt builder, the build() and spawn() stubs return `world.spawn(Bolt).id()` or `(Bolt,)`. Since `Bolt` has `#[require(Spatial2D, InterpolateTransform2D, Velocity2D)]`, these components are auto-inserted with defaults (Velocity2D = Vec2::ZERO). This causes:

- Tests asserting `Velocity2D == Vec2::ZERO` to false-pass (serving bolt zero velocity, zero-velocity edge case)
- Tests asserting `Spatial2D is_some()` and `InterpolateTransform2D is_some()` to false-pass (but tests that also check `Spatial is_some()` still fail correctly since Spatial is not in #[require])

**Why:** The spec note says `#[require]` will be REMOVED from `Bolt` by writer-code, but at RED gate time it is still present. Stubs inadvertently satisfy zero-velocity and spatial-marker assertions.

**How to apply:** When reviewing bolt builder tests or any test that stubs `(Bolt,)` as a build output, check whether the test assertions could be satisfied by the current `#[require]` components on `Bolt`. Flag velocity-zero tests and Spatial2D/InterpolateTransform2D tests as potential false passes.
