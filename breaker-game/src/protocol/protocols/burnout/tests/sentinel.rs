//! Group I — Sentinel drift guards (Behaviors I1–I2).
//!
//! Pins the exact literal values of the `BURNOUT_SENTINEL` and
//! `BURNOUT_SHOCKWAVE_SOURCE` consts. Any refactor that renames or retypes
//! these consts must also update these tests.

use super::super::system::{BURNOUT_SENTINEL, BURNOUT_SHOCKWAVE_SOURCE};

// ── I1 — BURNOUT_SENTINEL exact value ──────────────────────────────────────-

#[test]
fn burnout_sentinel_value_is_protocol_burnout() {
    assert_eq!(
        BURNOUT_SENTINEL, "protocol:burnout",
        "BURNOUT_SENTINEL drift guard — must be exactly \"protocol:burnout\"; got {BURNOUT_SENTINEL:?}"
    );
}

// ── I1b — Byte-for-byte length match (not substring) ───────────────────────-

#[test]
fn burnout_sentinel_length_matches_exactly() {
    assert_eq!(
        BURNOUT_SENTINEL.len(),
        "protocol:burnout".len(),
        "BURNOUT_SENTINEL must be byte-for-byte identical to the literal"
    );
}

// ── I2 — BURNOUT_SHOCKWAVE_SOURCE exact value ──────────────────────────────-

#[test]
fn burnout_shockwave_source_value_is_protocol_burnout_shockwave() {
    assert_eq!(
        BURNOUT_SHOCKWAVE_SOURCE, "protocol:burnout:shockwave",
        "BURNOUT_SHOCKWAVE_SOURCE drift guard — must be exactly \
         \"protocol:burnout:shockwave\"; got {BURNOUT_SHOCKWAVE_SOURCE:?}"
    );
}

// ── I2b — Sentinels must remain distinct but related ───────────────────────-

#[test]
fn burnout_shockwave_source_and_sentinel_are_distinct_and_related() {
    assert_ne!(
        BURNOUT_SHOCKWAVE_SOURCE, BURNOUT_SENTINEL,
        "BURNOUT_SHOCKWAVE_SOURCE must NOT equal BURNOUT_SENTINEL — they tag different things"
    );
    assert!(
        BURNOUT_SHOCKWAVE_SOURCE.starts_with(BURNOUT_SENTINEL),
        "BURNOUT_SHOCKWAVE_SOURCE should start with BURNOUT_SENTINEL as a namespace prefix"
    );
}
