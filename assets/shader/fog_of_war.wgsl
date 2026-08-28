#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View

@group(0) @binding(0) var depth_texture: texture_depth_2d;
@group(0) @binding(1) var fog_explored_texture: texture_2d<f32>;
@group(0) @binding(2) var scene_color_texture: texture_2d<f32>;
@group(0) @binding(3) var texture_sampler: sampler;
@group(0) @binding(4) var<uniform> view: View;
struct FogParams {
    zeppelin_world: vec2<f32>,
    window_origin: vec2<f32>,
    window_size: vec2<f32>,
    elapsed_secs: f32,
    visibility_radius: f32,
}
@group(0) @binding(5) var<uniform> params: FogParams;

fn get_world_pos(in: FullscreenVertexOutput, depth: f32) -> vec3<f32> {
    let ndc = vec3<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0, depth);
    let world_pos_h = view.world_from_clip * vec4<f32>(ndc, 1.0);
    let world_pos = world_pos_h.xyz / world_pos_h.w;
    return world_pos;
}

fn world_to_window_uv(world: vec2<f32>) -> vec2<f32> {
    return (world - params.window_origin) / params.window_size;
}

fn hash3(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn value_noise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let c000 = hash3(i + vec3<f32>(0.0, 0.0, 0.0));
    let c100 = hash3(i + vec3<f32>(1.0, 0.0, 0.0));
    let c010 = hash3(i + vec3<f32>(0.0, 1.0, 0.0));
    let c110 = hash3(i + vec3<f32>(1.0, 1.0, 0.0));
    let c001 = hash3(i + vec3<f32>(0.0, 0.0, 1.0));
    let c101 = hash3(i + vec3<f32>(1.0, 0.0, 1.0));
    let c011 = hash3(i + vec3<f32>(0.0, 1.0, 1.0));
    let c111 = hash3(i + vec3<f32>(1.0, 1.0, 1.0));

    let x00 = mix(c000, c100, u.x);
    let x10 = mix(c010, c110, u.x);
    let x01 = mix(c001, c101, u.x);
    let x11 = mix(c011, c111, u.x);

    let y0 = mix(x00, x10, u.y);
    let y1 = mix(x01, x11, u.y);

    return mix(y0, y1, u.z);
}

fn fbm(p: vec3<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var pos = p;
    for (var i = 0; i < 4; i = i + 1) {
        value += amplitude * value_noise(pos);
        pos *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

// Density at a world-space point inside the fog slab. Clouds scroll away
// from the zeppelin, faster near the reveal boundary, plus a slow ambient
// drift everywhere so distant cloud still feels alive.
fn cloud_density(sample_pos: vec3<f32>) -> f32 {
    let to_sample = sample_pos.xz - params.zeppelin_world;
    let dist_to_zeppelin = length(to_sample);
    let away_dir = select(vec2<f32>(1.0, 0.0), to_sample / dist_to_zeppelin, dist_to_zeppelin > 0.001);

    let edge_band = 15.0;
    let edge_proximity = 1.0 - smoothstep(params.visibility_radius, params.visibility_radius + edge_band, dist_to_zeppelin);
    let roll_speed = mix(0.4, 5.0, edge_proximity);
    let roll_offset = vec3<f32>(away_dir.x, 0.0, away_dir.y) * params.elapsed_secs * roll_speed;

    let noise_pos = sample_pos * 0.12 - roll_offset;
    return fbm(noise_pos);
}

// Depth-aware raymarch through a horizontal fog slab. `max_distance` is the
// distance to the actual scene surface along the ray (from the depth
// buffer) - the march never accumulates density past it, so solid geometry
// correctly occludes/clips the cloud volume, and a fragment above the slab
// (e.g. a roof poking through) naturally gets zero density with no
// per-building special-casing needed.
fn raymarch_slab(ray_origin: vec3<f32>, ray_dir: vec3<f32>, max_distance: f32) -> vec4<f32> {
    let fog_bottom = -2.0;
    let fog_top = 12.0;

    var t_enter = 0.0;
    var t_exit = max_distance;

    if abs(ray_dir.y) > 0.0001 {
        let t_a = (fog_bottom - ray_origin.y) / ray_dir.y;
        let t_b = (fog_top - ray_origin.y) / ray_dir.y;
        t_enter = max(t_enter, min(t_a, t_b));
        t_exit = min(t_exit, max(t_a, t_b));
    } else if ray_origin.y < fog_bottom || ray_origin.y > fog_top {
        return vec4<f32>(0.0);
    }

    t_exit = min(t_exit, max_distance);

    if t_enter >= t_exit {
        return vec4<f32>(0.0);
    }

    let step_count = 12;
    let step_size = (t_exit - t_enter) / f32(step_count);

    var accum_color = vec3<f32>(0.0);
    var accum_alpha = 0.0;
    let cloud_tint = vec3<f32>(0.75, 0.77, 0.82);

    for (var i = 0; i < step_count; i = i + 1) {
        if accum_alpha > 0.98 {
            break;
        }
        let t = t_enter + step_size * (f32(i) + 0.5);
        let sample_pos = ray_origin + ray_dir * t;

        let density = cloud_density(sample_pos);
        let step_alpha = clamp(density * step_size * 0.2, 0.0, 1.0);

        accum_color += (1.0 - accum_alpha) * step_alpha * cloud_tint;
        accum_alpha += (1.0 - accum_alpha) * step_alpha;
    }

    return vec4<f32>(accum_color, accum_alpha);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let depth = textureLoad(depth_texture, vec2<i32>(in.position.xy), 0);
    let world_pos = get_world_pos(in, depth);
    let scene_color = textureLoad(scene_color_texture, vec2<i32>(in.position.xy), 0);

    let edge_softness = 1.0;
    let visible = smoothstep(params.visibility_radius, params.visibility_radius - edge_softness, distance(world_pos.xz, params.zeppelin_world));

    let explored = textureSample(fog_explored_texture, texture_sampler, world_to_window_uv(world_pos.xz)).r;

    let luminance = dot(scene_color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    let greyed_color = vec4<f32>(vec3<f32>(luminance) * 0.5, scene_color.a);

    let base_cloud_color = vec3<f32>(0.05, 0.05, 0.08);
    var cloud_color = vec4<f32>(base_cloud_color, 1.0);

    // The raymarch is only worth paying for on genuinely unexplored pixels -
    // explored/visible ones never show cloud at all (see the mixes below).
    if explored < 0.5 {
        let ray_origin = view.world_position;
        let max_distance = distance(ray_origin, world_pos);
        let ray_dir = normalize(world_pos - ray_origin);
        let cloud = raymarch_slab(ray_origin, ray_dir, max_distance);
        cloud_color = vec4<f32>(mix(base_cloud_color, cloud.rgb, cloud.a), 1.0);
    }

    var out_color = mix(cloud_color, greyed_color, explored);
    out_color = mix(out_color, scene_color, visible);

    return out_color;
}
