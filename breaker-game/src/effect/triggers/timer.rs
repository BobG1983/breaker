use bevy::prelude::*;

use super::evaluate::RemoveChainsCommand;
use crate::effect::{commands::EffectCommandsExt, core::*};

fn tick_time_expires(
    time: Res<Time>,
    mut query: Query<(Entity, &mut StagedEffects)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut staged) in &mut query {
        let mut additions = Vec::new();
        staged.0.retain_mut(|(chip_name, node)| {
            if let EffectNode::When {
                trigger: Trigger::TimeExpires(remaining),
                then,
            } = node
            {
                *remaining -= dt;
                if *remaining <= 0.0 {
                    for child in then {
                        match child {
                            EffectNode::Do(effect) => {
                                commands.fire_effect(entity, effect.clone(), chip_name.clone());
                            }
                            EffectNode::Reverse { effects, chains } => {
                                for effect in effects {
                                    commands.reverse_effect(
                                        entity,
                                        effect.clone(),
                                        chip_name.clone(),
                                    );
                                }
                                if !chains.is_empty() {
                                    commands.queue(RemoveChainsCommand {
                                        entity,
                                        chains: chains.clone(),
                                    });
                                }
                            }
                            other => {
                                additions.push((chip_name.clone(), other.clone()));
                            }
                        }
                    }
                    return false; // consumed
                }
            }
            true // keep
        });
        staged.0.extend(additions);
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(FixedUpdate, tick_time_expires);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, tick_time_expires);
        app
    }

    /// Accumulates one fixed timestep then runs one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    /// Helper: build a `When(TimeExpires(secs), [child])` node.
    fn time_expires_node(secs: f32, child: EffectNode) -> EffectNode {
        EffectNode::When {
            trigger: Trigger::TimeExpires(secs),
            then: vec![child],
        }
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[test]
    fn time_expires_decrements_remaining() {
        // Entity with When(TimeExpires(2.0), ...) in StagedEffects.
        // After one tick, remaining should be less than 2.0. Entry retained.
        let mut app = test_app();

        let inner = EffectNode::When {
            trigger: Trigger::Death,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        };
        let node = time_expires_node(2.0, inner);
        let entity = app
            .world_mut()
            .spawn(StagedEffects(vec![("chip_a".into(), node)]))
            .id();

        tick(&mut app);

        let staged = app.world().get::<StagedEffects>(entity).unwrap();
        assert_eq!(
            staged.0.len(),
            1,
            "Entry should be retained (timer not expired)"
        );
        if let EffectNode::When {
            trigger: Trigger::TimeExpires(remaining),
            ..
        } = &staged.0[0].1
        {
            assert!(
                *remaining < 2.0,
                "Remaining should have been decremented from 2.0, got {remaining}"
            );
            assert!(
                *remaining > 0.0,
                "Timer should not have expired yet, got {remaining}"
            );
        } else {
            panic!("Expected When(TimeExpires(...)) node");
        }
    }

    #[test]
    fn time_expires_consumes_entry_at_zero() {
        // Entity with When(TimeExpires(0.001), [non-Do child]) in StagedEffects.
        // After one tick (dt ~0.015625s), remaining goes below 0 → entry consumed.
        // Use a non-Do child to avoid fire_effect command panics.
        let mut app = test_app();

        let inner = EffectNode::When {
            trigger: Trigger::Death,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        };
        let node = time_expires_node(0.001, inner);
        let entity = app
            .world_mut()
            .spawn(StagedEffects(vec![("chip_a".into(), node)]))
            .id();

        tick(&mut app);

        let staged = app.world().get::<StagedEffects>(entity).unwrap();
        // The TimeExpires entry is consumed; inner non-Do child pushed to additions.
        assert_eq!(
            staged.0.len(),
            1,
            "TimeExpires consumed, non-Do child added as addition (net 1)"
        );
        assert!(
            matches!(
                &staged.0[0].1,
                EffectNode::When {
                    trigger: Trigger::Death,
                    ..
                }
            ),
            "Remaining entry should be the inner When(Death) child"
        );
    }

    #[test]
    fn non_time_expires_entries_untouched() {
        // Entity with When(Bump, Do(X)) in StagedEffects. Timer ticks.
        // Non-TimeExpires entries should be completely untouched.
        let mut app = test_app();

        let node = EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        };
        let entity = app
            .world_mut()
            .spawn(StagedEffects(vec![("chip_a".into(), node.clone())]))
            .id();

        tick(&mut app);

        let staged = app.world().get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 1, "Non-TimeExpires entry must be retained");
        assert_eq!(staged.0[0].1, node, "Entry should be unchanged");
    }

    #[test]
    fn multiple_time_expires_tick_independently() {
        // Two TimeExpires entries: one at 1.0 (retained) and one at 0.001 (consumed).
        // Both use non-Do children to avoid command panics.
        let mut app = test_app();

        let inner_a = EffectNode::When {
            trigger: Trigger::Death,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        };
        let inner_b = EffectNode::When {
            trigger: Trigger::Death,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        };
        let node_long = time_expires_node(1.0, inner_a);
        let node_short = time_expires_node(0.001, inner_b);
        let entity = app
            .world_mut()
            .spawn(StagedEffects(vec![
                ("chip_a".into(), node_long),
                ("chip_b".into(), node_short),
            ]))
            .id();

        tick(&mut app);

        let staged = app.world().get::<StagedEffects>(entity).unwrap();
        // node_long retained (decremented), node_short consumed (addition pushed).
        // Net: 2 entries (1 retained TimeExpires + 1 addition from consumed).
        assert_eq!(
            staged.0.len(),
            2,
            "One retained + one addition from consumed timer"
        );

        // Find the retained TimeExpires entry
        let has_time_expires = staged.0.iter().any(|(_, node)| {
            matches!(node, EffectNode::When { trigger: Trigger::TimeExpires(r), .. } if *r < 1.0 && *r > 0.0)
        });
        assert!(
            has_time_expires,
            "Long timer should be retained with decremented value"
        );

        // Find the addition (inner When(Death))
        let has_addition = staged.0.iter().any(|(_, node)| {
            matches!(
                node,
                EffectNode::When {
                    trigger: Trigger::Death,
                    ..
                }
            )
        });
        assert!(
            has_addition,
            "Short timer consumed, its non-Do child should be added"
        );
    }

    #[test]
    fn time_expires_pushes_non_do_children_to_additions() {
        // When(TimeExpires(0.001), [When(Bump, Do(X))]) — on expiry,
        // the inner When(Bump) is a non-Do child, so it should be pushed
        // to StagedEffects as an addition.
        let mut app = test_app();

        let inner = EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
        };
        let node = time_expires_node(0.001, inner.clone());
        let entity = app
            .world_mut()
            .spawn(StagedEffects(vec![("chip_a".into(), node)]))
            .id();

        tick(&mut app);

        let staged = app.world().get::<StagedEffects>(entity).unwrap();
        assert_eq!(
            staged.0.len(),
            1,
            "TimeExpires consumed, non-Do child added"
        );
        assert_eq!(
            staged.0[0].1, inner,
            "Addition should be the inner When(Bump, Do(DamageBoost(3.0)))"
        );
    }

    // -- Section L: EffectSourceChip threading through tick_time_expires ───────────────────

    use crate::effect::{core::EffectSourceChip, effects::speed_boost::ActiveSpeedBoosts};

    #[test]
    fn tick_time_expires_threads_chip_name_as_source_chip_to_fire_effect() {
        // When(TimeExpires(0.001), [Do(Explode)]) with chip_name "timer_chip"
        // After timer expires, Explode fire() should be called with source_chip="timer_chip"
        // which results in EffectSourceChip(Some("timer_chip")) on the spawned ExplodeRequest.
        let mut app = test_app();

        let node = time_expires_node(
            0.001,
            EffectNode::Do(EffectKind::Explode {
                range: 60.0,
                damage_mult: 2.0,
            }),
        );

        app.world_mut().spawn((
            StagedEffects(vec![("timer_chip".into(), node)]),
            Transform::from_xyz(50.0, 50.0, 0.0),
        ));

        // First tick: timer expires, queues fire_effect command
        tick(&mut app);
        // Second tick: commands are applied (fire_effect → Explode::fire → spawn ExplodeRequest)
        tick(&mut app);

        let mut query = app.world_mut().query::<&EffectSourceChip>();
        let results: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(
            results.len(),
            1,
            "expected one entity with EffectSourceChip (on ExplodeRequest)"
        );
        assert_eq!(
            results[0].0,
            Some("timer_chip".to_string()),
            "tick_time_expires should thread chip_name 'timer_chip' to fire_effect"
        );
    }

    #[test]
    fn tick_time_expires_threads_chip_name_as_source_chip_to_reverse_effect() {
        // When(TimeExpires(0.001), [Reverse { effects: [SpeedBoost(1.3)], chains: [] }])
        // with chip_name "reversal_chip". SpeedBoost::reverse ignores source_chip
        // but this verifies the plumbing compiles and doesn't panic.
        let mut app = test_app();

        let node = EffectNode::When {
            trigger: Trigger::TimeExpires(0.001),
            then: vec![EffectNode::Reverse {
                effects: vec![EffectKind::SpeedBoost { multiplier: 1.3 }],
                chains: vec![],
            }],
        };

        let entity = app
            .world_mut()
            .spawn((
                StagedEffects(vec![("reversal_chip".into(), node)]),
                ActiveSpeedBoosts(vec![1.3]),
            ))
            .id();

        tick(&mut app);

        let boosts = app.world().get::<ActiveSpeedBoosts>(entity).unwrap();
        assert!(
            !boosts.0.contains(&1.3),
            "reverse_effect should have removed 1.3 from ActiveSpeedBoosts, got {:?}",
            boosts.0
        );
    }
}
