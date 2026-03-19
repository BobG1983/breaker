//! Per-effect observer handlers for chip effects.
//!
//! Each handler follows the observer pattern: triggered by [`ChipEffectApplied`],
//! self-selects via pattern matching, and modifies entity components directly.

mod bolt_size_boost;
mod bolt_speed_boost;
mod breaker_speed_boost;
mod bump_force_boost;
mod chain_hit;
mod damage_boost;
mod piercing;
mod tilt_control_boost;
mod width_boost;

use bevy::prelude::*;
pub(crate) use bolt_size_boost::handle_bolt_size_boost;
pub(crate) use bolt_speed_boost::handle_bolt_speed_boost;
pub(crate) use breaker_speed_boost::handle_breaker_speed_boost;
pub(crate) use bump_force_boost::handle_bump_force_boost;
pub(crate) use chain_hit::handle_chain_hit;
pub(crate) use damage_boost::handle_damage_boost;
pub(crate) use piercing::handle_piercing;
pub(crate) use tilt_control_boost::handle_tilt_control_boost;
pub(crate) use width_boost::handle_width_boost;

/// Stacks a `u32` component field on an entity.
///
/// - If `per_stack` is 0, this is a no-op regardless of `field`.
/// - If `field` is `Some`, adds `per_stack` when below the cap.
/// - If `field` is `None`, inserts the component with `per_stack` as the initial value.
pub(super) fn stack_u32<C, F>(
    entity: Entity,
    field: Option<&mut u32>,
    per_stack: u32,
    max_stacks: u32,
    commands: &mut Commands,
    constructor: F,
) where
    C: Component,
    F: FnOnce(u32) -> C,
{
    if per_stack == 0 {
        return;
    }
    if let Some(current) = field {
        if *current / per_stack < max_stacks {
            *current += per_stack;
        }
    } else {
        commands.entity(entity).insert(constructor(per_stack));
    }
}

/// Stacks an `f32` component field on an entity.
///
/// - If `per_stack` is 0.0, this is a no-op regardless of `field`.
/// - If `field` is `Some`, adds `per_stack` when below the cap.
/// - If `field` is `None`, inserts the component with `per_stack` as the initial value.
pub(super) fn stack_f32<C, F>(
    entity: Entity,
    field: Option<&mut f32>,
    per_stack: f32,
    max_stacks: u32,
    commands: &mut Commands,
    constructor: F,
) where
    C: Component,
    F: FnOnce(f32) -> C,
{
    if per_stack == 0.0 {
        return;
    }
    if let Some(current) = field {
        // Compare via f64 to avoid u32→f32 precision loss lint.
        if f64::from(*current / per_stack) < f64::from(max_stacks) {
            *current += per_stack;
        }
    } else {
        commands.entity(entity).insert(constructor(per_stack));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component)]
    struct TestU32(u32);

    #[derive(Component)]
    struct TestF32(f32);

    #[test]
    fn stack_u32_inserts_when_none() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            stack_u32::<TestU32, _>(entity, None, 2, 3, &mut commands, TestU32);
        });
        app.world_mut().flush();

        let val = app.world().entity(entity).get::<TestU32>().unwrap();
        assert_eq!(val.0, 2);
    }

    #[test]
    fn stack_u32_adds_when_under_cap() {
        let mut current = 2u32;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn_empty().id();
        let mut commands = app.world_mut().commands();
        // 2 / 2 = 1 stack, which is < 3 max — should add
        stack_u32::<TestU32, _>(entity, Some(&mut current), 2, 3, &mut commands, TestU32);
        assert_eq!(current, 4);
    }

    #[test]
    fn stack_u32_caps_at_max_stacks() {
        let mut current = 6u32;
        let per_stack = 2u32;
        let max_stacks = 3u32;
        // 6 / 2 = 3, which is NOT < 3, so no stacking
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn_empty().id();
        let mut commands = app.world_mut().commands();
        stack_u32::<TestU32, _>(
            entity,
            Some(&mut current),
            per_stack,
            max_stacks,
            &mut commands,
            TestU32,
        );
        assert_eq!(current, 6, "should not stack beyond max");
    }

    #[test]
    fn stack_u32_zero_per_stack_is_noop() {
        let mut current = 2u32;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn_empty().id();
        let mut commands = app.world_mut().commands();
        stack_u32::<TestU32, _>(entity, Some(&mut current), 0, 3, &mut commands, TestU32);
        assert_eq!(current, 2);
    }

    #[test]
    fn stack_f32_inserts_when_none() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            stack_f32::<TestF32, _>(entity, None, 1.5, 3, &mut commands, TestF32);
        });
        app.world_mut().flush();

        let val = app.world().entity(entity).get::<TestF32>().unwrap();
        assert!((val.0 - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn stack_f32_adds_when_under_cap() {
        let mut current = 1.5f32;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn_empty().id();
        let mut commands = app.world_mut().commands();
        // 1.5 / 1.5 = 1.0 stack, which is < 3 max — should add
        stack_f32::<TestF32, _>(entity, Some(&mut current), 1.5, 3, &mut commands, TestF32);
        assert!((current - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stack_f32_caps_at_max_stacks() {
        let mut current = 4.5f32;
        let per_stack = 1.5f32;
        let max_stacks = 3u32;
        // 4.5 / 1.5 = 3.0, which is NOT < 3
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn_empty().id();
        let mut commands = app.world_mut().commands();
        stack_f32::<TestF32, _>(
            entity,
            Some(&mut current),
            per_stack,
            max_stacks,
            &mut commands,
            TestF32,
        );
        assert!(
            (current - 4.5).abs() < f32::EPSILON,
            "should not stack beyond max"
        );
    }
}
