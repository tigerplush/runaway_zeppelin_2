use bevy::{
    core_pipeline::{
        Core3d, Core3dSystems, FullscreenShader,
        prepass::{DepthPrepass, ViewPrepassTextures},
    },
    material::descriptor::BindGroupLayoutDescriptor,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::ExtractResourcePlugin,
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        texture::GpuImage,
        uniform::{ComponentUniforms, DynamicUniformIndex, UniformComponentPlugin},
        view::{ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms},
        *,
    },
};

use crate::{
    camera::CameraMovementIntent,
    fog_of_war::{FOG_WINDOW_HALF_SIZE, FogTexture},
};

const BIND_GROUP_LAYOUT_DESCRIPTOR_LABEL: &str = "fog_of_war_bind_group_layout";
const RENDER_PIPELINE_DESCRIPTOR_LABEL: &str = "fog_of_war_post_process_pipeline";
const POST_PROCESSING_BIND_GROUP_LABEL: &str = "post_process_bind_group";
const POST_PROCESSING_RENDER_PASS_DESCRIPTOR_LABEL: &str = "post_process_pass";
const FOG_OF_WAR_SHADER_PATH: &str = "shader/fog_of_war.wgsl";

#[derive(Component, Default, Clone, Copy, ExtractComponent, Reflect, ShaderType)]
#[reflect(Component)]
struct PostProcessingSettings {
    focal_point: Vec3,
    fog_window_half_size: Vec2,
    yaw: f32,
}

#[derive(Resource)]
struct PostProcessPipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

fn init_post_process_pipeline(
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
    mut commands: Commands,
) {
    let layout = BindGroupLayoutDescriptor::new(
        BIND_GROUP_LAYOUT_DESCRIPTOR_LABEL,
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                texture_2d(TextureSampleType::Float { filterable: true }),
                texture_2d(TextureSampleType::Depth),
                uniform_buffer::<PostProcessingSettings>(true),
                uniform_buffer::<ViewUniform>(true),
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
        label: Some(RENDER_PIPELINE_DESCRIPTOR_LABEL.into()),
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

    commands.insert_resource(PostProcessPipeline {
        layout,
        sampler,
        pipeline_id,
    });
}

#[derive(Default)]
struct PostProcessBindGroupCache {
    cached: Option<(TextureViewId, BindGroup)>,
}

fn post_process_system(
    view: ViewQuery<(
        &ViewTarget,
        &PostProcessingSettings,
        &DynamicUniformIndex<PostProcessingSettings>,
        &ViewPrepassTextures,
        &ViewUniformOffset,
    )>,
    post_process_pipeline: Option<Res<PostProcessPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    settings_uniforms: Res<ComponentUniforms<PostProcessingSettings>>,
    fog_texture: Res<FogTexture>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    view_uniforms: Res<ViewUniforms>,
    mut cache: Local<PostProcessBindGroupCache>,
    mut ctx: RenderContext,
) {
    let Some(post_process_pipeline) = post_process_pipeline else {
        return;
    };

    let Some(fog_gpu_image) = gpu_images.get(&fog_texture.0) else {
        return;
    };

    let (
        view_target,
        _post_process_settings,
        settings_index,
        view_prepass_textures,
        view_uniform_offset,
    ) = view.into_inner();

    let Some(depth_texture) = view_prepass_textures.depth_view() else {
        return;
    };

    let Some(view_uniform) = view_uniforms.uniforms.binding() else {
        return;
    };

    let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
    else {
        return;
    };

    let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
        return;
    };

    let post_process = view_target.post_process_write();

    let bind_group = match &mut cache.cached {
        Some((texture_id, bind_group)) if post_process.source.id() == *texture_id => bind_group,
        cached => {
            let bind_group = ctx.render_device().create_bind_group(
                POST_PROCESSING_BIND_GROUP_LABEL,
                &pipeline_cache.get_bind_group_layout(&post_process_pipeline.layout),
                &BindGroupEntries::sequential((
                    post_process.source,
                    &post_process_pipeline.sampler,
                    &fog_gpu_image.texture_view,
                    depth_texture,
                    settings_binding.clone(),
                    view_uniform,
                )),
            );

            let (_, bind_group) = cached.insert((post_process.source.id(), bind_group));
            bind_group
        }
    };

    let mut render_pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some(POST_PROCESSING_RENDER_PASS_DESCRIPTOR_LABEL),
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

    render_pass.set_pipeline(pipeline);
    render_pass.set_bind_group(
        0,
        bind_group,
        &[settings_index.index(), view_uniform_offset.offset],
    );
    render_pass.draw(0..3, 0..1);
}

fn add_to_camera(
    trigger: On<Add, CameraMovementIntent>,
    intent: Single<&CameraMovementIntent>,
    mut commands: Commands,
) {
    commands.entity(trigger.entity).insert((
        PostProcessingSettings {
            focal_point: intent.focal_point,
            fog_window_half_size: FOG_WINDOW_HALF_SIZE,
            yaw: intent.yaw,
        },
        Msaa::Off,
        DepthPrepass,
    ));
}

fn sync_to_camera(camera: Single<(&mut PostProcessingSettings, &CameraMovementIntent)>) {
    let (mut settings, intent) = camera.into_inner();
    settings.focal_point = intent.focal_point;
}

pub fn plugin(app: &mut App) {
    app.register_type::<PostProcessingSettings>()
        .add_plugins((
            ExtractResourcePlugin::<FogTexture>::default(),
            ExtractComponentPlugin::<PostProcessingSettings>::default(),
            UniformComponentPlugin::<PostProcessingSettings>::default(),
        ))
        .add_systems(Update, sync_to_camera)
        .add_observer(add_to_camera);

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app.add_systems(RenderStartup, init_post_process_pipeline);
    #[cfg(not(debug_assertions))]
    render_app.add_systems(
        Core3d,
        post_process_system.in_set(Core3dSystems::PostProcess),
    );
    #[cfg(debug_assertions)]
    render_app.add_systems(
        Core3d,
        post_process_system
            .in_set(Core3dSystems::PostProcess)
            .before(bevy_egui::render::egui_pass),
    );
}
