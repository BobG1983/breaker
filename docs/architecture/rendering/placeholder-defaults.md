# Placeholder Defaults

Phase 5 placeholder values. All values here are **tuned in Phase 7**. These exist so implementation can proceed without blocking on creative tuning.

## Temperature Palette

```ron
// In GraphicsDefaults RON (shared/)
temperature_palette: (
    cool_grid: MidnightBlue,
    hot_grid: DarkRed,
    cool_bloom: SteelBlue,
    hot_bloom: OrangeRed,
    cool_wall: SlateGray,
    hot_wall: Sienna,
),
```

## Grid

```ron
// In VfxConfig
grid_line_spacing: 32.0,    // pixels between grid lines
grid_line_thickness: 0.5,   // pixel width
grid_glow_intensity: 0.15,  // very subtle
```

## Void Background

```ron
clear_color: (0.02, 0.02, 0.06, 1.0),  // #050510 deep blue-black
```

## Particle Sizes (per primitive)

| Primitive | Quad Size (px) | Notes |
|-----------|---------------|-------|
| SparkBurst | 4x4 | Tiny, numerous |
| ShardBurst | 8x12 | Elongated angular fragments |
| GlowMotes | 16x16 | Soft, large, ambient |
| ElectricArc | 3x3 | Tiny, high jitter makes them streak |
| TrailBurst | 4x16 | Elongated along velocity |

## Entity Glow Defaults

Placeholder `GlowParams` for entities that don't have RON rendering blocks yet:

```ron
// Default glow (used by AttachVisuals when no glow specified)
default_glow: (
    core_brightness: HdrBrightness(1.0),
    halo_radius: 2.5,
    halo_falloff: 2.0,
    bloom: BloomIntensity(0.7),
),
```

## Timer Wall

```ron
timer_wall_intensity: 0.8,
timer_wall_pulse_speed_normal: 0.5,
timer_wall_pulse_speed_danger: 4.0,   // <25% time
timer_wall_danger_threshold: 0.25,
```

## Life Orbs

```ron
life_orb_radius: 6.0,
life_orb_spacing: 16.0,    // between orb centers
life_orb_offset_y: -24.0,  // below breaker
```

## Node Progress Ticks

```ron
tick_width: 3.0,
tick_height: 8.0,
tick_spacing: 12.0,         // along wall
tick_bright_intensity: 1.2,  // current node
tick_dim_intensity: 0.4,     // completed
tick_outline_intensity: 0.1, // upcoming
```

## Modifier Diminishing Returns

```ron
// Default curve (used when no per-modifier curve is configured)
default_dr_curve: [1.0, 0.7, 0.4, 0.2],

// Per-modifier overrides (placeholder — all use default for now)
// trail_length: [1.0, 0.7, 0.4, 0.2],
// glow_intensity: [1.0, 0.8, 0.6, 0.4],
```

## Screen Shake Tiers

| Tier | Displacement (px) | Duration (frames) | Decay Rate |
|------|-------------------|-------------------|------------|
| Micro | 1.5 | 2 | 0.7 |
| Small | 4.0 | 3 | 0.6 |
| Medium | 8.0 | 5 | 0.5 |
| Heavy | 16.0 | 8 | 0.4 |

## Slow Motion Presets

| Event | Factor | Duration (s) | Ramp In (s) | Ramp Out (s) |
|-------|--------|-------------|-------------|--------------|
| Bolt lost | 0.3 | 0.3 | 0.05 | 0.1 |
| Life lost | 0.2 | 0.5 | 0.05 | 0.15 |
| Run won | 0.0 | 1.0 | 0.1 | 0.0 |
| Run over | 0.15 | 2.0 | 0.1 | 0.3 |

## Shield Barrier

```ron
shield_hex_scale: 8.0,        // honeycomb cell size
shield_pulse_speed: 1.5,
shield_intensity: 0.6,
shield_crack_radius: 0.15,    // per crack seed, normalized
```

## Chip Card

```ron
card_width: 120.0,
card_height: 160.0,
card_spacing: 20.0,
card_icon_size: 24.0,
card_border_glow: 0.8,
card_selected_scale: 1.1,
card_dim_intensity: 0.4,
```

## Transition Timing

```ron
transition_duration: 0.4,     // seconds, all styles
collapse_tile_count: (8, 6),  // columns x rows
```

## Font

Monospace font for timer readout, seed display, numeric data: **placeholder — use any monospace TTF available** (e.g., JetBrains Mono, Fira Mono, or Source Code Pro). Final font chosen in Phase 7 with the audio/polish pass. Asset path: `assets/fonts/mono.ttf`.
