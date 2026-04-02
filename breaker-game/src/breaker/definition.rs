//! Breaker definition — RON-deserialized breaker data.

use bevy::{math::curve::easing::EaseFunction, prelude::*};
use serde::Deserialize;

use crate::effect::RootEffect;

// ── Default value functions ─────────────────────────────────────────────────

fn default_bolt_name() -> String {
    "Bolt".to_owned()
}
const fn default_width() -> f32 {
    120.0
}
const fn default_height() -> f32 {
    20.0
}
const fn default_y_position() -> f32 {
    -250.0
}
const fn default_max_speed() -> f32 {
    500.0
}
const fn default_acceleration() -> f32 {
    3000.0
}
const fn default_deceleration() -> f32 {
    2500.0
}
const fn default_decel_ease() -> EaseFunction {
    EaseFunction::QuadraticIn
}
const fn default_decel_ease_strength() -> f32 {
    1.0
}
const fn default_dash_speed_multiplier() -> f32 {
    4.0
}
const fn default_dash_duration() -> f32 {
    0.15
}
const fn default_dash_tilt_angle() -> f32 {
    15.0
}
const fn default_dash_tilt_ease() -> EaseFunction {
    EaseFunction::QuadraticInOut
}
const fn default_brake_tilt_angle() -> f32 {
    25.0
}
const fn default_brake_tilt_duration() -> f32 {
    0.2
}
const fn default_brake_tilt_ease() -> EaseFunction {
    EaseFunction::CubicInOut
}
const fn default_brake_decel_multiplier() -> f32 {
    2.0
}
const fn default_settle_duration() -> f32 {
    0.25
}
const fn default_settle_tilt_ease() -> EaseFunction {
    EaseFunction::CubicOut
}
const fn default_perfect_window() -> f32 {
    0.15
}
const fn default_early_window() -> f32 {
    0.15
}
const fn default_late_window() -> f32 {
    0.15
}
const fn default_perfect_bump_cooldown() -> f32 {
    0.0
}
const fn default_weak_bump_cooldown() -> f32 {
    0.15
}
const fn default_bump_visual_duration() -> f32 {
    0.15
}
const fn default_bump_visual_peak() -> f32 {
    24.0
}
const fn default_bump_visual_peak_fraction() -> f32 {
    0.3
}
const fn default_bump_visual_rise_ease() -> EaseFunction {
    EaseFunction::CubicOut
}
const fn default_bump_visual_fall_ease() -> EaseFunction {
    EaseFunction::QuadraticIn
}
const fn default_reflection_spread() -> f32 {
    75.0
}
/// Default HDR color for breaker rendering (values may exceed 1.0 for bloom).
pub const DEFAULT_COLOR_RGB: [f32; 3] = [0.2, 2.0, 3.0];

const fn default_color_rgb() -> [f32; 3] {
    DEFAULT_COLOR_RGB
}

