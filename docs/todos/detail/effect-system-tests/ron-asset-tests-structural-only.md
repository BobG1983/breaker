# RON asset tests: do not pin specific tuning values

## Problems addressed

- Cross-cutting pattern flagged during Iron Curtain interrogation (2026-04-21) \u2014 every protocol and hazard `tests/ron_asset.rs` pins specific tuning values, which is brittle and does not test the right thing. Tuning values change; tests that assert them break for non-behavioral reasons and don't catch real wiring bugs.

This file is the sweep; `iron-curtain-falloff-ron-fix.md` is the bug fix that exposed the pattern.

## Rule

A `ron_asset.rs` test MUST NOT assert specific tuning values (multipliers, thresholds, durations, etc.). It MUST assert only:

1. The RON file parses into the expected `ProtocolTuning` / `HazardTuning` variant.
2. All fields are structurally valid (floats are finite, durations/counts are non-negative, required fields are present, enum variants are recognized).
3. Activation inserts the expected `XxxConfig` resource (when activation is in scope for the test).

Exact-value pinning lives in design-behavior tests \u2014 where the test constructs its own `XxxConfig` with specific values and exercises the system against those values. Those tests own the behavior, not the RON author's choice of number.

## Sweep

Rewrite each of the following files to match the rule:

- `breaker-game/src/mutators/protocols/protocols/fission/tests/ron_asset.rs`
- `breaker-game/src/mutators/protocols/protocols/debt_collector/tests/ron_asset.rs`
- `breaker-game/src/mutators/protocols/protocols/iron_curtain/tests/ron_asset.rs` (covered by `iron-curtain-falloff-ron-fix.md`)
- `breaker-game/src/mutators/protocols/protocols/echo_strike/tests/ron_asset.rs`
- `breaker-game/src/mutators/protocols/protocols/greed/tests/ron_asset.rs`
- `breaker-game/src/mutators/protocols/protocols/conductor/tests/ron_asset.rs`
- `breaker-game/src/mutators/protocols/protocols/burnout/tests/ron_asset.rs`
- `breaker-game/src/mutators/protocols/protocols/afterimage/tests/ron_asset.rs`
- `breaker-game/src/mutators/protocols/protocols/siphon/tests/ron_asset.rs`
- `breaker-game/src/mutators/protocols/protocols/reckless_dash/tests/ron_asset.rs`
- `breaker-game/src/mutators/hazards/hazards/momentum/tests/ron_asset.rs`
- `breaker-game/src/mutators/hazards/hazards/sympathy/tests/ron_asset.rs`
- `breaker-game/src/mutators/hazards/hazards/diffusion/tests/ron_asset.rs`
- `breaker-game/src/mutators/hazards/hazards/tether/tests/ron_asset.rs`

For each file, the rewrite replaces any `assert_eq!(config.field, exact_value)` with structural validity checks. Example shape:

```rust
#[test]
fn ron_parses_and_activates() {
    let tuning = load_protocol_tuning("foo.protocol.ron");
    let ProtocolTuning::Foo { field_a, field_b, field_c } = tuning else {
        panic!("expected ProtocolTuning::Foo");
    };
    assert!(field_a.is_finite());
    assert!(field_a >= 0.0);
    assert!(field_b > 0); // if the field is a strictly-positive count
    // field_c is an enum variant: no value assertion, just unpack proves the variant is valid.
}
```

### What tests DO pin the value

Specific values are pinned in design-behavior tests. Example (Iron Curtain): `tests/on_bolt_lost.rs` constructs `IronCurtainConfig { damage_fraction: 0.5, falloff_start: 50.0, .. }` directly in the test body (not loaded from RON) and exercises the system. That's the right place to pin the formula's behavior at a chosen value; it's independent of what the shipped RON contains.

### Optional: one smoke test of RON \u2192 config \u2192 system pipeline

A single cross-cutting smoke test per protocol/hazard that drives the full RON \u2192 parse \u2192 activate \u2192 system-runs pipeline with the shipped RON is fine \u2014 as long as it asserts only structural / behavioral properties (e.g., "system runs one frame without panicking, config resource is present") and NOT specific output values. Authors may add or skip this at their discretion.

### Documentation

Add to `docs/architecture/testing.md` (or the equivalent testing-conventions doc if a different name is used) a new section \u00a7RON asset tests stating the rule above. Link this remediation file from the doc so readers see the rationale.
