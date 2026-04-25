//! `EffectSourceChip` — tracks which chip sourced an effect entity.

use bevy::prelude::*;

use crate::prelude::SourceId;

/// Identifies which chip (upgrade) caused this effect entity to be spawned.
///
/// Used for damage attribution and UI display.
/// `None` indicates the effect was not chip-sourced (e.g., from a cell death cascade).
///
/// Tests prefer direct construction (`EffectSourceChip(Some(id))`), since the
/// W5 spec exercises that surface explicitly. The `new()` helper exists for
/// callers that already have an `Option<SourceId>` and prefer the named
/// constructor — its behaviour is covered transitively by the round-trip
/// tests below.
#[derive(Component, Debug, Clone)]
pub struct EffectSourceChip(pub Option<SourceId>);

impl EffectSourceChip {
    /// Constructs from a fully-built `SourceId`.
    /// `None` indicates a non-chip-sourced effect (e.g. cell-death cascade).
    #[must_use]
    pub const fn new(source: Option<SourceId>) -> Self {
        Self(source)
    }

    /// Constructs from the `&str` source passed into `Fireable::fire`.
    /// Empty strings (the W7 "no source" sentinel) map to `None`; non-empty
    /// strings are wrapped in a `SourceId`. Centralizes the construction
    /// pattern previously copy-pasted across every `fire()` impl.
    #[must_use]
    pub fn from_source_str(source: &str) -> Self {
        Self(if source.is_empty() {
            None
        } else {
            Some(SourceId::from(source.to_owned()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chips::definition::Rarity, prelude::SourceIdExt};

    // ── B38: EffectSourceChip(Some(source_id)) round-trip ──

    #[test]
    fn effect_source_chip_holds_some_built_source_id() {
        let id = SourceId::chip("Piercing").rarity(Rarity::Common).build();
        let comp = EffectSourceChip(Some(id.clone()));
        assert_eq!(comp.0, Some(id));
    }

    #[test]
    fn effect_source_chip_with_chip_only_no_rarity_round_trips() {
        let id = SourceId::chip("Piercing").build();
        let comp = EffectSourceChip(Some(id.clone()));
        assert_eq!(comp.0, Some(id));
    }

    #[test]
    fn effect_source_chip_none_round_trips() {
        let comp = EffectSourceChip(None);
        assert_eq!(comp.0, None);
    }
}
