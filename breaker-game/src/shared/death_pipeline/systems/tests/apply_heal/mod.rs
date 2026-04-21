//! Tests for `apply_heal<T>`.
//!
//! Covers Groups A–H (Behaviors 1–25), plus Behavior 31 (per-T queue isolation
//! with `Salvo` + `TestEntity` side-by-side) and Behavior 35 (`HealCap`
//! exhaustive compile-time match) from the heal-pipeline test spec.

use crate::shared::death_pipeline::heal_dealt::HealCap;

// Behavior 35: HealCap exhaustive compile-time match — adding a new variant
// without updating this match breaks the build.
const _: fn(HealCap) = |cap| match cap {
    HealCap::Starting | HealCap::Max => (),
};

mod attribution;
mod baseline;
mod ceiling_clamp;
mod dead_filter;
mod invulnerable_filter;
mod mixed_cap;
mod numeric_edges;
mod per_t_isolation;
mod target_edges;
