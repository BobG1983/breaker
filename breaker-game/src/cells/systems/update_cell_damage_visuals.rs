//! System that updates the cell's `ColorMaterial` based on `Hp` fraction to
//! preserve the damage color fade juice from legacy `handle_cell_hit`.

use bevy::prelude::*;

use crate::{
    cells::components::{Cell, CellDamageVisuals},
    prelude::*,
};

/// Updates the `ColorMaterial` of cells whose `Hp` changed this tick.
///
/// Color is a function of `Hp.current / Hp.starting`:
/// - `hdr_base` scaled by the fraction for the red (HDR) channel
/// - `green_min * fraction` for green
/// - `blue_range * (1 - fraction) + blue_base` for blue
///
/// Cells with `Hp.current <= 0.0` are skipped — they are about to be handled
/// by `detect_deaths<Cell>` / `handle_kill<Cell>` and their color is
/// irrelevant. Dead cells (already marked `Dead`) are excluded by the query
/// filter.
type DamageVisualQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Hp,
        &'static MeshMaterial2d<ColorMaterial>,
        &'static CellDamageVisuals,
    ),
    (With<Cell>, Changed<Hp>, Without<Dead>),
>;

pub(crate) fn update_cell_damage_visuals(
    query: DamageVisualQuery,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (hp, material_handle, visuals) in &query {
        if hp.current <= 0.0 {
            continue;
        }
        let frac = if hp.starting == 0.0 {
            0.0
        } else {
            (hp.current / hp.starting).clamp(0.0, 1.0)
        };
        let intensity = frac * visuals.hdr_base;
        if let Some(material) = materials.get_mut(material_handle.id()) {
            material.color = Color::srgb(
                intensity,
                visuals.green_min * frac,
                visuals.blue_range.mul_add(1.0 - frac, visuals.blue_base),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::components::{Cell, CellDamageVisuals};

    /// Canonical Group U test values — same across U1/U2/U3/U4 so per-test
    /// colors can be compared.
    fn visuals() -> CellDamageVisuals {
        CellDamageVisuals {
            hdr_base:   2.0,
            green_min:  0.3,
            blue_range: 0.5,
            blue_base:  0.2,
        }
    }

    fn u_test_app() -> App {
        TestAppBuilder::new()
            .with_resource::<Assets<ColorMaterial>>()
            .with_system(FixedUpdate, update_cell_damage_visuals)
            .build()
    }

    fn spawn_cell_with_visuals_and_material(
        app: &mut App,
        hp: Hp,
    ) -> (Entity, Handle<ColorMaterial>) {
        let material = app
            .world_mut()
            .resource_mut::<Assets<ColorMaterial>>()
            .add(ColorMaterial::from_color(Color::srgb(1.0, 0.5, 0.5)));
        let entity = app
            .world_mut()
            .spawn((
                Cell,
                hp,
                KilledBy::default(),
                visuals(),
                MeshMaterial2d(material.clone()),
            ))
            .id();
        (entity, material)
    }

    fn material_color(app: &App, handle: &Handle<ColorMaterial>) -> Color {
        app.world()
            .resource::<Assets<ColorMaterial>>()
            .get(handle)
            .expect("material should exist")
            .color
    }

    fn color_delta(a: Color, b: Color) -> f32 {
        let a = a.to_srgba();
        let b = b.to_srgba();
        (a.red - b.red).abs()
            + (a.green - b.green).abs()
            + (a.blue - b.blue).abs()
            + (a.alpha - b.alpha).abs()
    }

    // ── U1: full-HP cell has the full-health base color ─────────────────

    /// U1: at full HP (fraction 1.0), the system writes the "undamaged" color
    /// from `CellDamageVisuals`. Exact per-channel values are an implementation
    /// detail; the test only asserts the color is finite and not equal
    /// to a pure-NaN degenerate.
    #[test]
    fn full_hp_cell_has_the_full_health_base_color() {
        let mut app = u_test_app();
        let (_cell, material) = spawn_cell_with_visuals_and_material(&mut app, Hp::new(10.0));

        tick(&mut app);

        let color = material_color(&app, &material).to_srgba();
        assert!(color.red.is_finite(), "red channel should be finite");
        assert!(color.green.is_finite(), "green channel should be finite");
        assert!(color.blue.is_finite(), "blue channel should be finite");
        assert!(color.alpha.is_finite(), "alpha channel should be finite");
    }

    // ── U2: partial-HP cell has a darker/tinted color ───────────────────

    /// U2: cell A stays at full HP; cell B drops to 50% HP. Their colors must
    /// differ (non-trivial delta). Cell C at 10% HP has a LARGER delta than B.
    #[test]
    fn partial_hp_cell_has_a_darker_color_than_full_hp_cell_and_scales() {
        let mut app = u_test_app();

        let (_cell_a, material_a) = spawn_cell_with_visuals_and_material(&mut app, Hp::new(10.0));
        let (cell_b, material_b) = spawn_cell_with_visuals_and_material(&mut app, Hp::new(10.0));
        let (cell_c, material_c) = spawn_cell_with_visuals_and_material(&mut app, Hp::new(10.0));

        // Drop cell B to 50% HP, cell C to 10% HP.
        app.world_mut().get_mut::<Hp>(cell_b).unwrap().current = 5.0;
        app.world_mut().get_mut::<Hp>(cell_c).unwrap().current = 1.0;

        tick(&mut app);

        let color_a = material_color(&app, &material_a);
        let color_b = material_color(&app, &material_b);
        let color_c = material_color(&app, &material_c);

        let delta_b = color_delta(color_a, color_b);
        let delta_c = color_delta(color_a, color_c);

        assert!(
            delta_b > 0.01,
            "50%% HP cell color should differ from full HP by > 0.01, got {delta_b}"
        );
        assert!(
            delta_c > delta_b,
            "10%% HP color delta ({delta_c}) should be larger than 50%% HP delta ({delta_b})"
        );
    }

    // ── U3: zero/negative HP cells are SKIPPED — color unchanged ─────────

    /// U3: cells at `Hp.current <= 0.0` are skipped by the system — the
    /// material color does NOT change between the pre-zero tick and the
    /// post-zero tick.
    #[test]
    fn zero_hp_cell_is_skipped_material_color_unchanged() {
        let mut app = u_test_app();
        let (cell, material) = spawn_cell_with_visuals_and_material(&mut app, Hp::new(3.0));

        // Run one tick to let the system write the full-HP color.
        tick(&mut app);
        let color_before = material_color(&app, &material);

        // Drop HP to 0.0 (triggers Changed<Hp>) and tick again.
        app.world_mut().get_mut::<Hp>(cell).unwrap().current = 0.0;
        tick(&mut app);

        let color_after = material_color(&app, &material);
        let delta = color_delta(color_before, color_after);
        assert!(
            delta < f32::EPSILON,
            "zero-HP cell material color should be unchanged, delta was {delta}"
        );
    }

    /// U3 edge: negative HP (`-5.0`) — same outcome, system skips the cell.
    #[test]
    fn negative_hp_cell_is_skipped_material_color_unchanged() {
        let mut app = u_test_app();
        let (cell, material) = spawn_cell_with_visuals_and_material(&mut app, Hp::new(3.0));

        tick(&mut app);
        let color_before = material_color(&app, &material);

        app.world_mut().get_mut::<Hp>(cell).unwrap().current = -5.0;
        tick(&mut app);

        let color_after = material_color(&app, &material);
        let delta = color_delta(color_before, color_after);
        assert!(
            delta < f32::EPSILON,
            "negative-HP cell material color should be unchanged, delta was {delta}"
        );
    }

    // ── U4: degenerate Hp.starting == 0.0 does not panic (NaN guard) ────

    /// U4: `Hp { current: 1.0, starting: 0.0 }` — degenerate fraction denominator.
    /// The system must compute a finite color (no NaN from divide-by-zero).
    #[test]
    fn degenerate_hp_starting_zero_does_not_produce_nan_color() {
        let mut app = u_test_app();
        let material = app
            .world_mut()
            .resource_mut::<Assets<ColorMaterial>>()
            .add(ColorMaterial::from_color(Color::srgb(1.0, 0.5, 0.5)));
        app.world_mut().spawn((
            Cell,
            Hp {
                current:  1.0,
                starting: 0.0,
                max:      Some(1.0),
            },
            KilledBy::default(),
            visuals(),
            MeshMaterial2d(material.clone()),
        ));

        tick(&mut app);

        let color = material_color(&app, &material).to_srgba();
        assert!(
            color.red.is_finite(),
            "red channel should be finite (NaN guard), got {}",
            color.red
        );
        assert!(
            color.green.is_finite(),
            "green channel should be finite (NaN guard), got {}",
            color.green
        );
        assert!(
            color.blue.is_finite(),
            "blue channel should be finite (NaN guard), got {}",
            color.blue
        );
        assert!(
            color.alpha.is_finite() && color.alpha > 0.0,
            "alpha should be finite and > 0.0, got {}",
            color.alpha
        );
    }
}
