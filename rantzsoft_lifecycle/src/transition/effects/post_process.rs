//! Post-process transition effect component and types.
//!
//! `TransitionEffect` is inserted on the camera entity by start systems and
//! removed by end systems. The shader reads this component as a uniform to
//! drive fullscreen post-processing transitions via `FullscreenMaterial`.

use bevy::{
    asset::embedded_asset,
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        fullscreen_material::FullscreenMaterial,
    },
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_graph::{InternedRenderLabel, InternedRenderSubGraph, RenderLabel, RenderSubGraph},
        render_resource::ShaderType,
    },
    shader::ShaderRef,
};

/// Fullscreen transition effect component, inserted on the camera entity.
///
/// All fields are WGSL-aligned (`Vec4` first for 16-byte alignment).
/// The shader reads these fields each frame to render the transition.
#[derive(Component, ExtractComponent, Clone, Copy, Default, ShaderType)]
pub struct TransitionEffect {
    /// Effect color as linear RGBA Vec4.
    pub color: Vec4,
    /// Direction vector (used by wipe: normalized axis; slide: UV offset
    /// direction).
    pub direction: Vec4,
    /// Effect type discriminator: 0=none, 1=fade, 2=dissolve, 3=pixelate,
    /// 4=iris, 5=wipe, 6=slide.
    pub effect_type: u32,
    /// Animation progress, 0.0 to 1.0.
    pub progress: f32,
}

impl FullscreenMaterial for TransitionEffect {
    fn fragment_shader() -> ShaderRef {
        "embedded://rantzsoft_lifecycle/transition/effects/shaders/transition.wgsl".into()
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            Node2d::Tonemapping.intern(),
            TransitionLabel.intern(),
            Node2d::EndMainPassPostProcessing.intern(),
        ]
    }

    fn sub_graph() -> Option<InternedRenderSubGraph> {
        Some(Core2d.intern())
    }
}

/// Effect type constants used in `TransitionEffect::effect_type`.
pub struct EffectType;

impl EffectType {
    /// No effect (default / cleared state).
    pub const NONE: u32 = 0;
    /// Fade in/out effect.
    pub const FADE: u32 = 1;
    /// Dissolve in/out effect.
    pub const DISSOLVE: u32 = 2;
    /// Pixelate in/out effect.
    pub const PIXELATE: u32 = 3;
    /// Iris in/out effect.
    pub const IRIS: u32 = 4;
    /// Wipe in/out effect.
    pub const WIPE: u32 = 5;
    /// Slide effect.
    pub const SLIDE: u32 = 6;
}

/// Render label for the transition post-process pass.
#[derive(Debug, Hash, PartialEq, Eq, Clone, bevy::render::render_graph::RenderLabel)]
pub struct TransitionLabel;

/// Register the `FullscreenMaterialPlugin` and embed the shader asset.
///
/// Requires `AssetPlugin` to be present (skipped in headless/test environments
/// that use `MinimalPlugins` without asset support).
pub(crate) fn setup_post_process(app: &mut App) {
    use bevy::{
        asset::io::embedded::EmbeddedAssetRegistry,
        core_pipeline::fullscreen_material::FullscreenMaterialPlugin,
    };

    // Skip in headless/test environments without AssetPlugin
    if !app.world().contains_resource::<EmbeddedAssetRegistry>() {
        return;
    }

    app.add_plugins(FullscreenMaterialPlugin::<TransitionEffect>::default());
    embedded_asset!(app, "shaders/transition.wgsl");
}

// ---------------------------------------------------------------------------
// Direction encoding helpers
// ---------------------------------------------------------------------------

/// Convert a `WipeDirection` to a direction `Vec4` for the shader.
#[must_use]
pub const fn wipe_direction_to_vec4(
    direction: &crate::transition::effects::shared::WipeDirection,
) -> Vec4 {
    match direction {
        crate::transition::effects::shared::WipeDirection::Left => Vec4::new(-1.0, 0.0, 0.0, 0.0),
        crate::transition::effects::shared::WipeDirection::Right => Vec4::new(1.0, 0.0, 0.0, 0.0),
        crate::transition::effects::shared::WipeDirection::Up => Vec4::new(0.0, 1.0, 0.0, 0.0),
        crate::transition::effects::shared::WipeDirection::Down => Vec4::new(0.0, -1.0, 0.0, 0.0),
    }
}

