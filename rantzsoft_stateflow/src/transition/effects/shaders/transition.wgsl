#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct TransitionEffect {
    color: vec4<f32>,
    direction: vec4<f32>,
    effect_type: u32,
    progress: f32,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: TransitionEffect;

// Procedural hash for dissolve noise
fn hash(p: vec2<f32>) -> f32 {
    var p2 = fract(p * vec2<f32>(443.8975, 397.2973));
    p2 += dot(p2, p2 + 19.19);
    return fract(p2.x * p2.y);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let screen = textureSample(screen_texture, texture_sampler, in.uv);

    switch settings.effect_type {
        // Fade: lerp toward target color
        case 1u: {
            return vec4<f32>(mix(screen.rgb, settings.color.rgb, settings.progress), 1.0);
        }

        // Dissolve: noise threshold
        case 2u: {
            let screen_size = vec2<f32>(textureDimensions(screen_texture));
            let pixel = in.uv * screen_size;
            let noise = hash(floor(pixel));
            if noise < settings.progress {
                return settings.color;
            }
            return screen;
        }

        // Pixelate: snap UV to grid
        case 3u: {
            let min_blocks = 4.0;
            let max_blocks = 256.0;
            let blocks = mix(max_blocks, min_blocks, settings.progress);
            let snapped_uv = floor(in.uv * blocks) / blocks + 0.5 / blocks;
            return textureSample(screen_texture, texture_sampler, snapped_uv);
        }

        // Iris: circle mask from center
        case 4u: {
            let centered = in.uv - vec2<f32>(0.5, 0.5);
            let dist = length(centered);
            // Max radius to cover corners: sqrt(0.5^2 + 0.5^2) ≈ 0.707
            let radius = (1.0 - settings.progress) * 0.75;
            let alpha = smoothstep(radius - 0.02, radius + 0.02, dist);
            return vec4<f32>(mix(screen.rgb, settings.color.rgb, alpha), 1.0);
        }

        // Wipe: directional threshold
        case 5u: {
            let dir = settings.direction.xy;
            // Map UV from [0,1] to [-0.5, 0.5] centered
            let centered = in.uv - vec2<f32>(0.5, 0.5);
            let proj = dot(centered, dir) + 0.5; // remap to [0, 1]
            let edge = smoothstep(settings.progress - 0.02, settings.progress + 0.02, proj);
            return vec4<f32>(mix(screen.rgb, settings.color.rgb, 1.0 - edge), 1.0);
        }

        // Slide: UV offset + solid fill
        case 6u: {
            let offset = settings.direction.xy * settings.progress;
            let shifted_uv = in.uv + offset;
            // If shifted UV is outside [0,1], show target color
            if shifted_uv.x < 0.0 || shifted_uv.x > 1.0 || shifted_uv.y < 0.0 || shifted_uv.y > 1.0 {
                return settings.color;
            }
            return textureSample(screen_texture, texture_sampler, shifted_uv);
        }

        // None / default: passthrough
        default: {
            return screen;
        }
    }
}
