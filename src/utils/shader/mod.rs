//! Debug-only overlay that visualizes the current depth prepass texture on
//! screen, toggled via the [`DepthOverlay`] resource (edit `enabled` from the
//! world inspector - no keybinding). Entirely absent in release builds.

use bevy::{
    core_pipeline::{FullscreenShader, prepass::{DepthPrepass, ViewPrepassTextures}}, material::descriptor::BindGroupLayoutDescriptor, prelude::*, render::{
        RenderApp, RenderStartup, extract_resource::{ExtractResource, ExtractResourcePlugin}, render_resource::{binding_types::texture_2d, *}, renderer::{RenderContext, ViewQuery}, view::ViewTarget,
    },
};

pub fn plugin(app: &mut bevy::app::App) {
    app.register_type::<DepthOverlay>()
        .init_resource::<DepthOverlay>()
        .add_plugins(ExtractResourcePlugin::<DepthOverlay>::default())
        .add_observer(ensure_camera_prepass);

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app
        .add_systems(RenderStartup, init_depth_overlay_pipeline)
        .add_systems(
            bevy::core_pipeline::Core3d,
            depth_overlay_system
                .in_set(bevy::core_pipeline::Core3dSystems::PostProcess)
                .before(bevy_egui::render::egui_pass),
        );
}


const BIND_GROUP_LAYOUT_LABEL: &str = "depth_overlay_bind_group_layout";
const BIND_GROUP_LABEL: &str = "depth_overlay_bind_group";
const RENDER_PIPELINE_LABEL: &str = "depth_overlay_pipeline";
const RENDER_PASS_LABEL: &str = "depth_overlay_pass";
const DEPTH_OVERLAY_SHADER_PATH: &str = "shader/depth_overlay.wgsl";

/// Toggle this from the world inspector to show/hide the depth overlay.
#[derive(Clone, Default, Reflect, Resource)]
#[reflect(Resource)]
struct DepthOverlay {
    enabled: bool,
}

impl ExtractResource for DepthOverlay {
    type Source = Self;

    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

/// Ensures any `Camera3d` has what the overlay needs to read a depth buffer,
/// regardless of whether some other plugin (e.g. fog of war) already did.
fn ensure_camera_prepass(trigger: On<Add, Camera3d>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert((Msaa::Off, DepthPrepass));
}

#[derive(Resource)]
struct DepthOverlayPipeline {
    layout: BindGroupLayoutDescriptor,
    pipeline_id: CachedRenderPipelineId,
}

fn init_depth_overlay_pipeline(
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
    mut commands: Commands,
) {
    let layout = BindGroupLayoutDescriptor::new(
        BIND_GROUP_LAYOUT_LABEL,
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (texture_2d(TextureSampleType::Depth),),
        ),
    );

    let shader = asset_server.load(DEPTH_OVERLAY_SHADER_PATH);
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

    commands.insert_resource(DepthOverlayPipeline { layout, pipeline_id });
}

#[derive(Default)]
struct DepthOverlayBindGroupCache {
    cached: Option<(TextureViewId, BindGroup)>,
}

fn depth_overlay_system(
    enabled: Option<Res<DepthOverlay>>,
    view: ViewQuery<(&ViewTarget, &ViewPrepassTextures)>,
    pipeline: Option<Res<DepthOverlayPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    mut cache: Local<DepthOverlayBindGroupCache>,
    mut ctx: RenderContext,
) {
    // Off by default, and skipping the whole pass (rather than branching in
    // the shader) means the view target is left untouched when disabled -
    // no uniform buffer needed just to carry a single bool.
    let Some(enabled) = enabled else {
        return;
    };
    if !enabled.enabled {
        return;
    }

    let Some(pipeline) = pipeline else {
        return;
    };
    let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline.pipeline_id) else {
        return;
    };

    let (view_target, prepass_textures) = view.into_inner();
    let Some(depth_view) = prepass_textures.depth_view() else {
        return;
    };

    let post_process = view_target.post_process_write();

    let bind_group = match &mut cache.cached {
        Some((texture_id, bind_group)) if post_process.source.id() == *texture_id => bind_group,
        cached => {
            let bind_group = ctx.render_device().create_bind_group(
                BIND_GROUP_LABEL,
                &pipeline_cache.get_bind_group_layout(&pipeline.layout),
                &BindGroupEntries::sequential((depth_view,)),
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
    render_pass.set_bind_group(0, bind_group, &[]);
    render_pass.draw(0..3, 0..1);
}