/// Convert a `SlideDirection` to a direction `Vec4` for the shader (UV Y
/// inverted).
#[must_use]
pub const fn slide_direction_to_vec4(
    direction: &crate::transition::effects::slide::SlideDirection,
) -> Vec4 {
    match direction {
        crate::transition::effects::slide::SlideDirection::Left => Vec4::new(-1.0, 0.0, 0.0, 0.0),
        crate::transition::effects::slide::SlideDirection::Right => Vec4::new(1.0, 0.0, 0.0, 0.0),
        crate::transition::effects::slide::SlideDirection::Up => Vec4::new(0.0, -1.0, 0.0, 0.0),
        crate::transition::effects::slide::SlideDirection::Down => Vec4::new(0.0, 1.0, 0.0, 0.0),
    }
}

/// Convert a `Color` to linear RGBA `Vec4`.
pub(crate) fn color_to_linear_vec4(color: Color) -> Vec4 {
    let linear = color.to_linear();
    Vec4::new(linear.red, linear.green, linear.blue, linear.alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    // =======================================================================
    // Spec Behavior 1: TransitionEffect default values
    // =======================================================================

    #[test]
    fn transition_effect_default_has_effect_type_none_and_zero_progress() {
        let effect = TransitionEffect::default();
        assert_eq!(
            effect.color,
            Vec4::ZERO,
            "default color should be Vec4::ZERO"
        );
        assert_eq!(
            effect.direction,
            Vec4::ZERO,
            "default direction should be Vec4::ZERO"
        );
        assert_eq!(
            effect.effect_type,
            EffectType::NONE,
            "default effect_type should be EffectType::NONE (0)"
        );
        assert!(
            effect.progress.abs() < f32::EPSILON,
            "default progress should be 0.0"
        );
    }

    // =======================================================================
    // Spec Behavior 2: TransitionEffect can be inserted on an entity
    // =======================================================================

    #[test]
    fn transition_effect_can_be_inserted_and_queried_on_entity() {
        let mut world = World::new();
        let entity = world
            .spawn(TransitionEffect {
                color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                direction: Vec4::ZERO,
                effect_type: EffectType::FADE,
                progress: 0.5,
            })
            .id();

        let effect_ref = world.get::<TransitionEffect>(entity);
        assert!(effect_ref.is_some(), "entity should have TransitionEffect");
        let effect = effect_ref.copied().unwrap_or_default();
        assert_eq!(effect.color, Vec4::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(effect.direction, Vec4::ZERO);
        assert_eq!(effect.effect_type, EffectType::FADE);
        assert!((effect.progress - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn transition_effect_can_be_removed_without_despawning_entity() {
        let mut world = World::new();
        let entity = world
            .spawn(TransitionEffect {
                color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                direction: Vec4::ZERO,
                effect_type: EffectType::FADE,
                progress: 0.5,
            })
            .id();

        world.entity_mut(entity).remove::<TransitionEffect>();
        assert!(
            world.get::<TransitionEffect>(entity).is_none(),
            "TransitionEffect should be removed"
        );
        assert!(
            world.get_entity(entity).is_ok(),
            "entity should still exist after component removal"
        );
    }

    // =======================================================================
    // Spec Behavior 3: TransitionEffect is Copy and Clone
    // =======================================================================

    #[test]
    fn transition_effect_is_copy_and_clone() {
        let original = TransitionEffect {
            color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            direction: Vec4::ZERO,
            effect_type: EffectType::FADE,
            progress: 0.5,
        };
        let copied = original;
        // Verify Copy works (original still usable after copy)
        assert_eq!(copied.color, original.color);
        assert_eq!(copied.effect_type, original.effect_type);
    }

    // =======================================================================
    // Spec Behavior 4: EffectType constants
    // =======================================================================

    #[test]
    fn effect_type_constants_have_correct_values() {
        assert_eq!(EffectType::NONE, 0);
        assert_eq!(EffectType::FADE, 1);
        assert_eq!(EffectType::DISSOLVE, 2);
        assert_eq!(EffectType::PIXELATE, 3);
        assert_eq!(EffectType::IRIS, 4);
        assert_eq!(EffectType::WIPE, 5);
        assert_eq!(EffectType::SLIDE, 6);
    }

    #[test]
    fn effect_type_constants_are_all_distinct() {
        let values = [
            EffectType::NONE,
            EffectType::FADE,
            EffectType::DISSOLVE,
            EffectType::PIXELATE,
            EffectType::IRIS,
            EffectType::WIPE,
            EffectType::SLIDE,
        ];
        for i in 0..values.len() {
            for j in (i + 1)..values.len() {
                assert_ne!(
                    values[i], values[j],
                    "EffectType constants must be distinct: index {i} == index {j}"
                );
            }
        }
    }

    // =======================================================================
    // Spec Behavior 91: WipeDirection to Vec4 encoding
    // =======================================================================

    #[test]
    fn wipe_direction_to_vec4_left_is_negative_x() {
        use crate::transition::effects::shared::WipeDirection;
        let v = wipe_direction_to_vec4(&WipeDirection::Left);
        assert_eq!(
            v,
            Vec4::new(-1.0, 0.0, 0.0, 0.0),
            "Left should be (-1, 0, 0, 0)"
        );
    }

    #[test]
    fn wipe_direction_to_vec4_right_is_positive_x() {
        use crate::transition::effects::shared::WipeDirection;
        let v = wipe_direction_to_vec4(&WipeDirection::Right);
        assert_eq!(
            v,
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            "Right should be (1, 0, 0, 0)"
        );
    }

    #[test]
    fn wipe_direction_to_vec4_up_is_positive_y() {
        use crate::transition::effects::shared::WipeDirection;
        let v = wipe_direction_to_vec4(&WipeDirection::Up);
        assert_eq!(
            v,
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            "Up should be (0, 1, 0, 0)"
        );
    }

    #[test]
    fn wipe_direction_to_vec4_down_is_negative_y() {
        use crate::transition::effects::shared::WipeDirection;
        let v = wipe_direction_to_vec4(&WipeDirection::Down);
        assert_eq!(
            v,
            Vec4::new(0.0, -1.0, 0.0, 0.0),
            "Down should be (0, -1, 0, 0)"
        );
    }

    // =======================================================================
    // Spec Behavior 92: SlideDirection to Vec4 encoding (UV Y inverted)
    // =======================================================================

    #[test]
    fn slide_direction_to_vec4_left_is_negative_x() {
        use crate::transition::effects::slide::SlideDirection;
        let v = slide_direction_to_vec4(&SlideDirection::Left);
        assert_eq!(
            v,
            Vec4::new(-1.0, 0.0, 0.0, 0.0),
            "Left should be (-1, 0, 0, 0)"
        );
    }

    #[test]
    fn slide_direction_to_vec4_right_is_positive_x() {
        use crate::transition::effects::slide::SlideDirection;
        let v = slide_direction_to_vec4(&SlideDirection::Right);
        assert_eq!(
            v,
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            "Right should be (1, 0, 0, 0)"
        );
    }

    #[test]
    fn slide_direction_to_vec4_up_is_negative_y_uv_inverted() {
        use crate::transition::effects::slide::SlideDirection;
        let v = slide_direction_to_vec4(&SlideDirection::Up);
        assert_eq!(
            v,
            Vec4::new(0.0, -1.0, 0.0, 0.0),
            "Up should be (0, -1, 0, 0) in UV space"
        );
    }

    #[test]
    fn slide_direction_to_vec4_down_is_positive_y_uv_inverted() {
        use crate::transition::effects::slide::SlideDirection;
        let v = slide_direction_to_vec4(&SlideDirection::Down);
        assert_eq!(
            v,
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            "Down should be (0, 1, 0, 0) in UV space"
        );
    }
}
