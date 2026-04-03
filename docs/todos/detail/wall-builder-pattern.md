# Wall Builder Pattern

## Summary
Full typestate builder for Wall entities with WallDefinition RON asset, effect support, and per-side placement.

## Context
Breaker and Bolt entities both use typestate builders. Walls are currently manual tuple assembly in `spawn_walls`. The user wants walls elevated to the same pattern — not because walls are complex today, but to open design space for interesting walls with baked-in effects (e.g., a wall that applies SpeedBoost on impact, or a wall that damages the bolt). This also lets walls follow the same run-start sequence: definition → builder → spawn → dispatch_initial_effects.

## Design Decisions
- **Full typestate builder** — consistent with Breaker/Bolt, compile-time dimension enforcement
- **WallDefinition with serde defaults** — no `default.wall.ron`. A single `wall.wall.ron` with just `(name: "Wall")` gets all defaults via `#[serde(default)]`. Same pattern as BreakerDefinition.
- **WallDefinition supports effects** — `effects: Vec<RootEffect>` field, dispatched via `dispatch_initial_effects` at spawn time. Opens design space for effect-bearing walls.
- **Side is a builder dimension, not a definition field** — the builder has a Side dimension (`Left`, `Right`, `Ceiling`, `Floor`) that determines position and orientation from PlayfieldConfig. The definition doesn't know or care which side it's on. This means you could have different definitions per side (e.g., a "bouncy" left wall and a "damaging" right wall).
- **All walls are rebound walls** — walls always reflect bolt velocity. No `WallBehavior` enum needed. Bolt-lost stays as the current position-based check — it's simple, correct, and avoids corner-overlap ambiguity where CCD might resolve a floor wall collision before a side wall collision, causing unfair bolt-loss.
- **Floor walls are spawnable shield entities, not default infrastructure** — a Shield/SecondWind chip spawns a temporary floor wall with `Rebound` behavior that catches the bolt once, then despawns. These sit *above* the bolt-lost line, so no corner conflict with side walls.
- **Wall positioning** — walls sit one full thickness outside the visible playfield (e.g., left wall center at `playfield.left() - half_thickness`). The builder supports `Floor` as a Side for shield walls, positioned above the bolt-lost line (e.g., at `playfield.bottom() + some_offset`).
- **Example RON file** — `assets/examples/wall.example.ron` documenting all fields
- **Follow run-start sequence** — `WallRegistry` + `spawn_walls` reads from registry, builds via `Wall::builder().left(&playfield).definition(&def).spawn(&mut commands)`

## Typestate Dimensions
1. **Side** — `NoSide` → `Left` / `Right` / `Ceiling` / `Floor` (required). Side transitions store playfield dimensions from `PlayfieldConfig` but do NOT compute final position — position and scale are computed at `build()`/`spawn()` time using the resolved `half_thickness` (override > definition > default). `.definition()` and all configuration methods are only available after a side is chosen (on `WallBuilder<S: SideData>`).

**Not typestate dimensions (stored data with optional setters):**
2. **Visual** — optional `.visible(meshes, materials)` adds mesh + material. If not called, no visual components are added. Not a typestate dimension — same approach should apply to Bolt/Breaker builders later.
3. **Lifetime** — stored enum (`Permanent` / `Timed(f32)` / `OneShot`), default `Permanent`. Setters `.timed(duration)` and `.one_shot()` are only `impl`'d for `WallBuilder<Floor>` — compile-time restriction without extra generic parameters. Left/Right/Ceiling are always Permanent (no setter available).

Builder struct: `WallBuilder<S>` (1 generic param — Side only).

## Definition (available after Side is chosen)
`.definition(&WallDefinition)` sets `half_thickness`, `color_rgb`, and `effects` from the definition. It does NOT transition any typestate dimensions — it fills optional/stored data. `.definition()` and all override methods (`.with_half_thickness()`, `.with_color()`, `.with_effects()`) are only available on `WallBuilder<S: SideData>` — i.e., after a side transition. This avoids a hidden ordering constraint where definition would need to be called before side transitions for half_thickness to affect position. Instead, the flow is: pick a side first, then configure. You can build a wall without a definition (all explicit), or call `.definition()` and override individual fields. **Specific beats definition regardless of call order** — same semantics as bolt/breaker builders.

## WallDefinition Fields
```rust
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WallDefinition {
    pub name: String,
    #[serde(default = "default_half_thickness")]
    pub half_thickness: f32,             // default: 90.0 (PlayfieldConfig::wall_half_thickness())
    #[serde(default)]
    pub color_rgb: Option<[f32; 3]>,     // None = invisible (current behavior)
    #[serde(default)]
    pub effects: Vec<RootEffect>,        // dispatched at spawn time
}
```

