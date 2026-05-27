//! Alpha-flicker system for phantom-breaker entities.

use bevy::prelude::*;

use crate::shared::phantom::PhantomFlicker;

/// Modulates the alpha of a `MeshMaterial2d<ColorMaterial>` using a cosine
/// waveform driven by global elapsed time and the entity's `PhantomFlicker`
/// parameters.
///
/// Formula: `alpha(t) = min_alpha + (1 - min_alpha) * 0.5 * (1 + cos(2π * frequency * t))`
pub(crate) fn tick_phantom_flicker(
    time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(&PhantomFlicker, &MeshMaterial2d<ColorMaterial>)>,
) {
    let t = time.elapsed_secs();
    for (flicker, mesh_material) in query.iter() {
        let alpha = ((1.0 - flicker.min_alpha) * 0.5).mul_add(
            1.0 + (std::f32::consts::TAU * flicker.frequency * t).cos(),
            flicker.min_alpha,
        );
        if let Some(material) = materials.get_mut(&mesh_material.0) {
            material.color = material.color.with_alpha(alpha);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::{prelude::*, shared::phantom::PhantomFlicker};

    // ── helpers ───────────────────────────────────────────────────────────────

    fn system_only_app() -> App {
        TestAppBuilder::new()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                20,
            )))
            .with_system(Update, tick_phantom_flicker)
            .build()
    }

    fn add_material(app: &mut App, alpha: f32) -> Handle<ColorMaterial> {
        app.world_mut()
            .resource_mut::<Assets<ColorMaterial>>()
            .add(ColorMaterial {
                color: Color::srgba(1.0, 1.0, 1.0, alpha),
                ..Default::default()
            })
    }

    fn sample_alpha(app: &App, handle: &Handle<ColorMaterial>) -> f32 {
        app.world()
            .resource::<Assets<ColorMaterial>>()
            .get(handle)
            .expect("material asset present")
            .color
            .alpha()
    }

    // ── Behavior 1: alpha stays within [min_alpha, 1.0] and reaches both ends ──

    #[test]
    fn phantom_flicker_alpha_stays_within_min_and_max_across_full_period() {
        let mut app = system_only_app();
        let handle = add_material(&mut app, 1.0);
        app.world_mut().spawn((
            MeshMaterial2d(handle.clone()),
            PhantomFlicker {
                frequency: 4.0,
                min_alpha: 0.3,
            },
        ));

        // 1 prime update, 13 advance ticks (13 × 20 ms = 260 ms > 250 ms period)
        app.update(); // prime
        let mut samples = Vec::with_capacity(13);
        for _ in 0..13 {
            app.update();
            samples.push(sample_alpha(&app, &handle));
        }

        for &a in &samples {
            assert!(
                (0.3 - 1e-4..=1.0 + 1e-4).contains(&a),
                "alpha {a} out of bounds [0.3, 1.0]"
            );
        }
        assert!(
            samples.iter().any(|&a| a <= 0.31),
            "no sample near floor 0.3; samples: {samples:?}"
        );
        assert!(
            samples.iter().any(|&a| a >= 0.95),
            "no sample near ceiling 1.0; samples: {samples:?}"
        );
    }

    // ── Behavior 2: min_alpha = 1.0 collapses the flicker ─────────────────────

    #[test]
    fn phantom_flicker_with_min_alpha_one_keeps_alpha_pinned() {
        let mut app = system_only_app();
        // Sentinel initial alpha 0.5 — no-op stub leaves it at 0.5, failing the
        // assertion below. A correct implementation drives it to 1.0 every tick.
        let handle = add_material(&mut app, 0.5);
        app.world_mut().spawn((
            MeshMaterial2d(handle.clone()),
            PhantomFlicker {
                frequency: 4.0,
                min_alpha: 1.0,
            },
        ));

        app.update(); // prime
        let mut samples = Vec::with_capacity(13);
        for _ in 0..13 {
            app.update();
            samples.push(sample_alpha(&app, &handle));
        }

        for &a in &samples {
            assert!(
                (a - 1.0).abs() <= 1e-4,
                "alpha {a} deviated from 1.0 when min_alpha == 1.0"
            );
        }
    }

    // ── Behavior 3: min_alpha = 0.0 lets alpha reach 0.0 ─────────────────────

    #[test]
    fn phantom_flicker_with_min_alpha_zero_reaches_zero() {
        let mut app = system_only_app();
        let handle = add_material(&mut app, 1.0);
        app.world_mut().spawn((
            MeshMaterial2d(handle.clone()),
            PhantomFlicker {
                frequency: 4.0,
                min_alpha: 0.0,
            },
        ));

        app.update(); // prime
        let mut samples = Vec::with_capacity(13);
        for _ in 0..13 {
            app.update();
            samples.push(sample_alpha(&app, &handle));
        }

        for &a in &samples {
            assert!(
                (-1e-4..=1.0 + 1e-4).contains(&a),
                "alpha {a} out of bounds [-1e-4, 1.0+1e-4]"
            );
        }
        assert!(
            samples.iter().any(|&a| a <= 0.01),
            "no sample reached near 0.0; samples: {samples:?}"
        );
    }

    // ── Behavior 4: higher frequency completes more cycles ────────────────────

    #[test]
    fn phantom_flicker_higher_frequency_completes_more_cycles() {
        let mut app = system_only_app();
        let handle_a = add_material(&mut app, 1.0);
        let handle_b = add_material(&mut app, 1.0);

        app.world_mut().spawn((
            MeshMaterial2d(handle_a.clone()),
            PhantomFlicker {
                frequency: 4.0,
                min_alpha: 0.3,
            },
        ));
        app.world_mut().spawn((
            MeshMaterial2d(handle_b.clone()),
            PhantomFlicker {
                frequency: 8.0,
                min_alpha: 0.3,
            },
        ));

        app.update(); // prime
        let mut samples_a = Vec::with_capacity(13);
        let mut samples_b = Vec::with_capacity(13);
        for _ in 0..13 {
            app.update();
            samples_a.push(sample_alpha(&app, &handle_a));
            samples_b.push(sample_alpha(&app, &handle_b));
        }

        let midpoint = 0.65_f32; // (0.3 + 1.0) / 2.0
        let count_transitions = |samples: &[f32]| -> usize {
            samples
                .windows(2)
                .filter(|w| (w[0] >= midpoint) != (w[1] >= midpoint))
                .count()
        };

        let transitions_a = count_transitions(&samples_a);
        let transitions_b = count_transitions(&samples_b);
        assert!(
            transitions_b > transitions_a,
            "expected entity B (8 Hz) to have more midpoint crossings than A (4 Hz); \
             B={transitions_b}, A={transitions_a}"
        );
    }

    // ── Behavior 5: two entities with identical params produce identical alpha ─

    #[test]
    fn phantom_flicker_two_entities_same_params_match_each_tick() {
        let mut app = system_only_app();
        // Sentinel initial alpha 0.5 — no-op stub leaves all samples at 0.5,
        // failing the "system ran" assertion below.
        let initial_alpha = 0.5_f32;
        let handle_a = add_material(&mut app, initial_alpha);
        let handle_b = add_material(&mut app, initial_alpha);

        app.world_mut().spawn((
            MeshMaterial2d(handle_a.clone()),
            PhantomFlicker {
                frequency: 4.0,
                min_alpha: 0.3,
            },
        ));
        app.world_mut().spawn((
            MeshMaterial2d(handle_b.clone()),
            PhantomFlicker {
                frequency: 4.0,
                min_alpha: 0.3,
            },
        ));

        app.update(); // prime
        let mut samples_a = Vec::with_capacity(6);
        for tick in 0..6 {
            app.update();
            let alpha_a = sample_alpha(&app, &handle_a);
            let alpha_b = sample_alpha(&app, &handle_b);
            samples_a.push(alpha_a);
            assert!(
                (alpha_a - alpha_b).abs() <= 1e-4,
                "tick {tick}: alpha_A={alpha_a} != alpha_B={alpha_b} (diff > 1e-4)"
            );
        }
        // Guard: the system must have actually changed alpha away from the sentinel.
        // Without this, a no-op stub produces (0.5, 0.5) every tick — trivially equal.
        assert!(
            samples_a.iter().any(|&a| (a - initial_alpha).abs() > 0.01),
            "system did not run — all samples equal initial alpha {initial_alpha}; samples: {samples_a:?}"
        );
    }

    // ── T25: bolt-side regression guard — PhantomFlicker modulates bolt alpha ──

    #[test]
    fn phantom_flicker_modulates_alpha_on_phantom_bolt_entity() {
        use crate::bolt::components::{Bolt, PhantomBolt, PhantomDamagedCells, PhantomDedupKey};

        let mut app = system_only_app();
        let handle = add_material(&mut app, 1.0);
        app.world_mut().spawn((
            Bolt,
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells::default(),
            MeshMaterial2d(handle.clone()),
            PhantomFlicker {
                frequency: 4.0,
                min_alpha: 0.3,
            },
        ));

        // 1 prime update, 13 advance ticks (13 × 20 ms = 260 ms > 250 ms period at 4 Hz)
        app.update(); // prime
        let mut samples = Vec::with_capacity(13);
        for _ in 0..13 {
            app.update();
            samples.push(sample_alpha(&app, &handle));
        }

        for &a in &samples {
            assert!(
                (0.3 - 1e-4..=1.0 + 1e-4).contains(&a),
                "bolt alpha {a} out of bounds [0.3, 1.0]"
            );
        }
        assert!(
            samples.iter().any(|&a| a <= 0.31),
            "no bolt-side sample near floor 0.3; samples: {samples:?}"
        );
        assert!(
            samples.iter().any(|&a| a >= 0.95),
            "no bolt-side sample near ceiling 1.0; samples: {samples:?}"
        );
    }

    // ── T25 (edge case) — two phantom bolt entities with same params stay in phase ──

    #[test]
    fn phantom_flicker_two_phantom_bolt_entities_same_params_match_each_tick() {
        use crate::bolt::components::{Bolt, PhantomBolt, PhantomDamagedCells, PhantomDedupKey};

        let mut app = system_only_app();
        let initial_alpha = 0.5_f32;
        let handle_a = add_material(&mut app, initial_alpha);
        let handle_b = add_material(&mut app, initial_alpha);

        app.world_mut().spawn((
            Bolt,
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells::default(),
            MeshMaterial2d(handle_a.clone()),
            PhantomFlicker {
                frequency: 4.0,
                min_alpha: 0.3,
            },
        ));
        app.world_mut().spawn((
            Bolt,
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells::default(),
            MeshMaterial2d(handle_b.clone()),
            PhantomFlicker {
                frequency: 4.0,
                min_alpha: 0.3,
            },
        ));

        app.update(); // prime
        let mut samples_a = Vec::with_capacity(6);
        for tick in 0..6 {
            app.update();
            let alpha_a = sample_alpha(&app, &handle_a);
            let alpha_b = sample_alpha(&app, &handle_b);
            samples_a.push(alpha_a);
            assert!(
                (alpha_a - alpha_b).abs() <= 1e-4,
                "tick {tick}: phantom bolt alpha_A={alpha_a} != alpha_B={alpha_b} (diff > 1e-4)"
            );
        }
        // Guard: system must have changed alpha away from the sentinel.
        assert!(
            samples_a.iter().any(|&a| (a - initial_alpha).abs() > 0.01),
            "system did not run — all bolt samples equal initial alpha {initial_alpha}; samples: {samples_a:?}"
        );
    }

    // ── Behavior 6: FxPlugin registers the system, runs in NodeState::Playing ─

    #[test]
    fn fx_plugin_registers_tick_phantom_flicker_running_in_node_playing() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                20,
            )))
            .build();

        app.add_plugins(crate::fx::FxPlugin);

        let handle = add_material(&mut app, 1.0);
        app.world_mut().spawn((
            MeshMaterial2d(handle.clone()),
            PhantomFlicker {
                frequency: 4.0,
                min_alpha: 0.3,
            },
        ));

        // 1 prime + 6 advance ticks
        app.update(); // prime
        let mut samples = Vec::with_capacity(6);
        for _ in 0..6 {
            app.update();
            samples.push(sample_alpha(&app, &handle));
        }

        assert!(
            samples.iter().any(|&a| a <= 0.7),
            "no sample dropped below 0.7 — system may not be running; samples: {samples:?}"
        );
    }
}
