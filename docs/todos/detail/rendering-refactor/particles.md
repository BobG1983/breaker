# Particle System

CPU particle system built into `rantzsoft_vfx`. Individual entity per particle with custom `Material2d` (additive blending via `specialize()`).

**Research**: See [research/particle-system-design-patterns.md](research/particle-system-design-patterns.md) and [research/gpu-particle-system-bevy.md](research/gpu-particle-system-bevy.md) for full cross-engine and Bevy-specific research.

## Why CPU, Not GPU Compute

- Our game needs < 500 concurrent particles at peak. CPU handles 10,000+ at 200fps.
- GPU compute adds Bevy render-world complexity, compute shader pipeline, Metal/macOS risk.
- `bevy_particle_systems` benchmarks 10k particles at 190-200fps on a 2019 MacBook Pro.
- Individual entity particles are simplest to implement and integrate with ECS queries.
- If profiling ever shows need, the emitter abstraction is GPU-upgrade-friendly.

## Architecture

### Per-Particle Entity

Each particle is an entity with:

```rust
#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub lifetime_remaining: f32,
    pub lifetime_total: f32,
    pub rotation_speed: f32,
    pub gravity: Vec2,
}
```

Plus `Mesh2d` (tiny quad) + `MeshMaterial2d<ParticleMaterial>` (additive blend).

### ParticleMaterial (Additive Blending)

Custom `Material2d` with additive blend via `specialize()` — Bevy 0.18 has no built-in `AlphaMode2d::Add`.

```rust
impl Material2d for ParticleMaterial {
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            for target in fragment.targets.iter_mut().flatten() {
                target.blend = Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::One,  // additive
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent::OVER,
                });
            }
        }
        Ok(())
    }
}
```

HDR color values > 1.0 produce bloom via the Bloom post-process pass.

### Update System

```rust
fn update_particles(
    mut commands: Commands,
    time: Res<Time>,  // Time<Virtual> — particles slow during slow-mo
    mut query: Query<(Entity, &mut Transform, &mut Particle, &mut MeshMaterial2d<ParticleMaterial>)>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
) {
    for (entity, mut xf, mut p, mat_handle) in &mut query {
        p.lifetime_remaining -= time.delta_secs();
        if p.lifetime_remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        p.velocity += p.gravity * time.delta_secs();
        xf.translation += (p.velocity * time.delta_secs()).extend(0.0);
        xf.rotation = Quat::from_rotation_z(xf.rotation.to_euler(EulerRot::ZYX).0 + p.rotation_speed * time.delta_secs());

        // Alpha fade over lifetime — update material color
        let t = p.lifetime_remaining / p.lifetime_total;
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.alpha = t;
        }
    }
}
```

## Emitter Component

```rust
#[derive(Component, Clone, Debug, Deserialize)]
pub struct ParticleEmitter {
    pub mode: EmissionMode,
    pub spawn: SpawnParams,
}

pub enum EmissionMode {
    Continuous { rate: f32 },    // particles per second
    Burst { count: u32 },        // spawn N immediately, then emitter idles
    OneShot { count: u32 },      // spawn N, auto-despawn emitter when all particles dead
}

pub struct SpawnParams {
    pub lifetime: RangeF32,
    pub velocity_shape: VelocityShape,
    pub speed: RangeF32,
    pub size: RangeF32,
    pub color: Hue,
    pub hdr: HdrBrightness,
    pub gravity: Vec2,
    pub rotation_speed: RangeF32,
}

pub enum VelocityShape {
    Radial,                      // uniform in all directions
    Cone { angle: f32 },         // within a cone (half-angle)
    Directional { dir: Vec2 },   // biased toward a direction
}
```

### How Primitives Use Emitters

When the recipe system dispatches a SparkBurst step, it spawns an entity with a `ParticleEmitter(Burst)`. The emitter system reads the emitter, spawns particle entities, and (for OneShot) tracks alive count to auto-despawn.

| Primitive | EmissionMode | VelocityShape | Key Params |
|-----------|-------------|---------------|------------|
| SparkBurst | OneShot(count) | Radial | Short lifetime, gravity, high speed, small size |
| ShardBurst | OneShot(count) | Radial | Longer lifetime, rotation, medium speed, angular mesh |
| GlowMotes | Continuous(rate) | Radial (slow) | Long lifetime, low speed, large size, no gravity |
| TrailBurst | OneShot(count) | Directional | Medium lifetime, elongated mesh |

**Note:** `ElectricArc` is NOT a particle primitive despite appearing in the PrimitiveStep enum alongside particles. It is a **segmented line mesh** with per-frame jitter — see `rantzsoft_vfx.md` — ElectricArc Rendering section. It does not use the emitter/particle system.

## Performance Budget

- **Concurrent particle cap**: 8192 maximum. This is a **soft cap**, not a pre-spawned pool — if the cap is reached, new emitters skip spawning until existing particles expire. At our typical load (<500 concurrent), this is never hit.
- **No pooling**: Particles spawn on demand and despawn when lifetime expires. At <500 concurrent, archetype churn is negligible. Pooling adds complexity for no measurable gain at this scale.
- **Entity overhead**: trivial at this count
- **Fill rate**: keep particles small (8x8 to 32x32 px), additive blending is cheapest (order-independent, no sorting)
- **Upgrade path**: if profiling shows > 2000 particles needed, switch to CPU instancing (one draw call per emitter). GPU compute only if > 10,000.
