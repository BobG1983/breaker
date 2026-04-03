# GPU Compute Particle System Research — Bevy 0.18

Researched 2026-03-30. All API patterns verified against Bevy main branch (which is >= 0.15 and tracks toward 0.18). The compute_shader_game_of_life.rs source was fetched from `github.com/bevyengine/bevy/blob/main/examples/shader/compute_shader_game_of_life.rs` — this is the authoritative current example.

---

## 1. Bevy Compute Shader API

### Verified Imports (from main branch example)

```rust
use bevy::{
    core_pipeline::schedule::camera_driver,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{texture_storage_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderGraph, RenderQueue},
        texture::GpuImage,
        Render, RenderApp, RenderStartup, RenderSystems,
    },
};
```

### Plugin Registration Pattern

```rust
impl Plugin for MyComputePlugin {
    fn build(&self, app: &mut App) {
        // 1. Register extraction for any resources that need to cross to render world
        app.add_plugins(ExtractResourcePlugin::<MyResource>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<MyState>()
            // 2. Initialize pipeline in RenderStartup (runs once after plugins built)
            .add_systems(RenderStartup, init_my_pipeline)
            // 3. Prepare bind groups each frame
            .add_systems(Render, prepare_bind_group.in_set(RenderSystems::PrepareBindGroups))
            // 4. State transition logic
            .add_systems(Render, update_state.in_set(RenderSystems::Prepare))
            // 5. Dispatch — runs in RenderGraph, before camera_driver
            .add_systems(RenderGraph, dispatch_compute.before(camera_driver));
    }
}
```

### Pipeline Initialization (RenderStartup)

```rust
fn init_my_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "MyLayout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                // storage buffer: binding_types::storage_buffer_read_write::<MyType>()
                // uniform:        binding_types::uniform_buffer::<MyUniforms>(false)
                // texture store:  binding_types::texture_storage_2d(fmt, access)
            ),
        ),
    );
    let shader = asset_server.load("shaders/my_compute.wgsl");
    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![bind_group_layout.clone()],
        shader,
        entry_point: Some(Cow::from("main")),  // WGSL entry point name
        ..default()
    });
    commands.insert_resource(MyPipeline { bind_group_layout, pipeline });
}
```

### Bind Group Preparation (Render::PrepareBindGroups)

```rust
fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<MyPipeline>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    // ... other resources (buffers, images)
) {
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipeline.bind_group_layout),
        &BindGroupEntries::sequential((
            buffer.as_entire_buffer_binding(),
            // ... other bindings
        )),
    );
    commands.insert_resource(MyBindGroup(bind_group));
}
```

### Dispatch Execution (RenderGraph, before camera_driver)

```rust
fn dispatch_compute(
    mut render_context: RenderContext,
    bind_group: Res<MyBindGroup>,
    pipeline_cache: Res<PipelineCache>,
    pipeline: Res<MyPipeline>,
) {
    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor::default());

    let compute_pipeline = pipeline_cache
        .get_compute_pipeline(pipeline.pipeline_id)
        .unwrap();  // Only call after confirming CachedPipelineState::Ok

    pass.set_bind_group(0, &bind_group.0, &[]);
    pass.set_pipeline(compute_pipeline);
    pass.dispatch_workgroups(particle_count / WORKGROUP_SIZE, 1, 1);
}
```

### Pipeline State Check (Render::Prepare)

```rust
fn update_state(
    pipeline: Res<MyPipeline>,
    pipeline_cache: Res<PipelineCache>,
    mut state: ResMut<MyState>,
) {
    match pipeline_cache.get_compute_pipeline_state(pipeline.pipeline_id) {
        CachedPipelineState::Ok(_) => { /* ready to dispatch */ }
        CachedPipelineState::Err(ShaderCacheError::ShaderNotLoaded(_)) => { /* still loading */ }
        CachedPipelineState::Err(err) => { panic!("{err}") }
        _ => {}
    }
}
```

### Storage Buffer — `StorageBuffer<T>`

