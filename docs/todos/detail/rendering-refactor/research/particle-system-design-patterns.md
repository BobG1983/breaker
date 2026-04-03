# GPU Particle System Design Patterns — Research Summary

Research date: 2026-03-30. For building a custom 2D GPU particle system in Rust/Bevy.

---

## 1. GPU Particle System Architecture Patterns

### Production Engine Architectures

**Unreal Niagara** (the gold standard for programmable VFX):
- Hierarchical: System > Emitter > Module > Parameter
- Modules are composable behavior blocks (velocity, forces, collision, color-over-life)
- Supports both CPU and GPU simulation paths per emitter
- Simulation Stages allow iterative per-particle updates within a single frame (e.g., constraint solving)
- Data namespaces: `Particle.Position`, `Emitter.Age`, `System.DeltaTime`
- Modules are visual graphs created by technical artists, composed into stacks by VFX artists

**Unity VFX Graph**:
- Node-based graph that compiles to GPU compute shaders
- Pre-allocates GPU buffers based on declared capacity — set realistic maximums to avoid waste
- Supports 100,000+ particles at 60fps on modern GPUs
- Separate from the legacy CPU-based Shuriken system

**PopcornFX** (middleware, used in many AAA titles):
- Batch-oriented: scripts run once per batch of particles, not once per particle
- Aggressive constant folding: compiler classifies values as per-particle, per-batch, per-frame, or constant
- Consolidates render batches: particles from different emitters with identical renderers share draw calls
- Can pool 10,000 effect instances into a single medium containing 100,000 total particles

**Godot GPU Particles**:
- Particle shaders (`shader_type particles`) update per-particle properties via built-ins (TRANSFORM, VELOCITY, COLOR, ACTIVE)
- Sub-emitter support: particles can trigger secondary emitters on birth/death/collision
- Attractors and collision nodes for environment interaction
- Simpler than Niagara/VFX Graph but covers most 2D/3D game needs

**Bevy Hanabi** (the existing Bevy ecosystem solution):
- Four GPU passes: `vfx_indirect` (workgroup calculation) > `vfx_init` (spawn) > `vfx_update` (simulate + write draw commands) > `vfx_render` (vertex/fragment)
- GPU-driven: the GPU calculates its own dispatch parameters, no CPU readback
- Modifier system: Init modifiers (spawn), Update modifiers (per-frame), Render modifiers (appearance)
- Slab allocator: each effect's particles occupy contiguous memory, allocated once and reused
- Integrates with both Bevy 2D and 3D pipelines
- Dynamically generates WGSL from modifier configuration

**Bevy Sprinkles** (newer alternative):
- Extends `StandardMaterial` via `MaterialExtension` with storage buffer for sorted particles
- Pre-duplicates mesh geometry per emitter; vertex shader reads transforms from sorted buffer using particle index stored in UV_B
- Deterministic per-particle random seed (multiply-xorshift hash), deriving all randomness from the seed — mirrors Godot's approach
- Simpler than Hanabi but GPU-only, no CPU/2D fallback

### Common Architectural Pattern

All production systems converge on this pattern:

```
CPU Side:                    GPU Side:
Emitter Config ──────────>   Spawn Compute Shader
  (rate, burst, params)        │
                               v
                             Simulate Compute Shader
                               │ (update position, velocity, lifetime)
                               │ (kill dead particles, write alive list)
                               v
                             [Optional: Sort / Compact]
                               │
                               v
                             Indirect Draw
                               (render only alive particles)
```

Key insight: the CPU's only job is to say "spawn N particles with these parameters." All lifecycle management happens on the GPU.

---

## 2. Buffer Management Strategies

### Strategy A: Dead List + Double Alive List (Wicked Engine pattern)

**The industry standard for production GPU particles.**

Buffers:
- **Particle Buffer**: Fixed-size array of particle structs (position, velocity, color, lifetime, age). Size = MAX_PARTICLES.
- **Dead List**: Stack of indices into Particle Buffer for dead/available particles. Initially full (all indices).
- **Alive List A**: Indices of living particles (previous frame + newly emitted this frame).
- **Alive List B**: Indices of particles that survived simulation this frame.
- **Counter Buffer**: 4 atomic counters — alive_count, dead_count, real_emit_count, post_sim_count.
- **Indirect Args Buffer**: Dispatch/draw parameters written by GPU.

