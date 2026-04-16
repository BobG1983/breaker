//! Entry point, typestate transitions, and optional chainable methods.

use bevy::prelude::*;

use super::types::*;
use crate::cells::{
    components::{Cell, CellDamageVisuals},
    definition::{CellBehavior, CellTypeDefinition},
};
#[cfg(test)]
use crate::{
    cells::{
        behaviors::{armored::components::ArmorDirection, phantom::components::PhantomPhase},
        definition::{AttackPattern, Toughness},
        resources::ToughnessConfig,
    },
    effect_v3::types::RootNode,
};

// ── Entry point ─────────────────────────────────────────────────────────────

impl Cell {
    /// Creates a cell builder in the unconfigured state.
    #[must_use]
    pub(crate) fn builder() -> CellBuilder<NoPosition, NoDimensions, NoHealth, Unvisual> {
        CellBuilder {
            position:   NoPosition,
            dimensions: NoDimensions,
            health:     NoHealth,
            visual:     Unvisual,
            optional:   OptionalCellData::default(),
        }
    }
}

// ── Position transition ─────────────────────────────────────────────────────

impl<D, H, V> CellBuilder<NoPosition, D, H, V> {
    #[must_use]
    pub(crate) fn position(self, pos: Vec2) -> CellBuilder<HasPosition, D, H, V> {
        CellBuilder {
            position:   HasPosition { pos },
            dimensions: self.dimensions,
            health:     self.health,
            visual:     self.visual,
            optional:   self.optional,
        }
    }
}

// ── Dimensions transition ───────────────────────────────────────────────────

impl<P, H, V> CellBuilder<P, NoDimensions, H, V> {
    #[must_use]
    pub(crate) fn dimensions(self, width: f32, height: f32) -> CellBuilder<P, HasDimensions, H, V> {
        CellBuilder {
            position:   self.position,
            dimensions: HasDimensions { width, height },
            health:     self.health,
            visual:     self.visual,
            optional:   self.optional,
        }
    }
}

// ── Health transition (explicit — test-only, production uses .definition()) ──

