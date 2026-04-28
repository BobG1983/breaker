//! Zero-alloc (for static strings) source identifier.

use std::{borrow::Cow, fmt};

/// Source identifier used to tag damage, heal, and similar messages with the
/// origin of their effect.
///
/// Wrapping `Cow<'static, str>` lets callers pass a `&'static str` without
/// allocation while still supporting dynamic IDs produced at runtime.
/// Typical values look like `"module:action"` or `"src:alpha"`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceId(pub Cow<'static, str>);

impl From<&'static str> for SourceId {
    fn from(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }
}

impl From<String> for SourceId {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// True iff a damage-boost (or vulnerability) entry whose filter is
/// `entry_filter` should apply to a `DamageDealt<T>` whose origin is
/// `emission_source`.
///
/// Strict-equality semantics — no instance-stripping, no namespace-prefix
/// matching, no custom predicates. The four-row truth table:
///
/// | `entry_filter` | `emission_source` | result |
/// |----------------|-------------------|--------|
/// | `None`         | `None`            | `true` |
/// | `None`         | `Some(_)`         | `true` |
/// | `Some(f)`      | `Some(s)`         | `f == s` |
/// | `Some(_)`      | `None`            | `false` |
///
/// In words: an unfiltered entry applies to every emission; a filtered
/// entry applies only when the emission carries a matching source id (a
/// `None` emission never matches a `Some(_)` filter).
///
/// Both arguments are passed by reference so callers retain ownership.
#[must_use]
pub fn entry_applies(entry_filter: Option<&SourceId>, emission_source: Option<&SourceId>) -> bool {
    match entry_filter {
        None => true,
        Some(f) => emission_source.is_some_and(|s| f == s),
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, collections::HashSet};

    use super::*;

    // ── Behavior 20: From<&'static str> produces Cow::Borrowed ──

    #[test]
    fn from_static_str_produces_borrowed_cow() {
        let id: SourceId = SourceId::from("module:action");
        assert_eq!(id.0, Cow::Borrowed("module:action"));
        assert!(matches!(id.0, Cow::Borrowed(_)));
    }

    #[test]
    fn from_empty_static_str_produces_borrowed_empty() {
        let id: SourceId = SourceId::from("");
        assert_eq!(id.0, Cow::Borrowed(""));
        assert!(matches!(id.0, Cow::Borrowed(_)));
    }

    // ── Behavior 21: From<String> produces Cow::Owned ──

    #[test]
    fn from_string_produces_owned_cow() {
        let s = format!("module:{}", "action");
        let id: SourceId = SourceId::from(s);
        assert!(matches!(id.0, Cow::Owned(_)));
        assert_eq!(id.0, "module:action");
    }

    #[test]
    fn from_empty_string_produces_owned_empty() {
        let id: SourceId = SourceId::from(String::new());
        assert!(matches!(id.0, Cow::Owned(_)));
        assert_eq!(id.0, "");
    }

    // ── Behavior 22: SourceId equality compares content, not Cow variant ──

    #[test]
    fn equality_compares_content_across_cow_variants() {
        let a = SourceId::from("module:action");
        let b = SourceId::from(format!("module:{}", "action"));
        assert_eq!(a, b);
    }

    #[test]
    fn equality_is_case_sensitive() {
        assert_ne!(
            SourceId::from("module:action"),
            SourceId::from("module:Action"),
        );
    }

    #[test]
    fn equality_matches_empty_across_variants() {
        assert_eq!(SourceId::from(""), SourceId::from(String::new()));
    }

    // ── Behavior 23: Display prints the inner string with no decoration ──

    #[test]
    fn display_prints_inner_string_exactly() {
        let id = SourceId::from("module:action");
        let s = format!("{id}");
        assert_eq!(s, "module:action");
    }

    #[test]
    fn display_empty_is_empty_string() {
        let id = SourceId::from("");
        let s = format!("{id}");
        assert_eq!(s, "");
    }

    #[test]
    fn display_of_owned_matches_content() {
        let s = format!("{}", SourceId::from(String::from("src:alpha")));
        assert_eq!(s, "src:alpha");
    }

    // ── Behavior 24: Hash + Eq deduplicate Borrowed and Owned in a HashSet ──

    #[test]
    fn hashset_deduplicates_borrowed_and_owned_with_same_content() {
        let mut set: HashSet<SourceId> = HashSet::new();
        let _ = set.insert(SourceId::from("module:action"));
        let _ = set.insert(SourceId::from(String::from("module:action")));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn hashset_distinguishes_different_content() {
        let mut set: HashSet<SourceId> = HashSet::new();
        let _ = set.insert(SourceId::from("module:action"));
        let _ = set.insert(SourceId::from(String::from("module:action")));
        let _ = set.insert(SourceId::from("src:beta"));
        assert_eq!(set.len(), 2);
    }

    // ── Behavior 25: SourceId derives Clone and Debug ──

    #[test]
    fn derives_clone_and_debug() {
        let id = SourceId::from("module:action");
        let cloned = id.clone();
        let s = format!("{cloned:?}");
        assert_eq!(cloned, id);
        assert!(!s.is_empty());
        assert!(s.contains("module:action"));
    }

    #[test]
    fn cloning_owned_variant_preserves_content() {
        let id = SourceId::from(String::from("src:alpha"));
        let cloned = id.clone();
        assert_eq!(cloned, id);
        assert!(matches!(cloned.0, Cow::Owned(_)));
    }

    // ── Behavior 26: `entry_applies(None, None)` — filterless entry, sourceless emission ──

    #[test]
    fn entry_applies_none_filter_none_emission_returns_true() {
        assert!(entry_applies(None, None));
    }

    // ── Behavior 27: `entry_applies(None, Some)` — filterless entry applies to any emission ──

    #[test]
    fn entry_applies_none_filter_any_emission_returns_true() {
        let emission = SourceId::from("protocol:burnout");
        assert!(entry_applies(None, Some(&emission)));
    }

    // ── Behavior 28: `entry_applies(Some(X), Some(X))` — filter matches emission ──

    #[test]
    fn entry_applies_matching_filter_and_emission_returns_true() {
        let filter = SourceId::from("protocol:burnout");
        let emission = SourceId::from("protocol:burnout");
        assert!(entry_applies(Some(&filter), Some(&emission)));
    }

    #[test]
    fn entry_applies_match_is_content_equality_across_cow_variants() {
        // Edge case: explicit `let` bindings keep the SourceId values alive
        // across the borrow into `entry_applies`. Filter is Cow::Borrowed
        // (from &'static str); emission is Cow::Owned (from String). Pins
        // that match is content-equality, NOT pointer/variant equality.
        let filter = SourceId::from("protocol:burnout");
        let emission = SourceId::from(String::from("protocol:burnout"));
        assert!(entry_applies(Some(&filter), Some(&emission)));
    }

    // ── Behavior 29: `entry_applies(Some(X), Some(Y))` — non-matching filter+emission ──

    #[test]
    fn entry_applies_mismatched_filter_and_emission_returns_false() {
        let filter = SourceId::from("protocol:burnout");
        let emission = SourceId::from("protocol:debt_collector");
        assert!(!entry_applies(Some(&filter), Some(&emission)));
    }

    #[test]
    fn entry_applies_is_case_sensitive() {
        // Edge case: case-sensitivity inherited from SourceId equality
        // (Behavior 22 in this file). "protocol:burnout" vs
        // "protocol:Burnout" must NOT match.
        let filter = SourceId::from("protocol:burnout");
        let emission = SourceId::from("protocol:Burnout");
        assert!(!entry_applies(Some(&filter), Some(&emission)));
    }

    // ── Behavior 30: `entry_applies(Some(X), None)` — filtered entry never applies to source-less emission ──

    #[test]
    fn entry_applies_some_filter_none_emission_returns_false() {
        let filter = SourceId::from("protocol:burnout");
        assert!(!entry_applies(Some(&filter), None));
    }

    #[test]
    fn entry_applies_some_filter_none_emission_holds_for_any_filter_id() {
        // Edge case: a different filter id with `None` emission also
        // returns false. Pins that NO filter id matches a None emission.
        let filter = SourceId::from("protocol:debt_collector");
        assert!(!entry_applies(Some(&filter), None));
    }
}