Flow per frame:
1. **Emit pass**: Atomically pop from Dead List, init particle in Particle Buffer, push to Alive List A
2. **Simulate pass**: Read Alive List A. For each: update particle. If alive, push to Alive List B. If dead, push to Dead List.
3. **Draw**: Use Alive List B count as instance count via indirect draw.
4. **Swap**: A becomes B for next frame.

Tradeoffs:
- (+) Only processes alive particles — 100 alive out of 1M pool costs 100 updates
- (+) No fragmentation, no compaction pass needed
- (+) Constant memory footprint
- (-) Requires 4 atomic operations per particle per frame
- (-) More complex than simpler approaches
- (-) Two alive lists double index buffer memory

### Strategy B: Flip-Flop Double Buffer (GPUParticles / DX11 pattern)

Two complete particle buffers. Simulate reads from buffer A, writes survivors to buffer B. Dead particles simply aren't copied. Active particles naturally compact into indices 0..n.

Tradeoffs:
- (+) Implicit compaction — no separate compaction pass
- (+) Simpler than dead/alive lists
- (-) Doubles particle data memory (two full copies)
- (-) Every particle must be read even if dead (iterate full buffer to find alive ones, or use alive index list)

### Strategy C: Fixed Ring Buffer

Particles are always appended at the head. Old particles die by age. No recycling — when the buffer wraps, old slots are overwritten.

Tradeoffs:
- (+) Simplest implementation — no atomic ops, no lists, no compaction
- (+) Perfect for constant-rate emitters where all particles have the same lifetime
- (-) Cannot recycle individual particles early
- (-) Poor for variable-lifetime or burst-heavy effects
- (-) Maximum particle count = buffer size, period

### Strategy D: Free List with Atomic Stack (Simple GPU Pool)

Single particle buffer + single free list (atomic stack). Dead particles push their index onto the free list. Spawn pops from the free list.

Tradeoffs:
- (+) Simpler than double alive list
- (+) Single particle buffer (no double copy)
- (-) Still iterates all slots during simulation (or needs a separate alive list)
- (-) Can have "holes" in the buffer — rendering must skip dead particles or use indirect

### Recommendation for a 2D Arcade Game

**Start with Strategy C (Ring Buffer) or Strategy D (Free List)**, not the full Wicked Engine pattern. For a 2D game with primarily additive blending and short-lived particles (sparks, impacts, trails), a ring buffer covers most effects. Graduate to dead list + alive list only if you need variable-lifetime particles with high throughput.

---

## 3. Particle Lifecycle on GPU

### The Three-Phase Model

Every production GPU particle system uses three compute dispatches per frame:

**Phase 1 — Emit (Spawn)**
- CPU uploads: spawn count, emitter position, initial velocity range, color, lifetime range
- Compute shader runs `spawn_count` invocations
- Each invocation atomically acquires a slot (from dead list or ring buffer head)
- Initializes particle: position, velocity (randomized within range), lifetime, age=0, color
- Random values: either from a noise texture, or from a deterministic hash of (particle_index, frame_count)

**Phase 2 — Simulate (Update)**
- Dispatched for alive particle count (via indirect dispatch to avoid CPU readback)
- Each invocation: read particle, apply forces, integrate position, decrement lifetime
- If lifetime <= 0: mark dead, return index to dead list
- If alive: write to alive list, write to particle buffer
- Optionally: write draw-indirect args (alive count -> instance count)

**Phase 3 — Render**
- Indirect draw using alive count
- Vertex shader reads particle data from storage buffer
- Expands point to quad (billboard) or uses instanced quad mesh
- Fragment shader: texture sample, color, alpha

### CPU-GPU Communication

The CPU's only responsibilities:
1. Set spawn count per frame (via uniform buffer or push constant)
2. Set emitter parameters (position, velocity ranges, colors — via uniform buffer)
3. Submit compute dispatches and draw commands
4. **Never read back from GPU** — this is the cardinal rule

All spawn counts, alive counts, and dispatch sizes are determined GPU-side via indirect dispatch:
```
// GPU writes its own dispatch args:
dispatch_indirect_buffer.x = ceil(alive_count / WORKGROUP_SIZE)
dispatch_indirect_buffer.y = 1
dispatch_indirect_buffer.z = 1
```

### Deterministic Randomness

