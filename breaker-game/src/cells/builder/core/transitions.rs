//! Entry point, typestate transitions, and optional chainable methods.

use bevy::prelude::*;

use super::types::*;
use crate::cells::{
    components::{Cell, CellDamageVisuals},
    definition::CellTypeDefinition,
};
#[cfg(test)]
use crate::{
    cells::{
        definition::{CellBehavior, Toughness},
        resources::ToughnessConfig,
    },
    effect::RootEffect,
};

// ── Entry point ─────────────────────────────────────────────────────────────

impl Cell {
    /// Creates a cell builder in the unconfigured state.
    #[must_use]
    pub(crate) fn builder() -> CellBuilder<NoPosition, NoDimensions, NoHealth, Unvisual> {
        CellBuilder {
            position: NoPosition,
            dimensions: NoDimensions,
            health: NoHealth,
            visual: Unvisual,
            optional: OptionalCellData::default(),
        }
    }
}

// ── Position transition ─────────────────────────────────────────────────────

impl<D, H, V> CellBuilder<NoPosition, D, H, V> {
    #[must_use]
    pub(crate) fn position(self, pos: Vec2) -> CellBuilder<HasPosition, D, H, V> {
        CellBuilder {
            position: HasPosition { pos },
            dimensions: self.dimensions,
            health: self.health,
            visual: self.visual,
            optional: self.optional,
        }
    }
}

// ── Dimensions transition ───────────────────────────────────────────────────

impl<P, H, V> CellBuilder<P, NoDimensions, H, V> {
    #[must_use]
    pub(crate) fn dimensions(self, width: f32, height: f32) -> CellBuilder<P, HasDimensions, H, V> {
        CellBuilder {
            position: self.position,
            dimensions: HasDimensions { width, height },
            health: self.health,
            visual: self.visual,
            optional: self.optional,
        }
    }
}

// ── Health transition (explicit — test-only, production uses .definition()) ──

#[cfg(test)]
impl<P, D, V> CellBuilder<P, D, NoHealth, V> {
    #[must_use]
    pub(crate) fn hp(self, value: f32) -> CellBuilder<P, D, HasHealth, V> {
        CellBuilder {
            position: self.position,
            dimensions: self.dimensions,
            health: HasHealth { hp: value },
            visual: self.visual,
            optional: self.optional,
        }
    }
}

// ── Definition convenience ──────────────────────────────────────────────────

impl<P, D, V> CellBuilder<P, D, NoHealth, V> {
    /// Configure the cell from a `CellTypeDefinition`.
    ///
    /// Transitions Health dimension and stores definition-derived params.
    #[must_use]
    pub(crate) fn definition(
        mut self,
        def: &CellTypeDefinition,
    ) -> CellBuilder<P, D, HasHealth, V> {
        self.optional.definition_params = Some(CellDefinitionParams {
            alias: def.alias.clone(),
            required_to_clear: def.required_to_clear,
            damage_visuals: CellDamageVisuals {
                hdr_base: def.damage_hdr_base,
                green_min: def.damage_green_min,
                blue_range: def.damage_blue_range,
                blue_base: def.damage_blue_base,
            },
            behaviors: def.behaviors.clone().unwrap_or_default(),
            effects: def.effects.clone(),
            color_rgb: def.color_rgb,
        });
        CellBuilder {
            position: self.position,
            dimensions: self.dimensions,
            health: HasHealth {
                hp: def.toughness.default_base_hp(),
            },
            visual: self.visual,
            optional: self.optional,
        }
    }
}

// ── Visual transitions ──────────────────────────────────────────────────────

