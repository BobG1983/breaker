use crate::{
    effect::{EffectKind, EffectNode, RootEffect, Target, Trigger},
    shared::PlayfieldConfig,
    wall::definition::WallDefinition,
};

/// Creates a `WallDefinition` from RON with only a name field.
/// Gets all serde defaults: `half_thickness`: 90.0, `color_rgb`: None, effects: [].
pub(super) fn test_wall_definition() -> WallDefinition {
    ron::de::from_str(
        r#"(
            name: "TestWall",
        )"#,
    )
    .expect("test RON should parse")
}

/// Creates a `WallDefinition` with non-default values for testing
/// specific field propagation.
pub(super) fn custom_wall_definition() -> WallDefinition {
    WallDefinition {
        name: "CustomWall".to_string(),
        half_thickness: 45.0,
        color_rgb: Some([0.2, 2.0, 3.0]),
        effects: vec![RootEffect::On {
            target: Target::Wall,
            then: vec![EffectNode::When {
                trigger: Trigger::Bumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    }
}

/// Convenience alias for the default playfield config.
pub(super) fn default_playfield() -> PlayfieldConfig {
    PlayfieldConfig::default()
}