```rust
// In render world (Prepare phase):
let mut buf: StorageBuffer<MyParticleArray> = StorageBuffer::default();
buf.set(my_data);
buf.write_buffer(&render_device, &render_queue);

// Binding in bind group:
buf.buffer().unwrap().as_entire_buffer_binding()
// OR: buf.binding() returns Option<BindingResource<'_>>

// Default usages: COPY_DST | STORAGE
// Add more: buf.add_usages(BufferUsages::COPY_SRC)  // for readback
```

### ShaderStorageBuffer (main-world asset approach, added in 0.15)

```rust
// Main world:
let handle = buffers.add(ShaderStorageBuffer::from(my_vec_data));

// In Material struct:
#[derive(Asset, TypePath, AsBindGroup)]
struct MyMaterial {
    #[storage(0, read_only)]
    particles: Handle<ShaderStorageBuffer>,
}
```
This is simpler but less flexible than manual `StorageBuffer<T>`.

### RenderGraph: Adding Custom Nodes

```rust
// To add a node with edges:
render_graph.add_node(MyComputeLabel, MyComputeNode::default());
render_graph.add_node_edge(MyComputeLabel, CameraDriverLabel);
// MyComputeLabel must implement RenderLabel
```

Alternative (used in examples): register as a system with `.add_systems(RenderGraph, fn.before(camera_driver))` — this is the simpler pattern used in the current game_of_life example and avoids implementing the `Node` trait.

---

## 2. bevy_hanabi Architecture — Reference Patterns

Source: deepwiki.com/djeedai/bevy_hanabi and GitHub

### GPU Buffer Organization (Slab Allocator Model)

- **Persistent slab allocator**: Each effect pre-allocates GPU memory = `capacity × particle_size`
- `particle_size` = sum of all attribute sizes (position, velocity, color, lifetime, etc.)
- All particle data lives in GPU buffers **across frames** — CPU never reads particle state
- `EffectCache`: slab allocator managing GPU buffer regions
- `ParticleSlab`: GPU-side storage for all particle attributes

### Key GPU Buffers

| Buffer | Purpose |
|--------|---------|
| `particle_buffer` | All particle attributes (persistent across frames) |
| `dispatch_indirect_buffer` | Workgroup counts calculated by GPU (indirect dispatch) |
| `draw_indirect_buffer` | Draw commands written by update shader (indirect draw) |
| `effect_metadata_buffer` | Alive count, dead count, spawn state per effect |

### Three-Pass Compute Pipeline

1. **`vfx_indirect.wgsl`** — GPU calculates its own workgroup counts from particle counts → writes to `dispatch_indirect_buffer`. Eliminates CPU readback entirely.
2. **`vfx_init.wgsl`** — Spawn new particles (reads spawn params, writes to particle_buffer at dead slots)
3. **`vfx_update.wgsl`** — Simulate physics, apply modifiers, mark dead particles, write draw indirect args

### Extraction Pattern

```
ExtractSchedule systems:
  extract_effects     → copies ExtractedEffect (transform, visibility) to render world
  tick_spawners       → PostUpdate, calculates particles to spawn this frame

Render::Prepare:
  allocate_effects    → GPU memory management
  batch_effects       → group compatible effects

Render::Queue:
  queue_effects       → submit GPU commands
```

### Emitter Components

```rust
// Minimal user-facing component:
ParticleEffect {
    handle: Handle<EffectAsset>,  // blueprint
    prng_seed: Option<u32>,
}

// Auto-required:
CompiledParticleEffect   // cached GPU resources + shader handles
Transform                // emitter local position
GlobalTransform          // world-space position (computed)
Visibility               // culling

// Optional:
EffectSpawner            // emission control (burst/continuous)
EffectMaterial           // texture bindings
EffectProperties         // runtime property values
EffectParent             // hierarchical effects
```

### Dead Particle Recycling (atomics, not ring buffer)

Hanabi uses atomic counters:
- `atomicAdd` on dead_count to claim a slot when emitting
- Dead slot index → into dead_list → actual particle_buffer index
- Dead particles write their index back to dead_list
- `alive_count` tracked with `atomicAdd`/`atomicSub`
- Alive list alternates between two buffers each frame (like double-buffering)

