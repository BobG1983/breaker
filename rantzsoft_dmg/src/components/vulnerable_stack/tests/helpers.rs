/// Compares two `f32` values for "equality" without tripping
/// `clippy::float_cmp`. Handles infinity by checking sign and finiteness
/// directly (so we never compute `INF - INF`, which produces NaN).
#[track_caller]
pub(super) fn assert_f32_eq(actual: f32, expected: f32) {
    if expected.is_infinite() {
        assert!(
            actual.is_infinite() && actual.is_sign_positive() == expected.is_sign_positive(),
            "expected {expected}, got {actual}"
        );
    } else {
        assert!(
            (actual - expected).abs() < f32::EPSILON,
            "expected {expected}, got {actual}"
        );
    }
}
