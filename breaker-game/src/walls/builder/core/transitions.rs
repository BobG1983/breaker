//! Entry point, side transitions, `definition()`, optional chainable methods.

use bevy::prelude::*;

use super::types::*;
use crate::{prelude::*, walls::definition::WallDefinition};

// ── Entry point ────────────────────────────────────────────────────────────

impl Wall {
    /// Creates a wall builder in the unconfigured side state.
    #[must_use]
    pub(crate) fn builder() -> WallBuilder<NoSide> {
        WallBuilder {
            side:     NoSide,
            optional: OptionalWallData::default(),
            lifetime: Lifetime::default(),
            visual:   Invisible,
        }
    }
}

// ── Side transitions (NoSide -> specific side) ─────────────────────────────

impl WallBuilder<NoSide> {
    /// Transitions to a Left wall, capturing playfield data.
    #[must_use]
    pub(crate) fn left(self, playfield: &PlayfieldConfig) -> WallBuilder<Left> {
        WallBuilder {
            side:     Left {
                playfield_left: playfield.left(),
                half_height:    playfield.height / 2.0,
            },
            optional: self.optional,
            lifetime: self.lifetime,
            visual:   Invisible,
        }
    }

    /// Transitions to a Right wall, capturing playfield data.
    #[must_use]
    pub(crate) fn right(self, playfield: &PlayfieldConfig) -> WallBuilder<Right> {
        WallBuilder {
            side:     Right {
                playfield_right: playfield.right(),
                half_height:     playfield.height / 2.0,
            },
            optional: self.optional,
            lifetime: self.lifetime,
            visual:   Invisible,
        }
    }

    /// Transitions to a Ceiling wall, capturing playfield data.
    #[must_use]
    pub(crate) fn ceiling(self, playfield: &PlayfieldConfig) -> WallBuilder<Ceiling> {
        WallBuilder {
            side:     Ceiling {
                playfield_top: playfield.top(),
                half_width:    playfield.width / 2.0,
            },
            optional: self.optional,
            lifetime: self.lifetime,
            visual:   Invisible,
        }
    }

    /// Transitions to a Floor wall, capturing playfield data.
    #[must_use]
    pub(crate) fn floor(self, playfield: &PlayfieldConfig) -> WallBuilder<Floor> {
        WallBuilder {
            side:     Floor {
                playfield_bottom: playfield.bottom(),
                half_width:       playfield.width / 2.0,
            },
            optional: self.optional,
            lifetime: self.lifetime,
            visual:   Invisible,
        }
    }
}

// ── Definition and optional chainables (require S: SideData) ───────────────
// Generic over V so these work on both Invisible and Visible builders.

impl<S: SideData, V> WallBuilder<S, V> {
    /// Stores definition values from a `WallDefinition`.
    #[must_use]
    pub(crate) fn definition(mut self, def: &WallDefinition) -> Self {
        self.optional.definition_half_thickness = Some(def.half_thickness);
        self.optional.definition_color_rgb = def.color_rgb;
        if !def.effects.is_empty() {
            self.optional.definition_effects = Some(def.effects.clone());
        }
        self
    }

    /// Overrides the `half_thickness`.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn with_half_thickness(mut self, ht: f32) -> Self {
        self.optional.override_half_thickness = Some(ht);
        self
    }

    /// Overrides the color RGB (test-only — production uses definition).
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn with_color(mut self, rgb: [f32; 3]) -> Self {
        self.optional.override_color_rgb = Some(rgb);
        self
    }

    /// Overrides the effect chains.
    #[must_use]
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "future API: Phase 5j / Shield chip")
    )]
    pub(crate) fn with_effects(mut self, effects: Vec<RootNode>) -> Self {
        self.optional.override_effects = Some(effects);
        self
    }

    /// No-op for self-documentation — wall has no visual components.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn invisible(self) -> Self {
        self
    }
}

// ── Visible transition (Invisible -> Visible) ──────────────────────────────

impl<S: SideData> WallBuilder<S, Invisible> {
    /// Marks the wall as visible with mesh and material.
    #[must_use]
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "test-only until system-param callers exist")
    )]
    pub(crate) fn visible(
        self,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> WallBuilder<S, Visible> {
        let color_rgb = self
            .optional
            .override_color_rgb
            .or(self.optional.definition_color_rgb)
            .unwrap_or([1.0, 1.0, 1.0]);
        let color = crate::shared::color_from_rgb(color_rgb);
        WallBuilder {
            side:     self.side,
            optional: self.optional,
            lifetime: self.lifetime,
            visual:   Visible {
                mesh:     meshes.add(Rectangle::new(1.0, 1.0)),
                material: materials.add(ColorMaterial::from_color(color)),
            },
        }
    }
}

// ── Floor-only lifetime methods ────────────────────────────────────────────
// Generic over V so timed/one_shot work on both Invisible and Visible floors.

#[cfg(test)]
impl<V> WallBuilder<Floor, V> {
    /// Sets the wall lifetime to timed.
    #[must_use]
    pub(crate) const fn timed(mut self, duration: f32) -> Self {
        self.lifetime = Lifetime::Timed(duration);
        self
    }

    /// Sets the wall lifetime to one-shot.
    #[must_use]
    pub(crate) const fn one_shot(mut self) -> Self {
        self.lifetime = Lifetime::OneShot;
        self
    }
}