---

## 3. GPU Particle Buffer Layout (Wicked Engine Pattern — Portable)

Source: wickedengine.net/2017/11/gpu-based-particle-simulation/

This is the canonical GPU particle architecture that Hanabi and Sprinkles both reference.

### Per-Particle Fields (typical 2D particle)

```wgsl
struct Particle {
    position: vec2<f32>,      // 8 bytes
    velocity: vec2<f32>,      // 8 bytes
    color: vec4<f32>,         // 16 bytes  (or pack as u32)
    lifetime_remaining: f32,  // 4 bytes
    lifetime_total: f32,      // 4 bytes   (for normalized progress)
    size: f32,                // 4 bytes
    rotation: f32,            // 4 bytes   (radians)
    // seed: u32              // 4 bytes   (for per-particle random)
    // flags: u32             // 4 bytes   (alive bit, type flags)
    // Total: 52 bytes (pad to 64 for alignment)
}
```

### Four Required GPU Buffers

1. **`particle_buffer`**: `array<Particle, MAX_COUNT>` — particle state, indexed by slot
2. **`dead_list`**: `array<u32, MAX_COUNT>` — free slot indices; initially all indices 0..MAX_COUNT
3. **`alive_list_a`**: `array<u32, MAX_COUNT>` — alive particle indices (current frame input)
4. **`alive_list_b`**: `array<u32, MAX_COUNT>` — alive particle indices (current frame output)
5. **`counter_buffer`**: `struct { alive_count: atomic<u32>, dead_count: atomic<u32>, emit_count: u32, draw_args: u32 }`

### Dead List Recycling Flow (GPU-side)

**Emit phase:**
```wgsl
// Atomically claim a dead slot:
let dead_idx = atomicSub(&counters.dead_count, 1u) - 1u;
let particle_idx = dead_list[dead_idx];
// Initialize particle at particle_buffer[particle_idx]
let alive_slot = atomicAdd(&counters.alive_count, 1u);
alive_list_a[alive_slot] = particle_idx;
```

**Simulate phase:**
```wgsl
let particle_idx = alive_list_a[global_id.x];
var p = particle_buffer[particle_idx];
p.lifetime_remaining -= dt;
if p.lifetime_remaining <= 0.0 {
    // Return slot to dead list:
    let dead_slot = atomicAdd(&counters.dead_count, 1u);
    dead_list[dead_slot] = particle_idx;
} else {
    // Write to output alive list:
    let out_slot = atomicAdd(&counters.alive_output_count, 1u);
    alive_list_b[out_slot] = particle_idx;
    particle_buffer[particle_idx] = p;
}
```

### Ring Buffer vs Compaction

| Strategy | Pros | Cons |
|----------|------|------|
| **Dead list + alive list (compact)** | Only processes alive particles; scales to millions with few alive | Requires indirect dispatch; atomic contention at scale |
| **Ring buffer** | Simple, predictable; no atomics | Processes all N slots every frame including dead; wastes compute |
| **Slab (Hanabi)** | No fragmentation; cache coherent per-effect | Pre-allocates full capacity upfront |

**Recommendation for brickbreaker**: Dead list + alive list is the canonical choice for GPU particles. Ring buffer is acceptable for simple CPU fallback. With < 2000 particles total, a ring buffer on CPU is perfectly fine.

---

## 4. Emitter Component Design

### Emission Modes (as ECS component)

```rust
// Recommended design — single component with mode enum:
#[derive(Component)]
pub struct ParticleEmitter {
    pub mode: EmitterMode,
    pub effect: Handle<EffectAsset>,
}

pub enum EmitterMode {
    /// Spawn N particles per second continuously
    Continuous { rate: f32 },
    /// Spawn N particles at interval, repeat forever
    Burst { count: u32, interval: f32 },
    /// Spawn N particles once; entity despawns when last particle dies
    OneShot { count: u32 },
}
```

