//! `PulseConfig` — periodic shockwave emitter.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::components::PulseEmitter;
use crate::effect_v3::{
    components::EffectSourceChip,
    traits::{Fireable, Reversible},
};

/// Configuration for periodic pulse shockwave emission.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PulseConfig {
    /// Radius of each pulse shockwave.
    pub base_range:      OrderedFloat<f32>,
    /// Extra range per stack.
    pub range_per_level: OrderedFloat<f32>,
    /// Current stack count.
    pub stacks:          u32,
    /// Expansion speed of each pulse ring.
    pub speed:           OrderedFloat<f32>,
    /// Seconds between each pulse emission.
    pub interval:        OrderedFloat<f32>,
}

impl Fireable for PulseConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        if world.get_entity(entity).is_err() {
            return;
        }
        world.entity_mut(entity).insert(PulseEmitter {
            base_range:      self.base_range.0,
            range_per_level: self.range_per_level.0,
            stacks:          self.stacks,
            speed:           self.speed.0,
            interval:        self.interval.0,
            timer:           self.interval.0,
            source_chip:     EffectSourceChip::from_source(source),
        });
    }

    fn register(app: &mut App) {
        use super::systems::{
            apply_pulse_damage, despawn_finished_pulse_ring, tick_pulse, tick_pulse_ring,
        };
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            (
                tick_pulse,
                tick_pulse_ring,
                apply_pulse_damage,
                despawn_finished_pulse_ring,
            )
                .chain()
                .in_set(EffectV3Systems::Tick),
        );
    }
}

