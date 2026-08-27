#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var depth_texture: texture_depth_2d;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let depth = textureLoad(depth_texture, vec2<i32>(in.position.xy), 0);

    // Bevy's depth prepass is reverse-Z: 1.0 = near plane, 0.0 = far plane
    // (also what an empty/background pixel clears to). Raw depth values
    // clump very close to 0 for most on-screen geometry, so raise to a
    // fractional power to spread the visible range out instead of most of
    // the screen just looking black.
    let brightness = pow(depth, 0.2);
    return vec4<f32>(vec3<f32>(brightness), 1.0);
}