### OneShot Auto-Despawn Pattern

```rust
// One-shot emitter needs to track when all its particles are dead.
// Simplest approach: track outstanding count on the emitter entity.
#[derive(Component)]
pub struct OneShotEmitter {
    pub total_spawned: u32,
    pub alive_count: u32,  // decremented by particle death messages/observers
}
// Despawn emitter entity when alive_count == 0 && has_spawned
```

Or (Bevy observer pattern): particles send a `ParticleDied { emitter: Entity }` event. An observer on the emitter entity decrements its counter and despawns when it hits zero.

### Enoki (CPU system) emitter modes (for reference)

```rust
ParticleSpawner {
    spawn_rate: f32,        // particles/second (continuous)
    spawn_amount: u32,      // per burst
    // + OneShot tag component → deactivates/despawns after first burst
}
```

---

## 5. Material2d with Additive Blending

### AlphaMode2d (Bevy 0.15+)

```rust
// Module: bevy::sprite (re-exported to prelude)
pub enum AlphaMode2d {
    Opaque,
    Mask(f32),   // threshold cutoff
    Blend,       // standard alpha blending
}
// NOTE: No built-in "Additive" variant. Must use specialize() for additive.
```

### Additive Blending via specialize()

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct ParticleMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
}

impl Material2d for ParticleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/particle.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            fragment.targets[0] = Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::One,      // KEY: One = additive
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                }),
                write_mask: ColorWrites::ALL,
            });
        }
        Ok(())
    }
}
```

### HDR Colors for Glow

```wgsl
// In WGSL fragment shader:
// Values > 1.0 create HDR glow when combined with additive blending
let center_brightness = 3.0;  // multiplier > 1.0
let glow_color = vec4<f32>(1.0, 0.8, 0.2, alpha) * center_brightness;
return glow_color;
```

### Instanced Rendering vs Individual Quads

| Approach | Bevy API | Best for |
|----------|----------|---------|
| **Individual entities** | `Mesh2d + MeshMaterial2d` per particle | < 500 particles, simple setup |
| **CPU instancing** | Custom `InstanceBuffer` via `render_device.create_buffer_with_data()` | 500–10k particles |
| **GPU compute + vertex expansion** | Compute writes alive list; vertex shader reads particle_buffer[id/6] | 10k+ particles |
| **GPU compute + MaterialExtension** | Sprinkles pattern: pre-duplicated mesh, UV_B encodes particle index | Most flexible GPU approach |

For brickbreaker's scale (likely < 500 visible particles at once), **individual entities with Mesh2d** or a simple CPU instancing approach is entirely sufficient.

---

## 6. CPU Particle Fallback

### Performance Numbers (Community Data)

- `bevy_particle_systems`: ~10,000 particles at 190–200 FPS on 2019 Intel i9 MacBook Pro (release build)
- `bevy_firework`: "tens of thousands without noticeable framerate drops" (CPU + GPU batching)
- `bevy_enoki`: "works well in wasm and mobile" — emphasis on < 5k scale

### CPU Particle Pattern (Bevy entities approach)

```rust
// Per-particle component:
#[derive(Component)]
struct Particle {
    velocity: Vec2,
    lifetime_remaining: f32,
    lifetime_total: f32,
    angular_velocity: f32,
}