impl Reversible for PulseConfig {
    fn reverse(&self, entity: Entity, _source: &str, world: &mut World) {
        if world.get_entity(entity).is_ok() {
            world.entity_mut(entity).remove::<PulseEmitter>();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rantzsoft_spatial2d::components::Position2D;

    use super::*;
    use crate::{
        bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
        effect_v3::{
            components::EffectSourceChip,
            effects::{
                DamageBoostConfig,
                pulse::{
                    PulseRing,
                    components::{
                        PulseRingBaseDamage, PulseRingDamageMultiplier, PulseRingDamaged,
                        PulseRingMaxRadius, PulseRingRadius, PulseRingSpeed,
                    },
                    systems::tick_pulse,
                },
                shockwave::components::{
                    ShockwaveBaseDamage, ShockwaveDamageMultiplier, ShockwaveDamaged,
                    ShockwaveMaxRadius, ShockwaveRadius, ShockwaveSource, ShockwaveSpeed,
                },
            },
            stacking::EffectStack,
            traits::Fireable,
        },
        shared::test_utils::{TestAppBuilder, tick},
    };

    // ── Helpers ────────────────────────────────────────────────────────────

    fn emitter_test_app() -> App {
        TestAppBuilder::new()
            .with_system(FixedUpdate, tick_pulse)
            .build()
    }

    fn tick_with_dt(app: &mut App, dt: Duration) {
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .set_timestep(dt);
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(dt);
        app.update();
    }

    fn make_config() -> PulseConfig {
        PulseConfig {
            base_range:      OrderedFloat(64.0),
            range_per_level: OrderedFloat(16.0),
            stacks:          1,
            speed:           OrderedFloat(200.0),
            interval:        OrderedFloat(1.0),
        }
    }

    /// Forces the emitter on `entity` to fire on the next tick by zeroing its
    /// timer. Section E tests use this so a single tick predictably spawns one
    /// ring.
    fn force_fire_on_next_tick(app: &mut App, entity: Entity) {
        app.world_mut()
            .get_mut::<PulseEmitter>(entity)
            .expect("entity must carry a PulseEmitter")
            .timer = 0.0;
    }

    // ── E. tick_pulse — emitter fires rings with snapshotted values ────────

    // #19
    #[test]
    fn tick_pulse_snapshots_bolt_base_damage_into_pulse_ring_base_damage() {
        let mut app = emitter_test_app();

        let _emitter = app
            .world_mut()
            .spawn((
                BoltBaseDamage(25.0),
                Position2D(Vec2::new(100.0, 200.0)),
                PulseEmitter {
                    base_range:      64.0,
                    range_per_level: 16.0,
                    stacks:          1,
                    speed:           200.0,
                    interval:        1.0,
                    timer:           0.0,
                    source_chip:     EffectSourceChip(None),
                },
            ))
            .id();

        tick(&mut app);

        let ring_count = app
            .world_mut()
            .query::<&PulseRing>()
            .iter(app.world())
            .count();
        assert_eq!(ring_count, 1, "expected exactly one spawned pulse ring");

        let base_dmg: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingBaseDamage>()
            .iter(app.world())
            .map(|d| d.0)
            .collect();
        assert_eq!(
            base_dmg.len(),
            1,
            "expected one PulseRingBaseDamage component"
        );
        assert!(
            (base_dmg[0] - 25.0).abs() < f32::EPSILON,
            "PulseRingBaseDamage should snapshot BoltBaseDamage(25.0), got {}",
            base_dmg[0],
        );

        // Verify the rest of the spawn bundle.
        let radii: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingRadius>()
            .iter(app.world())
            .map(|r| r.0)
            .collect();
        assert_eq!(radii.len(), 1);
        assert!(radii[0].abs() < f32::EPSILON);

        let max_radii: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingMaxRadius>()
            .iter(app.world())
            .map(|r| r.0)
            .collect();
        assert_eq!(max_radii.len(), 1);
        assert!((max_radii[0] - 64.0).abs() < f32::EPSILON);

        let speeds: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingSpeed>()
            .iter(app.world())
            .map(|s| s.0)
            .collect();
        assert_eq!(speeds.len(), 1);
        assert!((speeds[0] - 200.0).abs() < f32::EPSILON);

        let damaged_lens: Vec<usize> = app
            .world_mut()
            .query::<&PulseRingDamaged>()
            .iter(app.world())
            .map(|d| d.0.len())
            .collect();
        assert_eq!(damaged_lens.len(), 1);
        assert_eq!(damaged_lens[0], 0);

        let mults: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingDamageMultiplier>()
            .iter(app.world())
            .map(|m| m.0)
            .collect();
        assert_eq!(mults.len(), 1);
        assert!((mults[0] - 1.0).abs() < f32::EPSILON);

        // Ring position must match emitter position at tick time.
        let positions: Vec<Vec2> = app
            .world_mut()
            .query_filtered::<&Position2D, With<PulseRing>>()
            .iter(app.world())
            .map(|p| p.0)
            .collect();
        assert_eq!(positions.len(), 1);
        assert!((positions[0].x - 100.0).abs() < f32::EPSILON);
        assert!((positions[0].y - 200.0).abs() < f32::EPSILON);
    }

    // #20
    #[test]
    fn tick_pulse_falls_back_to_default_bolt_base_damage_when_emitter_lacks_it() {
        let mut app = emitter_test_app();

        app.world_mut().spawn((
            Position2D(Vec2::new(100.0, 200.0)),
            PulseEmitter {
                base_range:      64.0,
                range_per_level: 16.0,
                stacks:          1,
                speed:           200.0,
                interval:        1.0,
                timer:           0.0,
                source_chip:     EffectSourceChip(None),
            },
        ));

        tick(&mut app);

        let base_dmg: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingBaseDamage>()
            .iter(app.world())
            .map(|d| d.0)
            .collect();
        assert_eq!(
            base_dmg.len(),
            1,
            "expected one PulseRingBaseDamage component"
        );
        assert!(
            (base_dmg[0] - DEFAULT_BOLT_BASE_DAMAGE).abs() < f32::EPSILON,
            "missing BoltBaseDamage should fall back to DEFAULT_BOLT_BASE_DAMAGE \
             ({DEFAULT_BOLT_BASE_DAMAGE}), got {}",
            base_dmg[0],
        );
    }

    // #21
    #[test]
    fn tick_pulse_snapshots_zero_bolt_base_damage_faithfully() {
        let mut app = emitter_test_app();

        app.world_mut().spawn((
            BoltBaseDamage(0.0),
            Position2D(Vec2::ZERO),
            PulseEmitter {
                base_range:      64.0,
                range_per_level: 16.0,
                stacks:          1,
                speed:           200.0,
                interval:        1.0,
                timer:           0.0,
                source_chip:     EffectSourceChip(None),
            },
        ));

        tick(&mut app);

        let base_dmg: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingBaseDamage>()
            .iter(app.world())
            .map(|d| d.0)
            .collect();
        assert_eq!(base_dmg.len(), 1);
        assert!(
            base_dmg[0].abs() < f32::EPSILON,
            "BoltBaseDamage(0.0) should snapshot to PulseRingBaseDamage(0.0), got {}",
            base_dmg[0],
        );
    }

    // #22
    #[test]
    fn tick_pulse_snapshots_single_entry_damage_boost_stack() {
        let mut app = emitter_test_app();

        let emitter = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::ZERO),
                PulseEmitter {
                    base_range:      64.0,
                    range_per_level: 16.0,
                    stacks:          1,
                    speed:           200.0,
                    interval:        1.0,
                    timer:           0.0,
                    source_chip:     EffectSourceChip(None),
                },
            ))
            .id();

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(emitter, "amp", app.world_mut());

