- [Stub with partial logic — sentinel values matching test assertions](pattern_stub_partial_logic.md) — hardcoded Vec2::ZERO/false in stub causes was_required_to_clear=false test to pass at RED
- [Default config values diverge from spec concrete values](pattern_default_config_vs_spec_values.md) — BoltConfig::default().radius (8.0) used instead of spec-mandated 5.0; test still compiles and overlaps but tests the wrong scenario
- [No-op stub satisfies negative behavior assertions](pattern_noop_stub_satisfies_negative_assertions.md) — tests for "no change" outcomes pass trivially against no-op stubs, breaking the RED gate for those behaviors
- [Design change without backing spec — contradicts existing spec-backed test](pattern_design_change_without_spec.md) — new types (PendingBreakerEffects) appear in tests with no spec, and contradict an existing passing test for the same behavior

- [Bolt #[require] causes false passes for zero-velocity and Spatial2D tests](pattern_require_component_false_pass.md) — stubs that spawn `(Bolt,)` auto-insert Velocity2D(Vec2::ZERO), Spatial2D, InterpolateTransform2D — zero-velocity assertions and spatial marker assertions pass at RED

- [Typestate transition methods with full production logic](pattern_transition_method_production_logic.md) — definition() and dimensions() stubs contain production min/max computations, crossing RED/GREEN boundary even when build() is correctly stubbed

- [Existing production dispatch causes false pass for orchestration tests](pattern_existing_production_dispatch_causes_false_pass.md) — when old dispatch fires unconditionally, State==B and StateChanged assertions pass at RED even with stub orchestration

## Session History
See [ephemeral/](ephemeral/) — not committed.
