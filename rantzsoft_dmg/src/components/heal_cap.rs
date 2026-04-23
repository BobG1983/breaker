//! Per-message ceiling selector for heal application.

/// Per-message ceiling selector for heal application.
///
/// Different senders want different clamps on the receiving entity's health.
/// `Starting` caps restoration at the pristine value the entity spawned with;
/// `Max` caps restoration at an elevated ceiling supplied elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealCap {
    /// Cap current health at the entity's starting value.
    Starting,
    /// Cap current health at the entity's configured maximum (falling back
    /// to the starting value if no maximum is set).
    Max,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 17: HealCap variants compare by value ──

    #[test]
    fn variants_compare_by_value() {
        // Bind each side through a helper to avoid `clippy::eq_op` on the
        // identical-operand comparisons the spec calls for.
        #[must_use]
        const fn pass(c: HealCap) -> HealCap {
            c
        }
        assert_eq!(HealCap::Starting, pass(HealCap::Starting));
        assert_eq!(HealCap::Max, pass(HealCap::Max));
        assert_ne!(HealCap::Starting, HealCap::Max);
    }

    // ── Behavior 18: HealCap is Copy — pass-by-value does not move ──

    #[test]
    fn is_copy_pass_by_value_does_not_move() {
        const fn take(_c: HealCap) {}
        let cap = HealCap::Starting;
        take(cap);
        take(cap);
    }

    // ── Behavior 19: HealCap derives Debug and Clone ──

    #[test]
    fn derives_clone_and_debug_max() {
        // Clone is routed through a generic `T: Clone` bound — proves the
        // derive exists at compile time without tripping `clippy::clone_on_copy`.
        #[must_use]
        fn require_clone<T: Clone>(value: &T) -> T {
            value.clone()
        }

        let c = HealCap::Max;
        let cloned = require_clone(&c);
        let s = format!("{cloned:?}");
        assert_eq!(cloned, HealCap::Max);
        assert!(s.contains("Max"));
    }

    #[test]
    fn derives_debug_starting() {
        let c = HealCap::Starting;
        let s = format!("{c:?}");
        assert!(s.contains("Starting"));
    }
}