## Wall Positioning (computed at build time from Side + PlayfieldConfig + resolved half_thickness)

Each `.left(&playfield)`, `.right(&playfield)`, `.ceiling(&playfield)`, `.floor(&playfield)` call stores the playfield dimensions in the side marker. The final position and scale are computed at `build()`/`spawn()` time using the resolved `half_thickness` (override > definition > default). This means `.definition()` and side transitions can be called in any order — both just store data, and resolution happens at build time.

Each side marker stores the playfield dimensions it needs (e.g., Left stores `playfield_left` and `half_height`). At build time, `half_thickness` is resolved and used to compute:

- `Left`:    center = `(playfield_left - ht, 0.0)`, half_extents = `(ht, half_height)`
- `Right`:   center = `(playfield_right + ht, 0.0)`, half_extents = `(ht, half_height)`
- `Ceiling`: center = `(0.0, playfield_top + ht)`, half_extents = `(half_width, ht)`
- `Floor`:   center = `(0.0, playfield_bottom)`, half_extents = `(half_width, ht)`

Left/Right/Ceiling sit fully outside the visible playfield area. Floor sits at the playfield bottom edge (matching current SecondWind behavior — top half overlaps the playfield to catch bolts before bolt-lost).

## Collision System Changes
- No changes to `bolt_wall_collision` — all walls rebound. Bolt-lost stays position-based.
- Shield/SecondWind floor walls are temporary spawnable entities that sit above the bolt-lost line and use normal rebound behavior.

## Scope
- **In**: `Wall::builder()`, `WallDefinition` struct + RON, `WallRegistry`, builder typestate (Side, Visual), Lifetime stored enum, `spawn_walls` migration, `second_wind::fire` migration to builder, `WallSize` removal, example RON, test migration (all manual Wall spawns → builder)
- **Out**: Wall rendering visuals (just capability, not visual design). Bolt-lost stays position-based (not migrated to floor wall). Shield chip floor wall is a future chip mechanic.

## Dependencies
- Depends on: Breaker builder pattern (establishes the pattern) — DONE
- Depends on: `dispatch_initial_effects` command — DONE
- Depends on: `RantzDefaultsPlugin` registry pattern — exists
- Blocks: Phase 5j (Walls & background visuals)

## Notes
- Current `spawn_walls` spawns 3 walls from `PlayfieldConfig`. After migration: reads `WallDefinition` from `WallRegistry`, calls builder 3 times (left, right, ceiling) with different Side dimensions.
- `WallSize` component is removed entirely. Walls use `Scale2D` + `Aabb2D` (computed from Side + half_thickness + playfield dimensions). Query filters use `With<Wall>` instead of `With<WallSize>`.
- `PlayfieldConfig.wall_half_thickness()` becomes the fallback default for `WallDefinition.half_thickness`.
- Walls use `CleanupOnNodeExit` (re-spawned each node), matching current behavior.
- **Floor wall is for chip mechanics**: Shield chip spawns a `Timed` floor wall (despawns after duration). SecondWind chip spawns a `OneShot` floor wall (despawns after one rebound). Both sit above the bolt-lost line. These are chip effects that CREATE wall entities via the builder, not part of the default `spawn_walls` sequence. The Lifetime dimension is what makes this work — `spawn_walls` uses the default `Permanent` lifetime for left/right/ceiling, while chip effects use `Timed`/`OneShot` for floor walls.
- **SecondWind migration**: `second_wind::fire` currently takes `&mut World`. Use the builder's `.build()` method to get a bundle, then `world.spawn((SecondWindWall, bundle))` for immediate entity visibility (required by the guard check). The builder call: `Wall::builder().floor(&playfield).one_shot().build()`. Using `.build()` instead of `.spawn()` avoids deferred commands complexity with exclusive world access.
- **Visual simplification**: Wall builder implements Visual as an optional chainable method (not a typestate dimension) — `WallBuilder<S>` with one generic param. This is the correct pattern: if you don't call `.visible()`, visual components aren't added. After this lands, backport the same simplification to Bolt and Breaker builders (remove `Unvisual`/`Rendered`/`Headless` typestate from both). Capture as a follow-up refactor.
- **Future design space**: different definitions per side, wall effects on Impacted(Wall), shield as a spawnable timed floor wall.

## Status
`ready`
