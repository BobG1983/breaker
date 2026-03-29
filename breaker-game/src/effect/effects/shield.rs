use bevy::prelude::*;

/// Marks an active shield on the owning entity.
///
/// Charges decrement on each bolt saved. When charges reach zero the component
/// is removed.
#[derive(Component)]
pub struct ShieldActive {
    /// Number of bolt-saves remaining.
    pub charges: u32,
}

/// Inserts or adds charges to `ShieldActive`.
///
/// If the entity already has a shield, adds `stacks` to existing charges.
/// If the entity has no shield and `stacks > 0`, inserts a new shield.
/// If the entity has no shield and `stacks == 0`, does nothing (no-op).
pub(crate) fn fire(entity: Entity, stacks: u32, world: &mut World) {
    if let Some(mut shield) = world.get_mut::<ShieldActive>(entity) {
        shield.charges += stacks;
    } else if stacks > 0 {
        world
            .entity_mut(entity)
            .insert(ShieldActive { charges: stacks });
    }
}

pub(crate) fn reverse(entity: Entity, world: &mut World) {
    world.entity_mut(entity).remove::<ShieldActive>();
}

pub(crate) fn register(_app: &mut App) {
    // No runtime systems — charge decrement happens in bolt_lost.
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 1: fire() inserts ShieldActive with charges equal to stacks ──

    #[test]
    fn fire_inserts_shield_active_with_charges_equal_to_stacks() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, 3, &mut world);

        let shield = world.get::<ShieldActive>(entity).unwrap();
        assert_eq!(
            shield.charges, 3,
            "fire(entity, 3) should insert ShieldActive {{ charges: 3 }}, got {}",
            shield.charges
        );
    }

    #[test]
    fn fire_inserts_shield_active_with_charges_1_minimum_meaningful() {
        // Edge case: stacks = 1 results in ShieldActive { charges: 1 }
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, 1, &mut world);

        let shield = world.get::<ShieldActive>(entity).unwrap();
        assert_eq!(
            shield.charges, 1,
            "fire(entity, 1) should insert ShieldActive {{ charges: 1 }}, got {}",
            shield.charges
        );
    }

    // ── Behavior 2: fire() on entity with existing ShieldActive adds charges ──

    #[test]
    fn fire_on_existing_shield_adds_charges() {
        let mut world = World::new();
        let entity = world.spawn(ShieldActive { charges: 2 }).id();

        fire(entity, 3, &mut world);

        let shield = world.get::<ShieldActive>(entity).unwrap();
        assert_eq!(
            shield.charges, 5,
            "fire(entity, 3) on existing charges: 2 should result in 5, got {}",
            shield.charges
        );
    }

    #[test]
    fn fire_stacks_0_on_existing_shield_leaves_charges_unchanged() {
        // Edge case: stacks = 0 adds 0 charges
        let mut world = World::new();
        let entity = world.spawn(ShieldActive { charges: 2 }).id();

        fire(entity, 0, &mut world);

        let shield = world.get::<ShieldActive>(entity).unwrap();
        assert_eq!(
            shield.charges, 2,
            "fire(entity, 0) on existing charges: 2 should remain 2, got {}",
            shield.charges
        );
    }

    // ── Behavior 3: fire() with stacks=0 on entity WITHOUT ShieldActive is no-op ──

    #[test]
    fn fire_stacks_0_without_shield_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, 0, &mut world);

        assert!(
            world.get::<ShieldActive>(entity).is_none(),
            "fire(entity, 0) without existing ShieldActive should not insert component"
        );
    }

    // ── Behavior 4: reverse() removes ShieldActive ──

    #[test]
    fn reverse_removes_shield_active() {
        let mut world = World::new();
        let entity = world.spawn(ShieldActive { charges: 5 }).id();

        reverse(entity, &mut world);

        assert!(
            world.get::<ShieldActive>(entity).is_none(),
            "reverse should remove ShieldActive component"
        );
    }

    #[test]
    fn reverse_without_shield_does_not_panic() {
        // Edge case: reverse on entity without ShieldActive should not panic
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        reverse(entity, &mut world); // should not panic
    }

    // ── Behavior 5: fire() uses new signature (compile-time constraint) ──
    // This is implicitly verified by all tests above calling fire(entity, stacks, world).

    // ── Behavior 6: tick_shield no longer exists — charges do not decay over time ──

    #[test]
    fn charges_do_not_decay_over_time() {
        // Given: Entity with ShieldActive { charges: 3 }
        // When: 10 seconds of game time elapse
        // Then: charges remain 3, component still present
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Register the shield module (which should be a no-op now)
        register(&mut app);

        let entity = app.world_mut().spawn(ShieldActive { charges: 3 }).id();

        // Simulate 10 seconds worth of ticks (at default 64Hz fixed timestep)
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        let ticks_for_10_seconds = (10.0_f64 / timestep.as_secs_f64()).ceil().max(0.0) as u32;

        for _ in 0..ticks_for_10_seconds {
            app.world_mut()
                .resource_mut::<Time<Fixed>>()
                .accumulate_overstep(timestep);
            app.update();
        }

        let shield = app.world().get::<ShieldActive>(entity).unwrap();
        assert_eq!(
            shield.charges, 3,
            "charges should not decay over time, expected 3, got {}",
            shield.charges
        );
    }

    #[test]
    fn charges_1_does_not_decay_after_60_seconds() {
        // Edge case: charges: 1 after 60 seconds — still charges: 1
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        register(&mut app);

        let entity = app.world_mut().spawn(ShieldActive { charges: 1 }).id();

        // 60 seconds at 64Hz = ~3840 ticks. Run a subset to keep test fast.
        // Even a few hundred ticks proves no decay system is running.
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        for _ in 0..200 {
            app.world_mut()
                .resource_mut::<Time<Fixed>>()
                .accumulate_overstep(timestep);
            app.update();
        }

        let shield = app.world().get::<ShieldActive>(entity).unwrap();
        assert_eq!(
            shield.charges, 1,
            "charges: 1 should not decay after time, expected 1, got {}",
            shield.charges
        );
    }

    // ── Behavior 7: ShieldActive no longer has remaining or owner fields ──
    // This is a compile-time constraint verified by all tests constructing
    // ShieldActive { charges: N } directly. If old fields existed without
    // defaults, these constructions would fail to compile.
}
