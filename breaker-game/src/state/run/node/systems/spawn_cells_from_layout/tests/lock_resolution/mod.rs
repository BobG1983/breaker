//! Tests for lock resolution behavior driven by `NodeLayout.locks`.
//!
//! These tests verify the two-pass spawn approach: Pass 1 spawns non-locked
//! cells; Pass 2 spawns locked cells with resolved entity IDs from Pass 1.

mod helpers;

mod basic_locking;
mod chain_resolution;
mod cycle_handling;
mod edge_cases;
mod lock_properties;