Bevy Sprinkles and Godot both use deterministic per-particle seeds:
- Seed = hash(particle_index, spawn_frame)
- All random decisions (velocity spread, color variation, size) derive from this seed
- Enables deterministic replay and avoids shipping RNG state to GPU

---

## 4. 2D-Specific Considerations

### Depth Sorting: Usually Not Needed

For a 2D game with additive blending, **depth sorting is unnecessary**. Additive blending is order-independent — the result of A + B is the same regardless of draw order. This is a massive simplification over 3D particle systems that need alpha blending with depth sorting.

When you DO need sorting (alpha-blended smoke, non-additive particles):
- Sort by Y-coordinate (or a layer value) on the GPU
- Bevy Sprinkles does this with a dedicated sort buffer
- But for a 2D arcade game, additive blending covers 90%+ of effects (sparks, glows, impacts, trails)

### Billboarding: Not Needed

3D particle systems spend effort making quads face the camera. In 2D, all quads already face the camera. Skip the billboard math entirely.

### Overdraw is the Bottleneck

In 2D, particles are typically large, overlapping quads with alpha/additive blending. The bottleneck is **fill rate** (pixel shader invocations per pixel), not vertex count or simulation cost.

Optimizations:
- **Off-screen rendering at reduced resolution**: NVIDIA GPU Gems 3 technique — render particles to a 1/4 or 1/16 size render target, composite back. Achieved 2x frame rate improvement (25 FPS -> 51 FPS) in their benchmarks.
- **Group by blend mode**: Minimize state switches. Render all additive particles together, then all alpha-blended.
- **Particle texture atlasing**: Single texture bind for all particle types.
- **Small particles**: Fewer pixels = less fill rate. For distant/small effects, reduce quad size aggressively.

### Simplifications Available in 2D

| 3D Concern | 2D Simplification |
|---|---|
| View-space billboarding | Not needed — quads always face camera |
| Depth sorting | Not needed for additive blending |
| Depth buffer collision | Not needed — use 2D spatial queries |
| Lighting interaction | Usually not needed — particles are self-illuminated |
| LOD / distance culling | Simpler — just screen bounds culling |
| 3D force fields | 2D force fields (gravity, radial, turbulence) |

---

## 5. Emitter Design Patterns

### Production Engine Approaches

**Niagara (UE5)**: Component-based modules
- Each behavior is a "module" (a visual graph producing code)
- Modules compose into a stack per emitter
- Artist adds/removes/reorders modules without code changes
- Extremely flexible but complex to implement

**VFX Graph (Unity)**: Node-based graph
- Contexts: Spawn, Initialize, Update, Output
- Each context contains blocks (nodes) defining behavior
- Graph compiles to compute shaders
- More visual but rigid structure

**PopcornFX**: Script-based with batch execution
- Particle behavior defined in script nodes
- Script runs per-batch, not per-particle
- Spawners define emission, mediums define simulation + rendering

### Practical Emitter Abstraction for a Custom System

For a 2D Bevy game, the cleanest abstraction:

```
Emitter {
    // Emission control
    mode: EmissionMode,        // Continuous(rate), Burst(count), OneShot(count)

    // Spawn parameters (what each new particle looks like)
    spawn: SpawnParams {
        lifetime: RangeF32,     // min..max seconds
        velocity: VelocityShape, // Cone, Radial, Directional, Random
        speed: RangeF32,
        size: RangeF32,
        color: ColorGradient,   // or start_color..end_color
        // ...
    },

    // Update parameters (how particles evolve)
    update: UpdateParams {
        gravity: Vec2,
        drag: f32,
        size_over_life: Curve,
        color_over_life: Gradient,
        // ...
    },

    // Render parameters
    render: RenderParams {
        blend_mode: BlendMode,  // Additive, Alpha, Multiply
        texture: Handle<Image>,
        // ...
    },
}
```

**EmissionMode** covers the three core patterns:
- `Continuous(rate: f32)` — fire, smoke, ambient dust
- `Burst(count: u32)` — explosion, impact, pickup
- `OneShot(count: u32)` — same as burst but auto-despawns emitter after

This maps cleanly to the GPU: each frame, the CPU computes how many particles to spawn based on mode + delta_time, uploads the count, and the GPU handles the rest.

### Sub-Emitters