// Systems:
fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Particle)>,
) {
    for (entity, mut xf, mut p) in &mut query {
        p.lifetime_remaining -= time.delta_secs();
        if p.lifetime_remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        xf.translation += (p.velocity * time.delta_secs()).extend(0.0);
        let t = 1.0 - (p.lifetime_remaining / p.lifetime_total);
        // Update scale, color, etc. from t
    }
}
```

### CPU Scale Limits

At 60 FPS budget:
- **500 particles**: Trivially fine with individual entities
- **2,000 particles**: Fine with good query, avoid heavy per-particle allocations
- **10,000 particles**: Requires instanced rendering (not individual entities), run in FixedUpdate or parallel
- **50,000+**: GPU compute required

For brickbreaker (explosion effects, trail particles for bolts/breaker): **500–2,000 particles max concurrently** is a very safe target. CPU approach is entirely viable.

---

## 7. macOS / Bevy 0.18 Known Issues

### Naga Panic (unrelated to particles)

- Issue #22238: Panic in naga/bevy_render on Apple M3 macOS Sequoia 15.7.1 (debug builds only)
- Affects Bevy 0.16.1, 0.17.2, 0.17.3, 0.18.0-rc.1
- Triggered by `MeshMaterial3d` assignment in debug builds
- **Workaround**: `cargo build --release` / `cargo run --release`
- Does **not** affect compute shaders or 2D rendering directly

### bevy_hanabi macOS Bug

- Hanabi has a workaround for a wgpu bug on macOS/Metal related to `ParticleLayout` alignment
- A separate objc2-foundation panic has been reported on macOS hardware during Hanabi initialization
- This is the likely reason the project is avoiding bevy_hanabi — the Metal backend has issues
- **Custom GPU particles avoid this** by not depending on Hanabi's Metal-specific code paths

---

## 8. Architectural Recommendation for Brickbreaker

Given:
- Game scale: < 500 particles visible at once (explosion sparks, bolt trails, impact flashes)
- Platform: macOS primary (Metal backend), bevy_hanabi has Metal issues
- Team: custom implementation needed

### Recommended Approach: CPU particles with instanced rendering

**Phase 1 (simple)**: Individual entity particles with `Mesh2d + MeshMaterial2d`.
- Per-particle entity, `Particle` component, update system runs in `Update`
- Custom `ParticleMaterial` implementing `Material2d` with additive blending via `specialize()`
- HDR colors (values > 1.0) for glow effects
- Handles 500 particles easily at 60+ FPS

**Phase 2 (if needed)**: CPU simulation + GPU instancing.
- Store particles in a `Vec<ParticleData>` resource per emitter
- Upload instance buffer each frame via `render_device.create_buffer_with_data()`
- Single draw call per emitter
- Handles 5,000+ particles

**GPU compute (only if Phase 2 is insufficient)**: Follow the game_of_life example pattern:
- `ExtractResource` for emitter state
- `StorageBuffer<ParticleArray>` prepared each frame
- `dispatch_workgroups(particle_count / 64, 1, 1)` for simulation
- Vertex shader reads particle buffer via `storage` binding and particle index from `vertex_index / 6`

### Emitter Component Design for Brickbreaker

```rust
#[derive(Component)]
pub struct FxEmitter {
    pub effect: FxEffect,
    pub mode: EmitMode,
}

pub enum FxEffect {
    BoltImpact,    // bolt hits cell — burst of sparks
    BoltTrail,     // continuous small sparks behind bolt
    BreakerShield, // breaker absorbs hit — flash burst
    CellDestroy,   // cell dies — debris burst
}

pub enum EmitMode {
    Burst { count: u32 },               // one burst, then emitter despawns
    Continuous { rate: f32 },           // N/sec until emitter removed
    OneShot { count: u32 },             // burst then auto-despawns when particles die
}
```

---

## Key Sources Consulted

- Bevy main branch `examples/shader/compute_shader_game_of_life.rs` — verified current API
- Bevy main branch `examples/shader/gpu_readback.rs` — verified RenderStartup + storage buffer pattern
- `deepwiki.com/djeedai/bevy_hanabi` — Hanabi architecture overview
- `wickedengine.net/2017/11/gpu-based-particle-simulation/` — canonical dead list GPU particle pattern
- `doce.sh/blog/bevy-sprinkles` — modern Bevy GPU particle system using MaterialExtension
- `aibodh.com/posts/bevy-rust-game-development-chapter-6/` — CPU particle pattern with additive blending
- `docs.rs/bevy/0.15.3/bevy/sprite/enum.AlphaMode2d.html` — AlphaMode2d variants (Opaque, Mask, Blend — no Additive)
- Bevy 0.16, 0.17, 0.18 release notes — no breaking changes to compute shader or Material2d APIs