impl<P, D, H> CellBuilder<P, D, H, Unvisual> {
    /// Configures the cell for rendered mode with mesh and material.
    #[must_use]
    pub(crate) fn rendered(
        mut self,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> CellBuilder<P, D, H, Rendered> {
        let color_rgb = self
            .optional
            .color_rgb
            .or_else(|| {
                self.optional
                    .definition_params
                    .as_ref()
                    .map(|d| d.color_rgb)
            })
            .unwrap_or(DEFAULT_CELL_COLOR_RGB);
        let color = Color::linear_rgb(color_rgb[0], color_rgb[1], color_rgb[2]);

        // Pre-compute guardian visual handles if guarded data exists.
        if let Some(ref mut guarded_data) = self.optional.guarded_data {
            let config = &guarded_data.guardian_config;
            let guardian_color = Color::linear_rgb(
                config.color_rgb[0],
                config.color_rgb[1],
                config.color_rgb[2],
            );
            guarded_data.guardian_visuals = Some((
                meshes.add(Rectangle::new(1.0, 1.0)),
                materials.add(ColorMaterial::from_color(guardian_color)),
            ));
        }

        CellBuilder {
            position: self.position,
            dimensions: self.dimensions,
            health: self.health,
            visual: Rendered {
                mesh: meshes.add(Rectangle::new(1.0, 1.0)),
                material: materials.add(ColorMaterial::from_color(color)),
            },
            optional: self.optional,
        }
    }

    /// Configures the cell for headless mode (test-only — production uses rendered).
    #[cfg(test)]
    #[must_use]
    pub(crate) fn headless(self) -> CellBuilder<P, D, H, Headless> {
        CellBuilder {
            position: self.position,
            dimensions: self.dimensions,
            health: self.health,
            visual: Headless,
            optional: self.optional,
        }
    }
}

// ── Tier HP transition ─────────────────────────────────────────────────────

#[cfg(test)]
impl<P, D, V> CellBuilder<P, D, NoHealth, V> {
    /// Sets HP from toughness config and tier context.
    /// Uses the stored toughness (from `.toughness()`) or defaults to Standard.
    #[must_use]
    pub(crate) fn tier_hp(
        self,
        config: &ToughnessConfig,
        tier: u32,
        position_in_tier: u32,
    ) -> CellBuilder<P, D, HasHealth, V> {
        let toughness = self.optional.toughness.unwrap_or_default();
        let hp = config.hp_for(toughness, tier, position_in_tier);
        CellBuilder {
            position: self.position,
            dimensions: self.dimensions,
            health: HasHealth { hp },
            visual: self.visual,
            optional: self.optional,
        }
    }
}

// ── Optional chainable methods (any typestate) ──────────────────────────────

impl<P, D, H, V> CellBuilder<P, D, H, V> {
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn toughness(mut self, toughness: Toughness) -> Self {
        self.optional.toughness = Some(toughness);
        self
    }

    #[must_use]
    pub(crate) fn alias(mut self, alias: String) -> Self {
        self.optional.alias = Some(alias);
        self
    }

    #[must_use]
    pub(crate) fn locked(mut self, entities: Vec<Entity>) -> Self {
        self.optional.locked_entities = Some(entities);
        self
    }

    #[must_use]
    pub(crate) fn guarded(mut self, slots: Vec<u8>, config: GuardianSpawnConfig) -> Self {
        self.optional.guarded_data = Some(GuardedSpawnData {
            slots,
            guardian_config: config,
            guardian_visuals: None,
        });
        self
    }
}

// ── Test-only optional methods (production uses .definition() for these) ─────

#[cfg(test)]
impl<P, D, H, V> CellBuilder<P, D, H, V> {
    #[must_use]
    pub(crate) const fn required_to_clear(mut self, value: bool) -> Self {
        self.optional.required_to_clear = Some(value);
        self
    }

    #[must_use]
    pub(crate) const fn damage_visuals(mut self, visuals: CellDamageVisuals) -> Self {
        self.optional.damage_visuals = Some(visuals);
        self
    }

    #[must_use]
    pub(crate) fn with_effects(mut self, effects: Vec<RootEffect>) -> Self {
        self.optional.effects = Some(effects);
        self
    }

    #[must_use]
    pub(crate) fn with_behavior(mut self, behavior: CellBehavior) -> Self {
        self.optional.behaviors.push(behavior);
        self
    }

    #[must_use]
    pub(crate) const fn color_rgb(mut self, rgb: [f32; 3]) -> Self {
        self.optional.color_rgb = Some(rgb);
        self
    }
}

// ── HasHealth-specific optional method ──────────────────────────────────────

impl<P, D, V> CellBuilder<P, D, HasHealth, V> {
    #[must_use]
    pub(crate) const fn override_hp(mut self, hp: f32) -> Self {
        self.optional.override_hp = Some(hp);
        self
    }
}
