# 5e: Particle System

**Goal**: Build the CPU particle system in `rantzsoft_vfx` and implement the 5 core particle primitives that all later VFX steps will use.

Architecture: `docs/architecture/rendering/particles.md`

## What to Build

### 1. Per-Particle Entity System

Each particle is an individual entity with:
- `Particle` component (velocity, lifetime, rotation_speed, gravity)
- `Mesh2d` (tiny quad) + `MeshMaterial2d<ParticleMaterial>` (additive blend)
- `ParticleMaterial` — custom `Material2d` with additive blend via `specialize()`
- HDR color values >1.0 produce bloom via the Bloom post-process pass

### 2. Emitter System

`ParticleEmitter` component with `EmissionMode` and `SpawnParams`:
- `Continuous { rate }` — particles per second
- `Burst { count }` — spawn N immediately, then idle
- `OneShot { count }` — spawn N, auto-despawn emitter when all particles dead

`SpawnParams`: lifetime range, velocity shape (Radial/Cone/Directional), speed range, size range, color (Hue), HDR brightness, gravity, rotation speed range.

### 3. Update + Cleanup System

Each tick: apply gravity, advance position, rotate, fade alpha over lifetime. Despawn on lifetime expiry. Soft cap of 8192 concurrent particles — new emitters skip spawning if cap is reached.

### 4. Five Particle Primitives

Each primitive has a corresponding `PrimitiveStep` variant (for recipes) and a direct message type:

| Primitive | EmissionMode | VelocityShape | Key Params |
|-----------|-------------|---------------|------------|
| SparkBurst | OneShot(count) | Radial | Short lifetime, gravity, high speed, small size |
| ShardBurst | OneShot(count) | Radial | Longer lifetime, rotation, medium speed, angular mesh |
| GlowMotes | Continuous(rate) | Radial (slow) | Long lifetime, low speed, large size, no gravity |
| ElectricArc | OneShot(count) | Directional | Very short lifetime, high jitter |
| TrailBurst | OneShot(count) | Directional | Medium lifetime, elongated mesh |

### 5. Debug Visualization

- Particle count display in debug menu
- Per-type spawn test buttons (fire a burst of each type)
- Performance overlay showing active particle count vs soft cap

## What NOT to Do

- Do NOT implement specific VFX that use particles (those are 5i, 5l, 5m, etc.)
- Build the types and prove they work via debug triggers

## Dependencies

- **Requires**: 5c (rantzsoft_vfx crate exists)
- **Independent of**: 5d (post-processing) — can be done in either order
- **Enhanced by**: 5d (additive blending, bloom) — particles look better with post-processing but work without it

## Verification

- All 5 particle primitives spawn correctly via debug menu
- Particles use additive blending (light-on-dark compositing)
- Particles bloom with HDR intensity (if 5d is done)
- Particle count stays below soft cap (no leaks)
- OneShot emitters auto-despawn when all particles expire
- All existing tests pass
