//! `BumpStatus` — whether bump input was active during bolt-breaker contact.

/// Whether bump input was active when the bolt contacted the breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BumpStatus {
    /// Bump input was active; the grading system will grade the timing.
    Active,
    /// No bump input was active.
    Inactive,
}
