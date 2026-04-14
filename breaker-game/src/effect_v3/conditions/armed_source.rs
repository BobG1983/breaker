//! `is_armed_source` — detection helper for armed-entry source names.
//!
//! The armed-entry naming convention is produced by
//! `format!("{source}#armed[{n}]")` in `evaluate_conditions::fire_scoped_tree`.
//! Both `evaluate_conditions.rs` (which creates armed entries) and
//! `walking/on.rs` (which detects whether a fire is happening inside an
//! armed context) must agree on the predicate; putting it in its own
//! small module keeps the convention in exactly one place and avoids a
//! `walking → conditions` cross-module dependency on anything else.

/// Returns `true` if the given source string is an armed-entry key
/// produced by `format!("{source}#armed[{n}]")`.
///
/// # Source-name invariant
///
/// Chip source strings (the RON-defined names like `"chip_redirect"`)
/// MUST NOT contain the substring `#armed[`. A chip named
/// `"my#armed[prefix]chip"` would be misclassified as armed by this
/// helper and spuriously populate `ArmedFiredParticipants`, creating a
/// slow component-state leak. The pattern `#armed[` is unique enough
/// that no current chip collides; enforcement beyond convention is out
/// of scope for this helper.
#[must_use]
pub(in crate::effect_v3) fn is_armed_source(source: &str) -> bool {
    source.contains("#armed[")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn armed_source_with_bracket_zero_is_detected() {
        assert!(is_armed_source("chip_redirect#armed[0]"));
    }

    #[test]
    fn armed_source_with_non_zero_index_is_detected() {
        assert!(is_armed_source("chip_redirect#armed[7]"));
    }

    #[test]
    fn bare_chip_source_is_not_armed() {
        assert!(!is_armed_source("chip_redirect"));
        assert!(!is_armed_source("chip_bare"));
    }

    #[test]
    fn empty_string_is_not_armed() {
        assert!(!is_armed_source(""));
    }
}
