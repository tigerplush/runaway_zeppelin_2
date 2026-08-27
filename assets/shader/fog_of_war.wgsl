#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct PostProcessSettings {
    focal_point: vec3<f32>,
    half_size: vec2<f32>,
    yaw: f32,
}
@group(0) @binding(2) var fog_texture: texture_2d<f32>;
@group(0) @binding(3) var depth_texture: texture_depth_2d;
@group(0) @binding(4) var<uniform> settings: PostProcessSettings;
@group(0) @binding(5) var<uniform> view: View;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let depth = textureLoad(depth_texture, vec2<i32>(in.position.xy), 0);
    let ndc = vec3<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0, depth);
    let world_pos_h = view.world_from_clip * vec4<f32>(ndc, 1.0);
    let world_pos = world_pos_h.xyz / world_pos_h.w;
    let relative = world_pos.xz - settings.focal_point.xz;
    let uv = (relative + settings.half_size) / (settings.half_size * 2.0);
    var haze = 1.0;
    var desaturate = 0.0;
    if (all(uv >= vec2(0.0)) && all(uv < vec2(1.0))) {
        let fog = textureSample(fog_texture, texture_sampler, uv);
        haze = fog.r;
        desaturate = fog.g;
    }
    let scene = textureSample(screen_texture, texture_sampler, in.uv);
    let luminance = dot(scene.rgb, vec3<f32>(0.299, 0.587, 0.114));
    var color = mix(scene.rgb, vec3<f32>(luminance), desaturate);
    let haze_color = vec3<f32>(0.1, 0.1, 0.1);
    color = mix(color, haze_color, haze);
    return vec4<f32>(color, scene.a);
}