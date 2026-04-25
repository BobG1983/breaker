//! Group G — Validation
//!
//! Tests for `PhantomConfig::validate()` rules. Pure unit tests, no App needed.

use crate::cells::behaviors::phantom::components::PhantomConfig;

// Behavior 37: Validation rejects cycle_secs <= 0
#[test]
fn validation_rejects_zero_cycle_secs() {
    let config = PhantomConfig {
        cycle_secs:     0.0,
        telegraph_secs: 0.0,
    };
    assert!(
        config.validate().is_err(),
        "cycle_secs=0.0 should be rejected"
    );
}

// Behavior 37 edge: negative cycle_secs also rejected
#[test]
fn validation_rejects_negative_cycle_secs() {
    let config = PhantomConfig {
        cycle_secs:     -1.0,
        telegraph_secs: 0.0,
    };
    assert!(
        config.validate().is_err(),
        "cycle_secs=-1.0 should be rejected"
    );
}

// Behavior 38: Validation rejects telegraph_secs >= cycle_secs
#[test]
fn validation_rejects_telegraph_equal_to_cycle() {
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 3.0,
    };
    assert!(
        config.validate().is_err(),
        "telegraph_secs=3.0 == cycle_secs=3.0 should be rejected (Solid duration would be zero)"
    );
}

// Behavior 38 edge: telegraph_secs > cycle_secs also rejected
#[test]
fn validation_rejects_telegraph_greater_than_cycle() {
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 3.5,
    };
    assert!(
        config.validate().is_err(),
        "telegraph_secs=3.5 > cycle_secs=3.0 should be rejected"
    );
}

// Behavior 39: Validation rejects negative telegraph_secs
#[test]
fn validation_rejects_negative_telegraph_secs() {
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: -0.1,
    };
    assert!(
        config.validate().is_err(),
        "telegraph_secs=-0.1 should be rejected"
    );
}

// Behavior 40: Validation accepts valid values
#[test]
fn validation_accepts_valid_config() {
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    assert!(
        config.validate().is_ok(),
        "cycle_secs=3.0, telegraph_secs=0.5 should be accepted"
    );
}

// Behavior 40 edge: minimal valid config
#[test]
fn validation_accepts_minimal_valid_config() {
    let config = PhantomConfig {
        cycle_secs:     0.01,
        telegraph_secs: 0.0,
    };
    assert!(
        config.validate().is_ok(),
        "cycle_secs=0.01, telegraph_secs=0.0 should be accepted as minimal valid"
    );
}

// Behavior 41: Validation rejects non-finite cycle_secs
#[test]
fn validation_rejects_infinity_cycle_secs() {
    let config = PhantomConfig {
        cycle_secs:     f32::INFINITY,
        telegraph_secs: 0.5,
    };
    assert!(
        config.validate().is_err(),
        "cycle_secs=INFINITY should be rejected"
    );
}

// Behavior 41 edge: NaN cycle_secs
#[test]
fn validation_rejects_nan_cycle_secs() {
    let config = PhantomConfig {
        cycle_secs:     f32::NAN,
        telegraph_secs: 0.5,
    };
    assert!(
        config.validate().is_err(),
        "cycle_secs=NAN should be rejected"
    );
}

// Behavior 42: Validation rejects non-finite telegraph_secs
#[test]
fn validation_rejects_infinity_telegraph_secs() {
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: f32::INFINITY,
    };
    assert!(
        config.validate().is_err(),
        "telegraph_secs=INFINITY should be rejected"
    );
}

// Behavior 42 edge: NaN telegraph_secs
#[test]
fn validation_rejects_nan_telegraph_secs() {
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: f32::NAN,
    };
    assert!(
        config.validate().is_err(),
        "telegraph_secs=NAN should be rejected"
    );
}
