//! Entry point, dimension transitions, `definition()`, optional/override methods.

use bevy::prelude::*;

use super::types::*;
use crate::{breaker::definition::BreakerDefinition, effect_v3::types::RootNode, prelude::*};

// ── Entry point ─────────────────────────────────────────────────────────────

impl Breaker {
    /// Creates a breaker builder in the unconfigured state.
    #[must_use]
    pub fn builder()
    -> BreakerBuilder<NoDimensions, NoMovement, NoDashing, NoSpread, NoBump, Unvisual, NoRole> {
        BreakerBuilder {
            dimensions: NoDimensions,
            movement:   NoMovement,
            dashing:    NoDashing,
            spread:     NoSpread,
            bump:       NoBump,
            visual:     Unvisual,
            role:       NoRole,
            optional:   OptionalBreakerData::default(),
        }
    }
}

// ── Dimensions transition ───────────────────────────────────────────────────

impl<Mv, Da, Sp, Bm, V, R> BreakerBuilder<NoDimensions, Mv, Da, Sp, Bm, V, R> {
    /// Sets width, height, and `y_position`. Min/max default to 0.5x and 5x base.
    pub fn dimensions(
        self,
        width: f32,
        height: f32,
        y_position: f32,
    ) -> BreakerBuilder<HasDimensions, Mv, Da, Sp, Bm, V, R> {
        BreakerBuilder {
            dimensions: HasDimensions {
                width,
                height,
                y_position,
                min_w: width * 0.5,
                max_w: width * 5.0,
                min_h: height * 0.5,
                max_h: height * 5.0,
            },
            movement:   self.movement,
            dashing:    self.dashing,
            spread:     self.spread,
            bump:       self.bump,
            visual:     self.visual,
            role:       self.role,
            optional:   self.optional,
        }
    }
}

// ── Movement transition ─────────────────────────────────────────────────────

impl<D, Da, Sp, Bm, V, R> BreakerBuilder<D, NoMovement, Da, Sp, Bm, V, R> {
    /// Configures movement parameters: speed, acceleration, deceleration.
    pub fn movement(
        self,
        settings: MovementSettings,
    ) -> BreakerBuilder<D, HasMovement, Da, Sp, Bm, V, R> {
        BreakerBuilder {
            dimensions: self.dimensions,
            movement:   HasMovement {
                max_speed:           settings.max_speed,
                acceleration:        settings.acceleration,
                deceleration:        settings.deceleration,
                decel_ease:          settings.decel_ease,
                decel_ease_strength: settings.decel_ease_strength,
            },
            dashing:    self.dashing,
            spread:     self.spread,
            bump:       self.bump,
            visual:     self.visual,
            role:       self.role,
            optional:   self.optional,
        }
    }
}

// ── Dashing transition ──────────────────────────────────────────────────────

impl<D, Mv, Sp, Bm, V, R> BreakerBuilder<D, Mv, NoDashing, Sp, Bm, V, R> {
    /// Configures dash, brake, and settle parameters.
    pub fn dashing(
        self,
        settings: DashSettings,
    ) -> BreakerBuilder<D, Mv, HasDashing, Sp, Bm, V, R> {
        BreakerBuilder {
            dimensions: self.dimensions,
            movement:   self.movement,
            dashing:    HasDashing { settings },
            spread:     self.spread,
            bump:       self.bump,
            visual:     self.visual,
            role:       self.role,
            optional:   self.optional,
        }
    }
}

// ── Spread transition ───────────────────────────────────────────────────────

impl<D, Mv, Da, Bm, V, R> BreakerBuilder<D, Mv, Da, NoSpread, Bm, V, R> {
    /// Sets the reflection spread angle in degrees.
    pub fn spread(self, degrees: f32) -> BreakerBuilder<D, Mv, Da, HasSpread, Bm, V, R> {
        BreakerBuilder {
            dimensions: self.dimensions,
            movement:   self.movement,
            dashing:    self.dashing,
            spread:     HasSpread {
                spread_degrees: degrees,
            },
            bump:       self.bump,
            visual:     self.visual,
            role:       self.role,
            optional:   self.optional,
        }
    }
}

// ── Bump transition ─────────────────────────────────────────────────────────

impl<D, Mv, Da, Sp, V, R> BreakerBuilder<D, Mv, Da, Sp, NoBump, V, R> {
    /// Configures bump timing windows, cooldowns, and feedback animation.
    pub fn bump(self, settings: BumpSettings) -> BreakerBuilder<D, Mv, Da, Sp, HasBump, V, R> {
        BreakerBuilder {
            dimensions: self.dimensions,
            movement:   self.movement,
            dashing:    self.dashing,
            spread:     self.spread,
            bump:       HasBump { settings },
            visual:     self.visual,
            role:       self.role,
            optional:   self.optional,
        }
    }
}