        tick(&mut app);

        let mults: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingDamageMultiplier>()
            .iter(app.world())
            .map(|m| m.0)
            .collect();
        assert_eq!(mults.len(), 1, "expected one PulseRingDamageMultiplier");
        assert!(
            (mults[0] - 2.0).abs() < 1e-5,
            "single-entry stack aggregate should be 2.0, got {}",
            mults[0],
        );

        // Snapshot must not consume the stack on the emitter.
        let stack = app
            .world()
            .get::<EffectStack<DamageBoostConfig>>(emitter)
            .expect("emitter should still carry an EffectStack<DamageBoostConfig>");
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 2.0).abs() < 1e-5);
    }

    // #23
    #[test]
    fn tick_pulse_snapshots_two_entry_damage_boost_stack_as_product() {
        let mut app = emitter_test_app();

        let emitter = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::ZERO),
                PulseEmitter {
                    base_range:      64.0,
                    range_per_level: 16.0,
                    stacks:          1,
                    speed:           200.0,
                    interval:        1.0,
                    timer:           0.0,
                    source_chip:     EffectSourceChip(None),
                },
            ))
            .id();

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(emitter, "amp_a", app.world_mut());
        DamageBoostConfig {
            multiplier: OrderedFloat(3.0),
        }
        .fire(emitter, "amp_b", app.world_mut());

        tick(&mut app);

        let mults: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingDamageMultiplier>()
            .iter(app.world())
            .map(|m| m.0)
            .collect();
        assert_eq!(mults.len(), 1);
        // 2.0 * 3.0 == 6.0 — unambiguous two-entry product.
        assert!(
            (mults[0] - 6.0).abs() < 1e-5,
            "two-entry stack aggregate should be 2.0 * 3.0 == 6.0, got {}",
            mults[0],
        );

        let stack = app
            .world()
            .get::<EffectStack<DamageBoostConfig>>(emitter)
            .expect("emitter should still carry an EffectStack<DamageBoostConfig>");
        assert_eq!(stack.len(), 2);
        assert!((stack.aggregate() - 6.0).abs() < 1e-5);
    }

    // #24
    #[test]
    fn tick_pulse_defaults_damage_multiplier_to_one_when_emitter_has_no_stack() {
        let mut app = emitter_test_app();

        app.world_mut().spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            PulseEmitter {
                base_range:      64.0,
                range_per_level: 16.0,
                stacks:          1,
                speed:           200.0,
                interval:        1.0,
                timer:           0.0,
                source_chip:     EffectSourceChip(None),
            },
        ));

        tick(&mut app);

        let mults: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingDamageMultiplier>()
            .iter(app.world())
            .map(|m| m.0)
            .collect();
        assert_eq!(mults.len(), 1);
        assert!(
            (mults[0] - 1.0).abs() < 1e-5,
            "missing-stack fallback should yield multiplier 1.0, got {}",
            mults[0],
        );
    }

    // #25
    #[test]
    fn tick_pulse_propagates_some_source_chip_onto_spawned_ring() {
        let mut app = emitter_test_app();

        let emitter = app
            .world_mut()
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        make_config().fire(emitter, "storm_chip", app.world_mut());
        force_fire_on_next_tick(&mut app, emitter);

        tick(&mut app);

        let chips: Vec<Option<String>> = app
            .world_mut()
            .query_filtered::<&EffectSourceChip, With<PulseRing>>()
            .iter(app.world())
            .map(|c| c.0.clone())
            .collect();
        assert_eq!(chips.len(), 1, "expected exactly one PulseRing with a chip");
        assert_eq!(
            chips[0],
            Some("storm_chip".to_string()),
            "spawned ring should carry EffectSourceChip(Some(\"storm_chip\")), got {:?}",
            chips[0],
        );
    }

    // #26
    #[test]
    fn tick_pulse_propagates_none_source_chip_for_empty_fire_source() {
        let mut app = emitter_test_app();

        let emitter = app
            .world_mut()
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        make_config().fire(emitter, "", app.world_mut());
        force_fire_on_next_tick(&mut app, emitter);

        tick(&mut app);

        let chips: Vec<Option<String>> = app
            .world_mut()
            .query_filtered::<&EffectSourceChip, With<PulseRing>>()
            .iter(app.world())
            .map(|c| c.0.clone())
            .collect();
        assert_eq!(
            chips.len(),
            1,
            "expected exactly one PulseRing with a chip component"
        );
        assert_eq!(
            chips[0], None,
            "empty source string should map to EffectSourceChip(None), got {:?}",
            chips[0],
        );
    }

    // #27
    #[test]
    fn tick_pulse_resnapshots_bolt_base_damage_between_ticks_when_it_changes() {
        let mut app = emitter_test_app();

        let emitter = app
            .world_mut()
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        make_config().fire(emitter, "storm_chip", app.world_mut());
        force_fire_on_next_tick(&mut app, emitter);

        tick(&mut app);

        // Mutate BoltBaseDamage and force a second fire.
        app.world_mut()
            .get_mut::<BoltBaseDamage>(emitter)
            .expect("emitter must carry BoltBaseDamage")
            .0 = 50.0;
        force_fire_on_next_tick(&mut app, emitter);

        tick(&mut app);

        let ring_count = app
            .world_mut()
            .query::<&PulseRing>()
            .iter(app.world())
            .count();
        assert_eq!(ring_count, 2, "two ticks should produce two rings");

        let mut base_dmgs: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingBaseDamage>()
            .iter(app.world())
            .map(|d| d.0)
            .collect();
        base_dmgs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            base_dmgs.len(),
            2,
            "expected two PulseRingBaseDamage values"
        );
        assert!(
            (base_dmgs[0] - 10.0).abs() < f32::EPSILON,
            "first ring should snapshot the original 10.0, got {}",
            base_dmgs[0],
        );
        assert!(
            (base_dmgs[1] - 50.0).abs() < f32::EPSILON,
            "second ring should snapshot the mutated 50.0, got {}",
            base_dmgs[1],
        );

        // Both rings should still carry the same chip string (it lives on the emitter).
        let chips: Vec<Option<String>> = app
            .world_mut()
            .query_filtered::<&EffectSourceChip, With<PulseRing>>()
            .iter(app.world())
            .map(|c| c.0.clone())
            .collect();
        assert_eq!(chips.len(), 2);
        for chip in &chips {
            assert_eq!(
                chip,
                &Some("storm_chip".to_string()),
                "both rings should carry the storm_chip source",
            );
        }
    }

    // #28
    #[test]
    fn spawned_ring_carries_pulse_ring_marker_alongside_new_components() {
        let mut app = emitter_test_app();

        app.world_mut().spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            PulseEmitter {
                base_range:      64.0,
                range_per_level: 16.0,
                stacks:          1,
                speed:           200.0,
                interval:        1.0,
                timer:           0.0,
                source_chip:     EffectSourceChip(None),
            },
        ));

        tick(&mut app);

        // Find the spawned ring entity.
        let rings: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<PulseRing>>()
            .iter(app.world())
            .collect();
        assert_eq!(rings.len(), 1, "expected exactly one PulseRing entity");
        let ring = rings[0];

        assert!(
            app.world().get::<PulseRingRadius>(ring).is_some(),
            "ring should carry PulseRingRadius",
        );
        assert!(
            app.world().get::<PulseRingMaxRadius>(ring).is_some(),
            "ring should carry PulseRingMaxRadius",
        );
        assert!(
            app.world().get::<PulseRingSpeed>(ring).is_some(),
            "ring should carry PulseRingSpeed",
        );
        assert!(
            app.world().get::<PulseRingDamaged>(ring).is_some(),
            "ring should carry PulseRingDamaged",
        );
        assert!(
            app.world().get::<PulseRingBaseDamage>(ring).is_some(),
            "ring should carry PulseRingBaseDamage",
        );
        assert!(
            app.world().get::<PulseRingDamageMultiplier>(ring).is_some(),
            "ring should carry PulseRingDamageMultiplier",
        );
        assert!(
            app.world().get::<Position2D>(ring).is_some(),
            "ring should carry Position2D",
        );
    }

    // #29
    #[test]
    fn spawned_ring_does_not_carry_shockwave_runtime_components() {
        let mut app = emitter_test_app();

        app.world_mut().spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            PulseEmitter {
                base_range:      64.0,
                range_per_level: 16.0,
                stacks:          1,
                speed:           200.0,
                interval:        1.0,
                timer:           0.0,
                source_chip:     EffectSourceChip(None),
            },
        ));

        tick(&mut app);

        let rings: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<PulseRing>>()
            .iter(app.world())
            .collect();
        assert_eq!(rings.len(), 1, "expected exactly one PulseRing entity");
        let ring = rings[0];

        assert!(
            app.world().get::<ShockwaveSource>(ring).is_none(),
            "pulse ring must NOT carry ShockwaveSource",
        );
        assert!(
            app.world().get::<ShockwaveRadius>(ring).is_none(),
            "pulse ring must NOT carry ShockwaveRadius",
        );
        assert!(
            app.world().get::<ShockwaveMaxRadius>(ring).is_none(),
            "pulse ring must NOT carry ShockwaveMaxRadius",
        );
        assert!(
            app.world().get::<ShockwaveSpeed>(ring).is_none(),
            "pulse ring must NOT carry ShockwaveSpeed",
        );
        assert!(
            app.world().get::<ShockwaveDamaged>(ring).is_none(),
            "pulse ring must NOT carry ShockwaveDamaged",
        );
        assert!(
            app.world().get::<ShockwaveBaseDamage>(ring).is_none(),
            "pulse ring must NOT carry ShockwaveBaseDamage",
        );
        assert!(
            app.world().get::<ShockwaveDamageMultiplier>(ring).is_none(),
            "pulse ring must NOT carry ShockwaveDamageMultiplier",
        );
    }

    // #37 — `tick_pulse` config-side cross-check (single-gate `if`, not `while`)
    //
    // The systems-side test at the same number locks the same invariant from
    // the test_utils-built emitter path. The config-side variant additionally
    // routes the emitter through `PulseConfig::fire()` so a regression in
    // `fire()` (e.g., re-installing the emitter with a `while`-emulating timer
    // shape) is also caught.
    #[test]
    fn tick_pulse_via_fire_fires_exactly_one_ring_per_tick_when_dt_exceeds_interval() {
        let mut app = emitter_test_app();

        let emitter = app
            .world_mut()
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        let config = PulseConfig {
            base_range:      OrderedFloat(64.0),
            range_per_level: OrderedFloat(16.0),
            stacks:          1,
            speed:           OrderedFloat(200.0),
            interval:        OrderedFloat(0.25),
        };
        config.fire(emitter, "storm_chip", app.world_mut());

        // Force timer to 0.25 so dt = 1.0 (4*interval) tries to "burst".
        app.world_mut()
            .get_mut::<PulseEmitter>(emitter)
            .expect("emitter must carry PulseEmitter")
            .timer = 0.25;

        tick_with_dt(&mut app, Duration::from_secs(1));

        let ring_count = app
            .world_mut()
            .query::<&PulseRing>()
            .iter(app.world())
            .count();
        assert_eq!(
            ring_count, 1,
            "single-gate `if` must fire exactly one ring when dt >> interval, got {ring_count}",
        );

        let timer = app.world().get::<PulseEmitter>(emitter).unwrap().timer;
        assert!(
            (timer - (-0.5)).abs() < f32::EPSILON,
            "single-decrement, single-fire, single-reload must yield timer == -0.5, got {timer}",
        );
    }
}
