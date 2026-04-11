//! `PassiveEffect` trait — aggregation contract for stackable passive effects.

use super::{Fireable, Reversible};

/// Trait bound for types that can live in an `EffectStack`.
///
/// Extends `Fireable + Reversible` — every passive effect must implement
/// fire and reverse explicitly.
///
/// Requires `Clone` (fire clones config into the stack) and `PartialEq + Eq`
/// (remove matches by `(source, config)` pair). `Eq` is possible because all
/// config structs use `OrderedFloat<f32>` instead of raw `f32`.
pub trait PassiveEffect:
    Fireable + Reversible + Sized + Clone + PartialEq + Eq + Send + Sync + 'static
{
    /// Compute the aggregated value from all stacked entries.
    ///
    /// Multiplicative effects return the product. Additive effects return the sum.
    /// Empty stack returns the identity value (1.0 for multiplicative, 0 for additive).
    fn aggregate(entries: &[(String, Self)]) -> f32;
}