Advanced but useful: particles can trigger secondary emitters on death. For a brick breaker:
- Bolt impact spawns sparks (primary) — each spark's death spawns a tiny flash (secondary)
- Not needed for MVP but worth designing the emitter hierarchy to support later

---

## 6. Performance Budgets

### Industry Guidelines

| Platform | Typical Budget | Particle Count |
|---|---|---|
| Mobile | 1-2ms | 200-500 active |
| Mid-range PC | 1-2ms | 2,000-5,000 active |
| High-end PC | 1-2ms | 10,000-50,000 active |
| GPU-driven (modern) | 1-2ms | 100,000+ active |

The universal target is **1-2ms of frame time** for the entire particle system at 60fps (16.6ms per frame).

### Effect Complexity Classes (PopcornFX scale)

- **Light**: < 500 particles (single impact, small trail)
- **Medium**: 500-2,000 particles (fire, waterfall, sustained effect)
- **Heavy**: 2,000-8,000 particles (large explosion, dense fog)
- **Extreme**: 8,000+ particles (weather systems, dense volumetric effects)

### Budget for a 2D Arcade Brick Breaker

For a 2D arcade game with additive blending:

- **Per-effect**: 50-200 particles per individual effect (bolt impact, cell shatter, power-up)
- **Simultaneous effects**: 5-15 active at once during intense gameplay
- **Total active particles**: ~500-3,000 at peak
- **Comfortable GPU budget**: 5,000-10,000 max capacity (gives 2-3x headroom)
- **Frame time target**: < 0.5ms (particles should be cheap, leaving budget for gameplay)

A GPU particle system with a **10,000 particle pool** is more than sufficient and will barely register on frame time. Even a CPU system handles this count with SIMD (documented: 24,000 particles across 200 emitters in ~2.25ms with SIMD + multithreading).

### Fill Rate is the Real Concern

For 2D particles, the bottleneck is overdraw, not particle count. A single large particle covering half the screen costs more than 1,000 tiny particles. Budget by **total particle pixel coverage** not just count.

Practical rule: keep average particle size small (8x8 to 32x32 pixels), use additive blending (cheaper than alpha), and limit the number of full-screen effects.

---

## 7. Open Source Rust/wgpu Particle Implementations

### bevy_hanabi (Most Mature)

- **GitHub**: github.com/djeedai/bevy_hanabi
- **Architecture**: Full GPU-driven pipeline with 4 compute/render passes
- **Buffer management**: Slab allocator, indirect dispatch, indirect draw
- **Modifier system**: Init/Update/Render modifiers compose behavior
- **Spawner modes**: Rate-based, burst, once
- **2D support**: Yes, via `2d` cargo feature
- **WASM**: Supported via WebGPU (since v0.13)
- **Pros**: Feature-rich, well-maintained, good Bevy integration
- **Cons**: Complex codebase, dynamically generates WGSL, heavy dependency for simple needs

### Sprinkles (Newer, Simpler)

- **Website**: doce.sh/blog/bevy-sprinkles
- **Architecture**: MaterialExtension-based, storage buffer for sorted particles
- **Key pattern**: Pre-duplicated mesh geometry with particle index in UV_B; vertex shader reads transforms from sorted buffer
- **Sorting**: Dedicated sort pass (compute > sort > render)
- **Randomness**: Deterministic per-particle seed via multiply-xorshift hash
- **Pros**: Simpler than Hanabi, good editor, integrates with StandardMaterial
- **Cons**: GPU-only, no 2D pipeline integration, newer/less proven

### sparticles (wgpu + EGUI)

- **GitHub**: github.com/Norlock/sparticles
- **Architecture**: Pure wgpu (not Bevy), with EGUI editor
- **88% Rust, 12% WGSL** — moderate shader complexity
- **Features**: Bloom post-processing, model/material import
- **Pros**: Standalone, editor included
- **Cons**: Not Bevy-integrated, limited documentation

### par-particle-life (WebGPU Compute)

- **Crates.io**: par-particle-life
- **Architecture**: WebGPU compute shaders for particle physics
- **Focus**: Particle life simulation (attraction/repulsion rules)
- **Pros**: Clean compute shader example in Rust
- **Cons**: Simulation demo, not a general-purpose particle system

### bevy_app_compute / bevy_easy_compute