/// A breaker definition loaded from a RON file.
///
/// All gameplay fields have serde defaults; RON files only need to specify `name`.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct BreakerDefinition {
    /// Display name of the breaker.
    pub name: String,
    /// Name of the bolt definition this breaker uses.
    #[serde(default = "default_bolt_name")]
    pub bolt: String,
    /// Number of lives, if the breaker uses a life pool. None = infinite.
    #[serde(default)]
    pub life_pool: Option<u32>,
    /// All effect chains for this breaker.
    #[serde(default)]
    pub effects: Vec<RootEffect>,

    // ── Dimensions ──────────────────────────────────────────────────────
    /// Full width of the breaker in world units.
    #[serde(default = "default_width")]
    pub width: f32,
    /// Full height of the breaker in world units.
    #[serde(default = "default_height")]
    pub height: f32,
    /// Y position of the breaker at rest.
    #[serde(default = "default_y_position")]
    pub y_position: f32,
    /// Min width clamp. None = computed as 0.5 × width at build time.
    #[serde(default)]
    pub min_w: Option<f32>,
    /// Max width clamp. None = computed as 5.0 × width at build time.
    #[serde(default)]
    pub max_w: Option<f32>,
    /// Min height clamp. None = computed as 0.5 × height at build time.
    #[serde(default)]
    pub min_h: Option<f32>,
    /// Max height clamp. None = computed as 5.0 × height at build time.
    #[serde(default)]
    pub max_h: Option<f32>,

    // ── Movement ────────────────────────────────────────────────────────
    /// Maximum horizontal speed in world units per second.
    #[serde(default = "default_max_speed")]
    pub max_speed: f32,
    /// Horizontal acceleration in world units per second squared.
    #[serde(default = "default_acceleration")]
    pub acceleration: f32,
    /// Horizontal deceleration in world units per second squared.
    #[serde(default = "default_deceleration")]
    pub deceleration: f32,
    /// Easing applied to deceleration based on speed ratio.
    #[serde(default = "default_decel_ease")]
    pub decel_ease: EaseFunction,
    /// Strength of eased deceleration.
    #[serde(default = "default_decel_ease_strength")]
    pub decel_ease_strength: f32,

    // ── Dash ────────────────────────────────────────────────────────────
    /// Dash speed multiplier relative to max speed.
    #[serde(default = "default_dash_speed_multiplier")]
    pub dash_speed_multiplier: f32,
    /// Duration of the dash in seconds.
    #[serde(default = "default_dash_duration")]
    pub dash_duration: f32,
    /// Maximum tilt angle during dash in degrees.
    #[serde(default = "default_dash_tilt_angle")]
    pub dash_tilt_angle: f32,
    /// Easing for dash tilt ramp-up.
    #[serde(default = "default_dash_tilt_ease")]
    pub dash_tilt_ease: EaseFunction,
    /// Maximum tilt angle during brake in degrees.
    #[serde(default = "default_brake_tilt_angle")]
    pub brake_tilt_angle: f32,
    /// Duration of the brake tilt ease in seconds.
    #[serde(default = "default_brake_tilt_duration")]
    pub brake_tilt_duration: f32,
    /// Easing for brake tilt.
    #[serde(default = "default_brake_tilt_ease")]
    pub brake_tilt_ease: EaseFunction,
    /// Brake deceleration multiplier relative to normal deceleration.
    #[serde(default = "default_brake_decel_multiplier")]
    pub brake_decel_multiplier: f32,
    /// Duration of the settle phase in seconds.
    #[serde(default = "default_settle_duration")]
    pub settle_duration: f32,
    /// Easing for settle tilt return to zero.
    #[serde(default = "default_settle_tilt_ease")]
    pub settle_tilt_ease: EaseFunction,

    // ── Bump ────────────────────────────────────────────────────────────
    /// Perfect bump timing window in seconds.
    #[serde(default = "default_perfect_window")]
    pub perfect_window: f32,
    /// Early bump window in seconds.
    #[serde(default = "default_early_window")]
    pub early_window: f32,
    /// Late bump window in seconds.
    #[serde(default = "default_late_window")]
    pub late_window: f32,
    /// Cooldown after a perfect bump in seconds.
    #[serde(default = "default_perfect_bump_cooldown")]
    pub perfect_bump_cooldown: f32,
    /// Cooldown after an early/late bump or whiff in seconds.
    #[serde(default = "default_weak_bump_cooldown")]
    pub weak_bump_cooldown: f32,
    /// Duration of the bump pop animation in seconds.
    #[serde(default = "default_bump_visual_duration")]
    pub bump_visual_duration: f32,
    /// Maximum Y offset at the peak of the bump pop animation.
    #[serde(default = "default_bump_visual_peak")]
    pub bump_visual_peak: f32,
    /// Fraction of bump pop duration spent rising (0.0–1.0).
    #[serde(default = "default_bump_visual_peak_fraction")]
    pub bump_visual_peak_fraction: f32,
    /// Easing for the rise phase of the bump pop.
    #[serde(default = "default_bump_visual_rise_ease")]
    pub bump_visual_rise_ease: EaseFunction,
    /// Easing for the fall phase of the bump pop.
    #[serde(default = "default_bump_visual_fall_ease")]
    pub bump_visual_fall_ease: EaseFunction,

    // ── Spread ──────────────────────────────────────────────────────────
    /// Maximum reflection angle from vertical in degrees.
    #[serde(default = "default_reflection_spread")]
    pub reflection_spread: f32,

    // ── Visual ──────────────────────────────────────────────────────────
    /// RGB values for the breaker HDR color (values may exceed 1.0 for bloom).
    #[serde(default = "default_color_rgb")]
    pub color_rgb: [f32; 3],
}