// ── Visual transitions ──────────────────────────────────────────────────────

impl<D, Mv, Da, Sp, Bm, R> BreakerBuilder<D, Mv, Da, Sp, Bm, Unvisual, R> {
    /// Configures the breaker for rendered mode with mesh and material.
    pub fn rendered(
        self,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> BreakerBuilder<D, Mv, Da, Sp, Bm, Rendered, R> {
        let color_rgb = self
            .optional
            .color_rgb
            .unwrap_or(crate::breaker::definition::DEFAULT_COLOR_RGB);
        let color = crate::shared::color_from_rgb(color_rgb);
        BreakerBuilder {
            dimensions: self.dimensions,
            movement:   self.movement,
            dashing:    self.dashing,
            spread:     self.spread,
            bump:       self.bump,
            visual:     Rendered {
                mesh:     meshes.add(Rectangle::new(1.0, 1.0)),
                material: materials.add(ColorMaterial::from_color(color)),
            },
            role:       self.role,
            optional:   self.optional,
        }
    }

    /// Configures the breaker for headless mode (no rendering components).
    pub fn headless(self) -> BreakerBuilder<D, Mv, Da, Sp, Bm, Headless, R> {
        BreakerBuilder {
            dimensions: self.dimensions,
            movement:   self.movement,
            dashing:    self.dashing,
            spread:     self.spread,
            bump:       self.bump,
            visual:     Headless,
            role:       self.role,
            optional:   self.optional,
        }
    }
}

// ── Role transitions ────────────────────────────────────────────────────────

impl<D, Mv, Da, Sp, Bm, V> BreakerBuilder<D, Mv, Da, Sp, Bm, V, NoRole> {
    /// Sets the breaker role to primary (persists across nodes).
    pub fn primary(self) -> BreakerBuilder<D, Mv, Da, Sp, Bm, V, Primary> {
        BreakerBuilder {
            dimensions: self.dimensions,
            movement:   self.movement,
            dashing:    self.dashing,
            spread:     self.spread,
            bump:       self.bump,
            visual:     self.visual,
            role:       Primary,
            optional:   self.optional,
        }
    }

    /// Sets the breaker role to extra (cleaned up on node exit).
    pub fn extra(self) -> BreakerBuilder<D, Mv, Da, Sp, Bm, V, Extra> {
        BreakerBuilder {
            dimensions: self.dimensions,
            movement:   self.movement,
            dashing:    self.dashing,
            spread:     self.spread,
            bump:       self.bump,
            visual:     self.visual,
            role:       Extra,
            optional:   self.optional,
        }
    }
}

// ── definition() convenience ────────────────────────────────────────────────

impl<V, R> BreakerBuilder<NoDimensions, NoMovement, NoDashing, NoSpread, NoBump, V, R> {
    /// Configure the breaker from a `BreakerDefinition`.
    ///
    /// Transitions Dimensions, Movement, Dashing, Spread, and Bump dimensions
    /// in one call. Also stores lives, effects, and `color_rgb` in optional data.
    ///
    /// Call `.definition()` **before** `.with_*()` overrides and `.rendered()` —
    /// it overwrites lives, effects, and color from the definition.
    pub fn definition(
        mut self,
        def: &BreakerDefinition,
    ) -> BreakerBuilder<HasDimensions, HasMovement, HasDashing, HasSpread, HasBump, V, R> {
        // Store optional data from definition
        self.optional.lives = def
            .life_pool
            .map_or(LivesSetting::Infinite, LivesSetting::Count);
        self.optional.effects = Some(def.effects.clone());
        self.optional.bolt_lost = Some(def.bolt_lost.clone());
        self.optional.salvo_hit = Some(def.salvo_hit.clone());
        self.optional.color_rgb = Some(def.color_rgb);

        BreakerBuilder {
            dimensions: HasDimensions {
                width:      def.width,
                height:     def.height,
                y_position: def.y_position,
                min_w:      def.min_w.unwrap_or(def.width * 0.5),
                max_w:      def.max_w.unwrap_or(def.width * 5.0),
                min_h:      def.min_h.unwrap_or(def.height * 0.5),
                max_h:      def.max_h.unwrap_or(def.height * 5.0),
            },
            movement:   HasMovement {
                max_speed:           def.max_speed,
                acceleration:        def.acceleration,
                deceleration:        def.deceleration,
                decel_ease:          def.decel_ease,
                decel_ease_strength: def.decel_ease_strength,
            },
            dashing:    HasDashing {
                settings: DashSettings {
                    dash:   DashParams {
                        speed_multiplier: def.dash_speed_multiplier,
                        duration:         def.dash_duration,
                        tilt_angle:       def.dash_tilt_angle,
                        tilt_ease:        def.dash_tilt_ease,
                    },
                    brake:  BrakeParams {
                        tilt_angle:       def.brake_tilt_angle,
                        tilt_duration:    def.brake_tilt_duration,
                        tilt_ease:        def.brake_tilt_ease,
                        decel_multiplier: def.brake_decel_multiplier,
                    },
                    settle: SettleParams {
                        duration:  def.settle_duration,
                        tilt_ease: def.settle_tilt_ease,
                    },
                },
            },
            spread:     HasSpread {
                spread_degrees: def.reflection_spread,
            },
            bump:       HasBump {
                settings: BumpSettings {
                    perfect_window:   def.perfect_window,
                    early_window:     def.early_window,
                    late_window:      def.late_window,
                    perfect_cooldown: def.perfect_bump_cooldown,
                    weak_cooldown:    def.weak_bump_cooldown,
                    feedback:         BumpFeedbackSettings {
                        duration:      def.bump_visual_duration,
                        peak:          def.bump_visual_peak,
                        peak_fraction: def.bump_visual_peak_fraction,
                        rise_ease:     def.bump_visual_rise_ease,
                        fall_ease:     def.bump_visual_fall_ease,
                    },
                },
            },
            visual:     self.visual,
            role:       self.role,
            optional:   self.optional,
        }
    }
}

// ── Optional chainable methods (any typestate) ──────────────────────────────

impl<D, Mv, Da, Sp, Bm, V, R> BreakerBuilder<D, Mv, Da, Sp, Bm, V, R> {
    /// Sets the life pool. None = infinite lives, Some(n) = n lives.
    #[must_use]
    pub const fn with_lives(mut self, lives: Option<u32>) -> Self {
        self.optional.lives = match lives {
            Some(n) => LivesSetting::Count(n),
            None => LivesSetting::Infinite,
        };
        self
    }

