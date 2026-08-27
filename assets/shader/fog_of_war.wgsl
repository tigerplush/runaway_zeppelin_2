#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct PostProcessSettings {
    focal_point: vec3<f32>,
    half_size: vec2<f32>,
    yaw: f32,
    time: f32,
}
@group(0) @binding(2) var fog_texture: texture_2d<f32>;
@group(0) @binding(3) var depth_texture: texture_depth_2d;
@group(0) @binding(4) var<uniform> settings: PostProcessSettings;
@group(0) @binding(5) var<uniform> view: View;

fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {

    let depth = textureLoad(depth_texture, vec2<i32>(in.position.xy), 0);
    let ndc = vec3<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0, depth);
    let world_pos_h = view.world_from_clip * vec4<f32>(ndc, 1.0);
    let world_pos = world_pos_h.xyz / world_pos_h.w;
    
    let warp = vec2<f32>(
        noise(world_pos.xz * 0.15 + vec2<f32>(settings.time * 0.05, 0.0)),
        noise(world_pos.xz * 0.15 + vec2<f32>(0.0, settings.time * 0.05)),
    ) - 0.5;

    let warped_xz = world_pos.xz + warp * 1.5;

    let c = cos(settings.yaw);
    let s = sin(settings.yaw);
    let relative = warped_xz - settings.focal_point.xz;
    let local = vec2<f32>(
        relative.x * c - relative.y * s,
        relative.x * s + relative.y * c,
    );
    let uv = (local + settings.half_size) / (settings.half_size * 2.0);

    var haze = 1.0;
    var desaturate = 0.0;
    if (all(uv >= vec2(0.0)) && all(uv < vec2(1.0))) {
        let fog = textureSample(fog_texture, texture_sampler, uv);
        haze = smoothstep(0.25, 0.75, fog.r);
        desaturate = smoothstep(0.25, 0.75, fog.g);
    }

    let shimmer = noise(world_pos.xz * 0.4 + settings.time * 0.15) * 0.12 - 0.06;
    haze = clamp(haze + shimmer, 0.0, 1.0);

    let fog_layer_height = 4.0;   // world units — tune against your hex/camera scale
    let height_falloff = 1.0 - smoothstep(0.0, fog_layer_height, world_pos.y);
    haze = haze * height_falloff;

    let scene = textureSample(screen_texture, texture_sampler, in.uv);
    let luminance = dot(scene.rgb, vec3<f32>(0.299, 0.587, 0.114));
    var color = mix(scene.rgb, vec3<f32>(luminance), desaturate);
    let haze_color = vec3<f32>(0.1, 0.1, 0.1);
    color = mix(color, haze_color, haze);
    return vec4<f32>(color, scene.a);
}