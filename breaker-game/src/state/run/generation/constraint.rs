use serde::{Deserialize, Deserializer};

/// Discriminator enum mirroring every `CellBehavior` variant — used in
/// generation constraints without carrying variant-specific runtime data.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum BehaviorKind {
    Regen,
    Guarded,
    Volatile,
    Sequence,
    Armored,
    Phantom,
    Magnetic,
    Survival,
    SurvivalPermanent,
    Portal,
}

/// Constraint on which cell modifier(s) a slot position may receive during
/// node generation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CellConstraint {
    /// No constraint — resolve from the tier modifier pool (may receive any
    /// modifier or none).
    Any,
    /// Cell must receive one of the listed modifiers.
    MustInclude(Vec<BehaviorKind>),
    /// Cell must NOT receive any of the listed modifiers. When the list is
    /// supplied as the shorthand `MustNotInclude(Any)` in RON, it deserializes
    /// to `PlainCell`.
    MustNotInclude(Vec<BehaviorKind>),
    /// Cell receives no modifier — plain cell only.
    PlainCell,
}

/// Wire-format mirror of `CellConstraint` used during deserialization. The
/// `MustNotInclude` variant accepts either a list of behaviors or the bare
/// token `Any` (shorthand for "no modifier") via the nested untagged enum.
#[derive(Deserialize)]
enum CellConstraintWire {
    Any,
    MustInclude(Vec<BehaviorKind>),
    MustNotInclude(MustNotIncludePayload),
    PlainCell,
}

/// Payload variants of `MustNotInclude`. `Any` MUST come first because
/// `#[serde(untagged)]` walks variants in declaration order — the bare token
/// `Any` would not match `Vec<BehaviorKind>` (which expects a sequence), so
/// `AnyToken` has to be tried first.
#[derive(Deserialize)]
#[serde(untagged)]
enum MustNotIncludePayload {
    Any(AnyToken),
    List(Vec<BehaviorKind>),
}

/// Helper unit enum so serde will match the bare token `Any` against a typed
/// value rather than failing the untagged branch.
#[derive(Deserialize)]
enum AnyToken {
    Any,
}

impl<'de> Deserialize<'de> for CellConstraint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match CellConstraintWire::deserialize(deserializer)? {
            CellConstraintWire::Any => Ok(Self::Any),
            CellConstraintWire::MustInclude(v) => Ok(Self::MustInclude(v)),
            CellConstraintWire::MustNotInclude(MustNotIncludePayload::Any(_)) => {
                Ok(Self::PlainCell)
            }
            CellConstraintWire::MustNotInclude(MustNotIncludePayload::List(v)) => {
                Ok(Self::MustNotInclude(v))
            }
            CellConstraintWire::PlainCell => Ok(Self::PlainCell),
        }
    }
}