    /// Sets the effect chains.
    #[must_use]
    pub fn with_effects(mut self, effects: Vec<RootNode>) -> Self {
        self.optional.effects = Some(effects);
        self
    }

    /// Sets the color RGB (HDR values, may exceed 1.0 for bloom).
    #[must_use]
    pub const fn with_color(mut self, rgb: [f32; 3]) -> Self {
        self.optional.color_rgb = Some(rgb);
        self
    }
}

// ── Override methods (require relevant dimension to be satisfied) ────────

impl<Mv, Da, Sp, Bm, V, R> BreakerBuilder<HasDimensions, Mv, Da, Sp, Bm, V, R> {
    /// Overrides the width set by `.dimensions()` or `.definition()`.
    #[must_use]
    pub const fn with_width(mut self, w: f32) -> Self {
        self.optional.override_width = Some(w);
        self
    }

    /// Overrides the height set by `.dimensions()` or `.definition()`.
    #[must_use]
    pub const fn with_height(mut self, h: f32) -> Self {
        self.optional.override_height = Some(h);
        self
    }

    /// Overrides the `y_position` set by `.dimensions()` or `.definition()`.
    #[must_use]
    pub const fn with_y_position(mut self, y: f32) -> Self {
        self.optional.override_y_position = Some(y);
        self
    }

    /// Sets the spawn position, overriding the definition's `y_position` and the
    /// default `x = 0.0`. If not called, x defaults to `0.0` and y defaults to
    /// the value from `.dimensions()` or `.definition()`.
    #[must_use]
    pub const fn at_position(mut self, pos: Vec2) -> Self {
        self.optional.override_x_position = Some(pos.x);
        self.optional.override_y_position = Some(pos.y);
        self
    }
}

impl<D, Da, Sp, Bm, V, R> BreakerBuilder<D, HasMovement, Da, Sp, Bm, V, R> {
    /// Overrides the `max_speed` set by `.movement()` or `.definition()`.
    #[must_use]
    pub const fn with_max_speed(mut self, speed: f32) -> Self {
        self.optional.override_max_speed = Some(speed);
        self
    }
}

impl<D, Mv, Da, Bm, V, R> BreakerBuilder<D, Mv, Da, HasSpread, Bm, V, R> {
    /// Overrides the reflection spread set by `.spread()` or `.definition()`.
    /// Value is in degrees (will be converted to radians in `build()`).
    #[must_use]
    pub const fn with_reflection_spread(mut self, degrees: f32) -> Self {
        self.optional.override_reflection_spread = Some(degrees);
        self
    }
}
