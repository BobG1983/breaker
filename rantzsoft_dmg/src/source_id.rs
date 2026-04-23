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
}
