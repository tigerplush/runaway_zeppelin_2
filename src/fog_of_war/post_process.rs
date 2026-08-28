//! Debug-only overlay that visualizes the current depth prepass texture on
//! screen, toggled via the [`DepthOverlay`] resource (edit `enabled` from the
//! world inspector - no keybinding). Entirely absent in release builds.

use bevy::{
    core_pipeline::{
        FullscreenShader,
        prepass::{DepthPrepass, ViewPrepassTextures},
    },
    material::descriptor::BindGroupLayoutDescriptor,
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
        texture::GpuImage,
        view::{ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms},
    },
};

use crate::fog_of_war::explored::{FogExploredTexture, FogParams};

const BIND_GROUP_LAYOUT_LABEL: &str = "fog_of_war_bind_group_layout";
const BIND_GROUP_LABEL: &str = "fog_of_war_bind_group";
const RENDER_PIPELINE_LABEL: &str = "fog_of_war_pipeline";
const RENDER_PASS_LABEL: &str = "fog_of_war_pass";
const FOG_OF_WAR_SHADER_PATH: &str = "shader/fog_of_war.wgsl";

/// Ensures any `Camera3d` has what the overlay needs to read a depth buffer,
/// regardless of whether some other plugin (e.g. fog of war) already did.
fn ensure_camera_prepass(trigger: On<Add, Camera3d>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert((Msaa::Off, DepthPrepass));
}

#[derive(Resource)]
struct FogOfWarPipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

fn init_fog_of_war_pipeline(
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
    mut commands: Commands,
) {
    let layout = BindGroupLayoutDescriptor::new(
        BIND_GROUP_LAYOUT_LABEL,
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Depth),
                texture_2d(TextureSampleType::Float { filterable: true }),
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<ViewUniform>(true),
                uniform_buffer::<FogParams>(false),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor {
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..default()
    });

    let shader = asset_server.load(FOG_OF_WAR_SHADER_PATH);
    let vertex_state = fullscreen_shader.to_vertex_state();

    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some(RENDER_PIPELINE_LABEL.into()),
        layout: vec![layout.clone()],
        vertex: vertex_state,
        fragment: Some(FragmentState {
            shader,
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::Rgba8UnormSrgb,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(FogOfWarPipeline {
        layout,
        sampler,
        pipeline_id,
    });
}

#[derive(Default)]
struct FogOfWarBindGroupCache {
    cached: Option<(TextureViewId, BindGroup)>,
}

#[derive(Default, Resource)]
struct UniformBufResource(UniformBuffer<FogParams>);

fn sync_params(
    fog_params: Res<FogParams>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut buffer: ResMut<UniformBufResource>,
) {
    buffer.0.set(fog_params.clone());
    buffer.0.write_buffer(&render_device, &render_queue);
}

fn fog_of_war_system(
    view: ViewQuery<(&ViewTarget, &ViewPrepassTextures, &ViewUniformOffset)>,
    pipeline: Option<Res<FogOfWarPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    fog_texture: Res<FogExploredTexture>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    view_uniforms: Res<ViewUniforms>,
    settings_uniforms: Res<UniformBufResource>,
    mut cache: Local<FogOfWarBindGroupCache>,
    mut ctx: RenderContext,
) {
    let Some(fog_of_war_pipeline) = pipeline else {
        return;
    };

    let Some(fog_gpu_image) = gpu_images.get(&fog_texture.image) else {
        return;
    };

    let Some(render_pipeline) = pipeline_cache.get_render_pipeline(fog_of_war_pipeline.pipeline_id)
    else {
        return;
    };

    let (view_target, prepass_textures, view_uniform_offset) = view.into_inner();
    let Some(depth_view) = prepass_textures.depth_view() else {
        return;
    };
    let Some(view_uniform) = view_uniforms.uniforms.binding() else {
        return;
    };
    let Some(settings_binding) = settings_uniforms.0.binding() else {
        return;
    };

    let post_process = view_target.post_process_write();

    let bind_group = match &mut cache.cached {
        Some((texture_id, bind_group)) if post_process.source.id() == *texture_id => bind_group,
        cached => {
            let bind_group = ctx.render_device().create_bind_group(
                BIND_GROUP_LABEL,
                &pipeline_cache.get_bind_group_layout(&fog_of_war_pipeline.layout),
                &BindGroupEntries::sequential((
                    depth_view,
                    &fog_gpu_image.texture_view,
                    post_process.source,
                    &fog_of_war_pipeline.sampler,
                    view_uniform,
                    settings_binding.clone(),
                )),
            );
            let (_, bind_group) = cached.insert((post_process.source.id(), bind_group));
            bind_group
        }
    };

    let mut render_pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some(RENDER_PASS_LABEL),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

    render_pass.set_pipeline(render_pipeline);
    render_pass.set_bind_group(0, bind_group, &[view_uniform_offset.offset]);
    render_pass.draw(0..3, 0..1);
}

pub fn plugin(app: &mut bevy::app::App) {
    app.add_observer(ensure_camera_prepass);

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app
        .init_resource::<UniformBufResource>()
        .add_systems(RenderStartup, init_fog_of_war_pipeline)
        .add_systems(Render, sync_params.in_set(RenderSystems::Prepare))
        .add_systems(
            bevy::core_pipeline::Core3d,
            fog_of_war_system
                .in_set(bevy::core_pipeline::Core3dSystems::PostProcess)
                .before(bevy_egui::render::egui_pass),
        );
}