impl Default for BreakerDefinition {
    fn default() -> Self {
        Self {
            name: String::new(),
            bolt: default_bolt_name(),
            life_pool: None,
            effects: vec![],
            width: default_width(),
            height: default_height(),
            y_position: default_y_position(),
            min_w: None,
            max_w: None,
            min_h: None,
            max_h: None,
            max_speed: default_max_speed(),
            acceleration: default_acceleration(),
            deceleration: default_deceleration(),
            decel_ease: default_decel_ease(),
            decel_ease_strength: default_decel_ease_strength(),
            dash_speed_multiplier: default_dash_speed_multiplier(),
            dash_duration: default_dash_duration(),
            dash_tilt_angle: default_dash_tilt_angle(),
            dash_tilt_ease: default_dash_tilt_ease(),
            brake_tilt_angle: default_brake_tilt_angle(),
            brake_tilt_duration: default_brake_tilt_duration(),
            brake_tilt_ease: default_brake_tilt_ease(),
            brake_decel_multiplier: default_brake_decel_multiplier(),
            settle_duration: default_settle_duration(),
            settle_tilt_ease: default_settle_tilt_ease(),
            perfect_window: default_perfect_window(),
            early_window: default_early_window(),
            late_window: default_late_window(),
            perfect_bump_cooldown: default_perfect_bump_cooldown(),
            weak_bump_cooldown: default_weak_bump_cooldown(),
            bump_visual_duration: default_bump_visual_duration(),
            bump_visual_peak: default_bump_visual_peak(),
            bump_visual_peak_fraction: default_bump_visual_peak_fraction(),
            bump_visual_rise_ease: default_bump_visual_rise_ease(),
            bump_visual_fall_ease: default_bump_visual_fall_ease(),
            reflection_spread: default_reflection_spread(),
            color_rgb: default_color_rgb(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 1: BreakerDefinition parses RON with explicit bolt field ──

    #[test]
    fn breaker_definition_parses_ron_with_explicit_bolt_field() {
        let ron_str = r#"(
            name: "Aegis",
            bolt: "HeavyBolt",
            life_pool: Some(3),
            effects: [],
        )"#;
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("RON with explicit bolt field should parse");
        assert_eq!(def.bolt, "HeavyBolt");
        assert_eq!(def.name, "Aegis");
        assert_eq!(def.life_pool, Some(3));
    }

    #[test]
    fn breaker_definition_parses_ron_with_empty_bolt_field() {
        let ron_str = r#"(
            name: "Aegis",
            bolt: "",
            life_pool: Some(3),
            effects: [],
        )"#;
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("RON with empty bolt field should parse");
        assert_eq!(def.bolt, "");
    }

    // ── Behavior 2: BreakerDefinition serde default for bolt field is "Bolt" ──

    #[test]
    fn breaker_definition_defaults_bolt_to_bolt_when_omitted() {
        let ron_str = r#"(
            name: "Chrono",
            life_pool: None,
            effects: [],
        )"#;
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("RON without bolt field should parse");
        assert_eq!(def.bolt, "Bolt");
    }

    #[test]
    fn breaker_definition_defaults_bolt_with_all_other_fields_present() {
        let ron_str = r#"(
            name: "Aegis",
            life_pool: Some(3),
            effects: [],
        )"#;
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("RON with all other fields should parse");
        assert_eq!(
            def.bolt, "Bolt",
            "bolt should default to \"Bolt\" when omitted, even with all other fields present"
        );
    }

    // ── Behavior 3: Existing aegis.breaker.ron parses with bolt defaulting to "Bolt" ──

    #[test]
    fn aegis_breaker_ron_parses_with_bolt_defaulting_to_bolt() {
        let ron_str = include_str!("../../assets/breakers/aegis.breaker.ron");
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("aegis.breaker.ron should parse");
        assert_eq!(def.name, "Aegis");
        assert_eq!(def.bolt, "Bolt");
        assert_eq!(def.life_pool, Some(3));
    }

    // ── Behavior 4: Existing chrono.breaker.ron parses with bolt defaulting to "Bolt" ──

    #[test]
    fn chrono_breaker_ron_parses_with_bolt_defaulting_to_bolt() {
        let ron_str = include_str!("../../assets/breakers/chrono.breaker.ron");
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("chrono.breaker.ron should parse");
        assert_eq!(def.name, "Chrono");
        assert_eq!(def.bolt, "Bolt");
        assert_eq!(def.life_pool, None);
    }

    // ── Behavior 5: Existing prism.breaker.ron parses with bolt defaulting to "Bolt" ──

    #[test]
    fn prism_breaker_ron_parses_with_bolt_defaulting_to_bolt() {
        let ron_str = include_str!("../../assets/breakers/prism.breaker.ron");
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("prism.breaker.ron should parse");
        assert_eq!(def.name, "Prism");
        assert_eq!(def.bolt, "Bolt");
        assert_eq!(def.life_pool, None);
    }

    // ── Behavior 6: BreakerDefinition clone preserves bolt field ──

    #[test]
    fn breaker_definition_clone_preserves_bolt_field() {
        let ron_str = r#"(name: "TestBreaker", bolt: "HeavyBolt", effects: [])"#;
        let def: BreakerDefinition = ron::de::from_str(ron_str).unwrap();
        let cloned = def.clone();
        assert_eq!(cloned.bolt, "HeavyBolt");
        assert_eq!(cloned.name, "TestBreaker");
        // Verify original is still intact after clone
        assert_eq!(def.bolt, "HeavyBolt");
    }

    #[test]
    fn breaker_definition_clone_preserves_default_bolt_value() {
        let ron_str = r#"(name: "TestBreaker", effects: [])"#;
        let def: BreakerDefinition = ron::de::from_str(ron_str).unwrap();
        let cloned = def.clone();
        assert_eq!(cloned.bolt, "Bolt");
        // Verify original is still intact after clone
        assert_eq!(def.bolt, "Bolt");
    }
}
