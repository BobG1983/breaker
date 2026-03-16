---
name: Review — fix/review-findings branch (2026-03-16)
description: Bevy API review of changes on fix/review-findings branch
type: project
---

## Bevy API Review (Bevy 0.18.1)

Files reviewed:
- `src/breaker/systems/init_breaker_params.rs`
- `src/ui/systems/animate_fade_out.rs`
- `src/ui/plugin.rs`
- `src/bolt/plugin.rs`
- `src/breaker/systems/spawn_breaker.rs`
- `src/breaker/behaviors/init.rs`
- `src/breaker/behaviors/consequences/life_lost.rs`
- `src/breaker/behaviors/consequences/bolt_speed_boost.rs`
- `src/bolt/systems/bolt_lost_feedback.rs`
- `src/breaker/systems/bump_feedback.rs`
- `src/physics/ccd.rs`

---

### Deprecated Patterns [Clean]

No deprecated bundle types, no `EventReader`/`EventWriter`, no `Parent` component, no
`despawn_recursive`. All patterns match confirmed-correct Bevy 0.18.1 idioms.

---

### System Parameters [Clean]

`animate_fade_out` — `Res<Time>, Commands, Query<(Entity, &mut FadeOut, &mut TextColor)>`
— no conflicts, straightforward and correct.

`init_breaker_params` — `Commands, Res<BreakerConfig>, Query<Entity, (With<Breaker>, Without<BreakerMaxSpeed>)>`
— no conflicts.

`spawn_breaker` — `Commands, Res<BreakerConfig>, ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>, Query<Entity, With<Breaker>>`
— distinct resource types, no conflicts.

`reset_breaker` — `Res<PlayfieldConfig>, mut Query<BreakerResetQuery, With<Breaker>>`
— no conflicts.

All observer signatures (`On<LoseLifeRequested>`) match the 0.18.1 observer trigger pattern.

---

### Query Syntax [Clean]

- `Query<(Entity, &mut FadeOut, &mut TextColor)>` — correct, no filter needed since the
  system should only run on entities that have all three components.
- `Query<Entity, (With<Breaker>, Without<BreakerMaxSpeed>)>` — correct filter usage.
- `query.single()` / `lives_query.single()` / `status_panel.single()` — all use the
  `let Ok(...) = ... else { return; }` fallible pattern, which is correct for 0.18.1
  where `single()` returns `Result`.
- `query_filtered::<Entity, With<FadeOut>>()` in tests — correct.
- `query_filtered::<Entity, With<LivesDisplay>>()` in tests — correct.

---

### Derive Macros & Traits [Clean]

- `FadeOut` — `#[derive(Component)]` present in `shared.rs` (line 124). Correct.
- `LivesCount` — `#[derive(Component)]` present. Correct.
- `LivesDisplay` — `#[derive(Component)]` present. Correct.
- `LoseLifeRequested` — `#[derive(Event)]` present. Correct (observers use `Event`,
  not `Message`, in 0.18.1).
- All other components reviewed have correct derives.

---

### Schedule & State [Clean]

- `animate_fade_out` registered in `Update` with `.run_if(in_state(PlayingState::Active))` —
  correct. Fade-out is frame-rate driven (uses `Res<Time>` delta), so `Update` is appropriate.
- `(update_timer_display, animate_fade_out).run_if(in_state(PlayingState::Active))` — correct
  grouping; both systems share the same run condition.
- `bolt/plugin.rs` — `animate_fade_out` correctly removed from bolt plugin; it now lives
  solely in `ui/plugin.rs`. No trailing-semicolon or syntax issue found — the last
  `.add_systems(...)` ends with `;` on line 46 which is idiomatic Rust (method chaining
  in a statement context).
- `(spawn_side_panels, ApplyDeferred, spawn_timer_hud).chain()` in `OnEnter(GameState::Playing)` —
  previously confirmed correct.

---

### Asset & Handle [Clean]

No new asset loading patterns introduced in these changes.

---

### Unverified (Needs Lookup)

#### `commands.entity(entity).insert_if_new((BumpPerfectMultiplier(1.0), BumpWeakMultiplier(1.0)))` — VERIFIED CORRECT

`EntityCommands::insert_if_new` exists in Bevy 0.18.1 (confirmed via docs.rs). It inserts
a bundle without overwriting existing components — exactly the semantics needed here.
The tuple `(BumpPerfectMultiplier(1.0), BumpWeakMultiplier(1.0))` is a valid `Bundle`.

#### `f32::midpoint(playfield.left(), playfield.right())` — VERIFIED CORRECT

`f32::midpoint` was stabilized in Rust 1.85.0 (released 2025-02-20), which also stabilized
the Rust 2024 edition. This project uses `edition = "2024"` in `Cargo.toml`. The minimum
Rust version for 2024 edition is 1.85.0, so `f32::midpoint` is guaranteed available.

---

### Summary

**Clean.** All five files specified in the task, plus supporting files reviewed as context,
use correct Bevy 0.18.1 APIs with no deprecated patterns, no system parameter conflicts,
no query syntax errors, and no wrong derive macros. The two items that required external
verification — `insert_if_new` and `f32::midpoint` — are both correct for this project's
Bevy and Rust versions.