- **GitHub**: github.com/Kjolnyr/bevy_app_compute
- **Purpose**: Simplifies compute shader integration with Bevy
- **Pattern**: Declare shaders via structs implementing `ComputeShader` trait
- **Useful as**: A building block for custom particle systems rather than a particle system itself

### Vulkan Tutorial Particle Example

- **URL**: vulkan-tutorial.com/Compute_Shader
- **Architecture**: Minimal — single storage buffer, single compute shader, single draw call
- **Pattern**: Double-buffered SSBOs (read previous frame, write current frame), synchronized via semaphores
- **Workgroup**: 256 invocations per workgroup, dispatch = particle_count / 256
- **Rendering**: Same buffer used for both compute (write) and vertex (read) — dual-use buffer
- **Simplest viable reference** for understanding the core pattern

---

## 8. Simplest Viable GPU Particle System

### Minimum Architecture

For a 2D game starting simple and iterating:

```
Components:
├── 1 Storage Buffer (particle data: pos, vel, color, size, lifetime, age)
├── 1 Uniform Buffer (delta_time, spawn_count, emitter_pos, emitter_params)
├── 1 Compute Shader (combined spawn + update)
├── 1 Vertex/Fragment Shader (instanced quad rendering)
└── 1 Indirect Draw Buffer (instance_count written by compute)
```

### Minimum Viable Feature Set

1. **Spawn**: CPU sets spawn_count, compute shader initializes N new particles at end of buffer
2. **Simulate**: Compute shader updates all particles (position += velocity * dt, age += dt)
3. **Kill**: If age > lifetime, mark particle as dead (set a flag or zero the size)
4. **Render**: Instanced quad drawing, vertex shader reads particle data from storage buffer, skips dead particles via size=0
5. **Blend**: Additive blending only (no sorting needed)

### The Simplest Possible Implementation (Ring Buffer)

For a first pass, skip all the dead list complexity:

```
Buffer: [particle_0, particle_1, ..., particle_N-1]  (fixed size, e.g., 4096)
Head pointer: uniform u32 (wraps around)

Each frame:
1. CPU: head += spawn_count (mod N)
2. CPU: upload uniform { delta_time, spawn_count, head, emitter_params }
3. GPU Compute:
   - For invocations [head - spawn_count .. head]: initialize new particle
   - For all invocations: if age < lifetime, update position/velocity/age
4. GPU Render:
   - Instanced draw, N instances
   - Vertex shader: if age >= lifetime, emit degenerate quad (size=0)
   - Fragment shader: sample texture * particle_color, additive blend
```

This is ~100 lines of WGSL + the Bevy pipeline integration code. No atomic operations, no dead lists, no alive lists, no indirect dispatch. Just a buffer, a compute shader, and instanced drawing.

### Iteration Path (Simple to Production)

| Level | Feature | Complexity |
|---|---|---|
| **MVP** | Ring buffer + compute + instanced draw + additive blend | Low |
| **v2** | Free list for variable-lifetime particles | Medium |
| **v3** | Indirect draw (GPU writes instance count) | Medium |
| **v4** | Dead list + alive list for high-throughput recycling | High |
| **v5** | Multiple emitter types, texture atlas, color-over-life curves | Medium |
| **v6** | Sub-emitters (particles spawning particles) | High |
| **v7** | Off-screen rendering at reduced resolution for fill-rate savings | Medium |

For a 2D arcade game, **Level MVP or v2 is likely sufficient**. The particle counts involved (hundreds to low thousands) don't require the full Wicked Engine architecture.

### Bevy Integration Points

In Bevy's render architecture, a custom GPU particle system needs:

1. **ECS Components**: `ParticleEmitter`, `ParticleEffect` on entities in the main world
2. **Extract**: Copy emitter state (position, spawn count, params) to render world each frame
3. **Prepare**: Create/resize GPU buffers, write uniform data
4. **Queue**: Submit compute dispatches (spawn + update)
5. **Render**: Instanced draw call in the Transparent2d render phase

The `PipelineCache` handles compute pipeline compilation and caching. Storage buffers are created via `RenderDevice::create_buffer_with_data()` with `STORAGE | VERTEX` usage flags.

For Bevy specifically:
- Use `ShaderStorageBuffer` asset type or raw `Buffer` via `RenderDevice`
- Compute pipelines: `ComputePipelineDescriptor` + `PipelineCache::queue_compute_pipeline()`
- Render phase: add items to `Transparent2d` phase (for 2D)
- Bind groups: derive `AsBindGroup` or create manually via `RenderDevice::create_bind_group()`

