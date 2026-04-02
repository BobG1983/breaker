//! Breaker domain resources.

use bevy::prelude::*;

use super::messages::BumpGrade;

/// Scenario runner override for bump grading. When `Some`, the `grade_bump`
/// system uses this grade instead of calculating from timing.
#[derive(Resource, Default)]
pub struct ForceBumpGrade(pub Option<BumpGrade>);

#[cfg(test)]
mod tests {
    use super::*;

    // ── ForceBumpGrade tests ────────────────────────────────────────

    #[test]
    fn force_bump_grade_default_is_none() {
        let force = ForceBumpGrade::default();
        assert!(
            force.0.is_none(),
            "ForceBumpGrade::default() should be None"
        );
    }

    #[test]
    fn force_bump_grade_holds_perfect() {
        let force = ForceBumpGrade(Some(BumpGrade::Perfect));
        assert_eq!(
            force.0,
            Some(BumpGrade::Perfect),
            "ForceBumpGrade should hold Some(Perfect)"
        );
    }
}

/// The breaker selected for the current run.
///
/// Set at run start; read by `spawn_or_reuse_breaker` to look up the breaker
/// definition from the registry.
#[derive(Resource, Debug, Clone)]
pub struct SelectedBreaker(pub String);

impl Default for SelectedBreaker {
    fn default() -> Self {
        Self("Aegis".to_owned())
    }
}