#[cfg(test)]
impl<P, D, V> CellBuilder<P, D, NoHealth, V> {
    #[must_use]
    pub(crate) fn hp(self, value: f32) -> CellBuilder<P, D, HasHealth, V> {
        CellBuilder {
            position:   self.position,
            dimensions: self.dimensions,
            health:     HasHealth { hp: value },
            visual:     self.visual,
            optional:   self.optional,
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
            alias:             def.alias.clone(),
            required_to_clear: def.required_to_clear,
            damage_visuals:    CellDamageVisuals {
                hdr_base:   def.damage_hdr_base,
                green_min:  def.damage_green_min,
                blue_range: def.damage_blue_range,
                blue_base:  def.damage_blue_base,
            },
            behaviors:         def.behaviors.clone().unwrap_or_default(),
            effects:           def.effects.clone(),
            color_rgb:         def.color_rgb,
        });
        CellBuilder {
            position:   self.position,
            dimensions: self.dimensions,
            health:     HasHealth {
                hp: def.toughness.default_base_hp(),
            },
            visual:     self.visual,
            optional:   self.optional,
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
            position:   self.position,
            dimensions: self.dimensions,
            health:     self.health,
            visual:     Rendered {
                mesh:     meshes.add(Rectangle::new(1.0, 1.0)),
                material: materials.add(ColorMaterial::from_color(color)),
            },
            optional:   self.optional,
        }
    }

    /// Configures the cell for headless mode (test-only — production uses rendered).
    #[cfg(test)]
    #[must_use]
    pub(crate) fn headless(self) -> CellBuilder<P, D, H, Headless> {
        CellBuilder {
            position:   self.position,
            dimensions: self.dimensions,
            health:     self.health,
            visual:     Headless,
            optional:   self.optional,
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
            position:   self.position,
            dimensions: self.dimensions,
            health:     HasHealth { hp },
            visual:     self.visual,
            optional:   self.optional,
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

    /// Adds a volatile detonation behavior that fires on cell death.
    ///
    /// Pushes `CellBehavior::Volatile { damage, radius }` onto the optional
    /// behaviors list. At spawn time, the match arm in `spawn_inner()` inserts
    /// the `VolatileCell` marker and stamps a `BoundEffects` entry keyed
    /// `"volatile"` whose tree fires an explosion on `Trigger::Died`.
    ///
    /// Test-only ergonomics — production cells acquire Volatile behavior via
    /// `.definition(&def)` from the RON definition's `behaviors:` field.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn volatile(mut self, damage: f32, radius: f32) -> Self {
        self.optional
            .behaviors
            .push(CellBehavior::Volatile { damage, radius });
        self
    }

    /// Adds a sequence behavior for this cell, placing it in `group` at
    /// `position`.
    ///
    /// Pushes `CellBehavior::Sequence { group, position }` onto the optional
    /// behaviors list. At spawn time, the match arm in `spawn_inner()` will
    /// insert `(SequenceCell, SequenceGroup(group), SequencePosition(position))`.
    /// `SequenceActive` is inserted at `OnEnter(NodeState::Playing)` by
    /// `init_sequence_groups`, NOT at spawn time.
    ///
    /// Production caller: `spawn_cells_from_grid` pass 1 resolves per-cell
    /// `(group, position)` from `NodeLayout.sequences` and invokes this method
    /// on the builder. Unlike `Regen`, `Guarded`, or `Volatile` whose config
    /// comes from the RON cell-type definition, Sequence group/position varies
    /// per layout placement — the cell type only signals "this type can be
    /// used as a sequence member", while the layout assigns membership.
    #[must_use]
    pub(crate) fn sequence(mut self, group: u32, position: u32) -> Self {
        self.optional
            .behaviors
            .push(CellBehavior::Sequence { group, position });
        self
    }

    /// Adds an armored behavior for this cell with facing defaulted to
    /// `ArmorDirection::Bottom` (armor plates face the breaker; weak point
    /// is the top of the cell).
    ///
    /// Test-only ergonomics — production cells acquire Armored behavior via
    /// `.definition(&def)` from the RON definition's `behaviors:` field.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn armored(mut self, value: u8) -> Self {
        self.optional.behaviors.push(CellBehavior::Armored {
            value,
            facing: ArmorDirection::Bottom,
        });
        self
    }

    /// Adds an armored behavior with an explicit facing direction.
    ///
    /// Test-only ergonomics — production cells acquire Armored behavior via
    /// `.definition(&def)` from the RON definition's `behaviors:` field.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn armored_facing(mut self, value: u8, facing: ArmorDirection) -> Self {
        self.optional
            .behaviors
            .push(CellBehavior::Armored { value, facing });
        self
    }

    /// Adds a magnetic behavior with the given radius and strength.
    ///
    /// Pushes `CellBehavior::Magnetic { radius, strength }` onto the optional
    /// behaviors list. At spawn time, the match arm in `spawn_inner()` inserts
    /// the `MagneticCell` marker and `MagneticField { radius, strength }`.
    ///
    /// Test-only ergonomics — production cells acquire Magnetic behavior via
    /// `.definition(&def)` from the RON definition's `behaviors:` field.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn magnetic(mut self, radius: f32, strength: f32) -> Self {
        self.optional
            .behaviors
            .push(CellBehavior::Magnetic { radius, strength });
        self
    }

    /// Adds a survival turret behavior with the given pattern and
    /// self-destruct timer.
    ///
    /// Test-only ergonomics — production cells acquire Survival behavior
    /// via `.definition(&def)` from the RON definition's `behaviors:` field.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn survival(mut self, pattern: AttackPattern, timer_secs: f32) -> Self {
        self.optional.behaviors.push(CellBehavior::Survival {
            pattern,
            timer_secs,
        });
        self
    }

    /// Adds a permanent survival turret behavior with the given pattern.
    /// No self-destruct timer (boss variant).
    ///
    /// Test-only ergonomics — production cells acquire `SurvivalPermanent`
    /// behavior via `.definition(&def)` from the RON definition's
    /// `behaviors:` field.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn survival_permanent(mut self, pattern: AttackPattern) -> Self {
        self.optional
            .behaviors
            .push(CellBehavior::SurvivalPermanent { pattern });
        self
    }

    /// Adds a phantom behavior with default timing (`cycle_secs=3.0`,
    /// `telegraph_secs=0.5`) and the given starting phase.
    ///
    /// Test-only ergonomics — production cells acquire Phantom behavior via
    /// `.definition(&def)` from the RON definition's `behaviors:` field.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn phantom(mut self, starting_phase: PhantomPhase) -> Self {
        self.optional.behaviors.push(CellBehavior::Phantom {
            cycle_secs: 3.0,
            telegraph_secs: 0.5,
            starting_phase,
        });
        self
    }

    /// Adds a phantom behavior with explicit timing and starting phase.
    ///
    /// Test-only ergonomics — production cells acquire Phantom behavior via
    /// `.definition(&def)` from the RON definition's `behaviors:` field.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn phantom_config(
        mut self,
        cycle_secs: f32,
        telegraph_secs: f32,
        starting_phase: PhantomPhase,
    ) -> Self {
        self.optional.behaviors.push(CellBehavior::Phantom {
            cycle_secs,
            telegraph_secs,
            starting_phase,
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
    pub(crate) fn with_effects(mut self, effects: Vec<RootNode>) -> Self {
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