---

## Key Takeaways for This Project

1. **Don't over-engineer**. A 2D arcade brick breaker needs ~500-3000 active particles at peak. A ring buffer + single compute shader + instanced quads handles this trivially.

2. **Additive blending is your friend**. It's order-independent (no sorting), looks great for sparks/glows/impacts, and is the cheapest blend mode.

3. **Fill rate matters more than particle count**. Keep particles small. Budget total pixel coverage, not just particle count.

4. **The CPU should only say "spawn N particles here"**. Everything else (lifecycle, death, rendering) lives on the GPU.

5. **Start with the simplest ring buffer architecture**. Graduate to dead lists only if you need variable-lifetime particles with high throughput — unlikely for this game.

6. **bevy_hanabi exists** but is complex. For full control over effects and minimal dependencies, a custom minimal system (100-300 lines of WGSL + Bevy integration) may be cleaner.

7. **Deterministic randomness** (hash-based per-particle seeds) is worth implementing from the start — it's simple and eliminates the need to ship RNG state to the GPU.

---

## Sources

- [Wicked Engine GPU Particle Simulation](https://wickedengine.net/2017/11/gpu-based-particle-simulation/) — Dead/alive list architecture reference
- [Rendering Particles with Compute Shaders (Mike Turitzin)](https://miketuritzin.com/post/rendering-particles-with-compute-shaders/) — Compute-only rendering, atomic splatting
- [GPU Particles (Juan Diego Montoya)](https://juandiegomontoya.github.io/particles.html) — Free list / SSBO architecture
- [GPU Particles (Brian Jiang / DX11)](https://github.com/Brian-Jiang/GPUParticles) — Double buffer flip-flop, 1M particles at 60fps
- [CPU Particle Systems (Alex Tardif)](https://alextardif.com/Particles.html) — CPU architecture, SIMD, when CPU beats GPU
- [NVIDIA GPU Gems 3 Ch.23: Off-Screen Particles](https://developer.nvidia.com/gpugems/gpugems3/part-iv-image-effects/chapter-23-high-speed-screen-particles) — Fill rate optimization
- [Game Particle Effects Guide 2025](https://generalistprogrammer.com/tutorials/game-particle-effects-complete-vfx-programming-guide-2025) — Performance budgets, platform guidelines
- [Vulkan Tutorial: Compute Shader](https://vulkan-tutorial.com/Compute_Shader) — Minimal GPU particle system reference
- [bevy_hanabi (DeepWiki)](https://deepwiki.com/djeedai/bevy_hanabi) — Bevy GPU particle architecture
- [Sprinkles GPU Particle System for Bevy](https://doce.sh/blog/bevy-sprinkles) — MaterialExtension-based approach
- [PopcornFX Particle System Overview](https://wiki.popcornfx.com/index.php/Particle_system_overview) — Batch-oriented architecture
- [Niagara Simulation Stages (HeyYo CG)](https://heyyocg.link/en/ue4-26-niagara-adavanced-simulation-stage-basic/) — Module/stage composition
- [Godot GPU Particles 4.0 Improvements](https://godotengine.org/article/improvements-gpuparticles-godot-40/) — Sub-emitters, attractors
- [Bevy Compute Shaders (Round Egg)](https://druskus.com/posts/round-egg-6/) — Bevy compute pipeline integration
- [sparticles (GitHub)](https://github.com/Norlock/sparticles) — Rust/wgpu GPU particle system
- [GameDev.net: GPU Particle System with Indirect Drawing](https://www.gamedev.net/forums/topic/702160-gpu-particle-system-with-indirect-drawing/) — Indirect dispatch patterns
- [Khronos Forums: Birth/Death without CPU readback](https://community.khronos.org/t/how-to-handle-the-birth-and-death-of-particle-without-reading-from-the-buffer/73291) — Atomic counter patterns
- [2D Particle System (nintervik)](https://nintervik.github.io/2D-Particle-System/) — 2D-specific design
- [LearnOpenGL: 2D Game Particles](https://learnopengl.com/In-Practice/2D-Game/Particles) — Simple 2D particle reference
- [PopcornFX Performance Budget Indicators](https://wiki.popcornfx.com/index.php/Performance_budget_indicators) — Effect complexity classes
