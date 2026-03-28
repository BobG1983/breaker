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
                                commands.fire_effect(entity, effect.clone());
                            }
                            EffectNode::Reverse { effects, chains } => {
                                for effect in effects {
                                    commands.reverse_effect(entity, effect.clone());
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
        let node = time_expires_node(0.001, inner.clone());
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
}
