# Phase 10: Polish & Ship

**Goal**: Make it a complete, shippable experience that maximizes game span.

## Core Polish

- **Visual polish**: Refine all shaders, particles, transitions. Every effect should feel intentional.
- **Audio polish**: Final sound design, music composition/commissioning. Adaptive audio tuning.
- **Performance**: Profile and optimize. Target 60fps on modest hardware with max-chaos builds.
- **Balance passes**: Difficulty curves, chip synergies, timer values, rarity weights. Data-driven tuning via RON hot-reload.

## Onboarding

- **Tutorial / onboarding**: Teach breaker mechanics gradually through gameplay, not text screens. First run is implicitly a tutorial (simple layout, generous timer, guaranteed chip offerings that demonstrate the system).
- **Progressive complexity**: First-time players see a curated subset of chips. Full pool unlocks through meta-progression. Prevents information overload while preserving discovery (Pillar 7).

## Sharing & Community (Pillar 9)

Features that extend game span by making runs shareable:

- **Seed sharing**: Copy seed to clipboard from run-end screen. "Try seed 42 with Momentum breaker."
- **Run replay**: Record inputs for seed-deterministic replay. Watch your own runs or others'.
- **Highlight capture**: Auto-screenshot or short clip on highlight moments (clutch clears, mass destruction). Easy share to social.
- **Daily challenge**: Prominent UI placement. "Today's seed" with community leaderboard.
- **Build sharing**: Export chip loadout as a shareable string/code. "Here's my build, try to get it on seed X."

## Run History & Stats

- **Per-run breakdown**: Full stats, highlights, build, seed, outcome — browseable history
- **Personal bests**: Per archetype, per modifier configuration
- **Achievement system**: Discoverable achievements for finding synergies, reaching tiers, performing specific feats. Another discovery layer.
- **Lifetime stats**: Total cells destroyed, total perfect bumps, favorite chip, most-used archetype

## Custom Material2d Rendering Pipeline

Migrate all 2D rendered entities (cells, bolt, breaker, walls, UI elements) from Bevy's built-in `ColorMaterial` to a custom `Material2d` implementation.

**Why — efficiency**: Currently every cell spawns its own `ColorMaterial` asset. With 200+ cells on screen, that's 200+ unique material binds per frame — no GPU batching possible. A custom `Material2d` with a per-instance storage buffer lets all cells share a single material bind group, collapsing hundreds of draw calls into one. The GPU renders the entire cell grid in a single batched pass.

**Why — creative freedom**: A custom shader unlocks effects impossible with `ColorMaterial`:
- Per-cell damage glow, pulse, and dissolve animations driven by a `health_fraction` uniform — no need to mutate material assets at runtime
- HDR bloom-aware emissive channels for overclock/amp visual feedback
- Procedural patterns (grid lines, energy fields, circuit traces) that make cells visually distinct without sprite assets
- Screen-space distortion effects (heat shimmer on damaged cells, shockwave ripple)
- Consistent visual language across all rendered entities when bolt, breaker, and cells all use the same shader family

**Scope**: Define a `CellMaterial` (and later `BoltMaterial`, `BreakerMaterial`) via `AsBindGroup` with a `@storage` buffer indexed by entity. Write WGSL shaders that read per-instance color, health fraction, and effect flags. Remove all per-entity `ColorMaterial` allocations. The damage visual system writes into the storage buffer instead of mutating material assets.

## Settings & Accessibility

- Resolution, audio, controls, accessibility options
- Colorblind modes (cell types must be distinguishable by shape/pattern, not just color)
- Input remapping
- Screen shake intensity slider (respect player preference while defaulting to maximum juice)

## Distribution

Core release infrastructure (GitHub Actions cross-compilation, itch.io butler, version bumping, changelog) is built in [Phase 4j](phase-4/phase-4j-release-infrastructure.md). This phase refines it:

- Steam integration if warranted by reception
- Platform-specific packaging (DMG, MSI, AppImage)
- Auto-update mechanism if distributing outside Steam
