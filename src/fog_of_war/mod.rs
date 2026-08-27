use std::collections::HashMap;

#[cfg(debug_assertions)]
use bevy::color::palettes::css::{DARK_GRAY, LIGHT_GRAY};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{extract_resource::ExtractResource, render_resource::{Extent3d, TextureDimension, TextureFormat}},
};

#[cfg(debug_assertions)]
use crate::utils::gizmo_traits::DrawHexagon;
use crate::{
    states::AppStates,
    utils::{
        hex::{AxialCoordinates, DEFAULT_HEX_SIZE},
        scale::WorldScale,
    },
    zeppelin::{EnteredCoordinatesMessage, VisibilityRadius, ZeppelinWrapper},
};

mod post_processing;

#[derive(Debug, Reflect)]
enum FogState {
    Visible,
    Revealed,
}

/// Half-extents of the fog data window in world units, shared between
/// `update_texture` (which maps world positions into the texture) and the
/// post-process shader (which maps reconstructed world positions into the
/// same texture) so they can't drift out of sync.
pub(super) const FOG_WINDOW_HALF_SIZE: Vec2 = Vec2::new(50.0, 25.0);

#[derive(Default, Reflect, Resource)]
struct FogOfWar {
    revealed: HashMap<AxialCoordinates, FogState>,
}

fn update_fog_of_war(
    world_scale: Res<WorldScale>,
    mut fog_of_war: ResMut<FogOfWar>,
    mut reader: MessageReader<EnteredCoordinatesMessage>,
    visibility_range: Single<&VisibilityRadius, With<ZeppelinWrapper>>,
) {
    for ev in reader.read() {
        let distance = world_scale.units(visibility_range.0) as isize;
        // reset all tiles to revealed
        fog_of_war
            .revealed
            .iter_mut()
            .for_each(|(_, state)| *state = FogState::Revealed);
        for coordinate in ev.0.within_distance(distance) {
            fog_of_war.revealed.insert(coordinate, FogState::Visible);
        }
    }
}

#[derive(Clone, Reflect, Resource)]
#[reflect(Resource)]
pub(super) struct FogTexture(pub(super) Handle<Image>);

impl FromWorld for FogTexture {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();
        let size = Extent3d {
            width: 256,
            height: 256,
            depth_or_array_layers: 1,
        };
        let pixel = [0, 0, 0, 255];
        let image = Image::new_fill(
            size,
            TextureDimension::D2,
            &pixel,
            TextureFormat::Rgba8UnormSrgb,
            // MAIN_WORLD is required, not just RENDER_WORLD - this texture is
            // repeatedly mutated from the CPU side (clear + set_color_at
            // below), which needs `Image::data` to still be present rather
            // than dropped after the initial GPU upload.
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );
        FogTexture(images.add(image))
    }
}

impl ExtractResource for FogTexture {
    type Source = Self;

    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

fn update_texture(
    reader: MessageReader<EnteredCoordinatesMessage>,
    fog_of_war: Res<FogOfWar>,
    fog_texture: Res<FogTexture>,
    mut images: ResMut<Assets<Image>>,
    zeppelin: Single<&Transform, With<ZeppelinWrapper>>,
) {
    if reader.is_empty() {
        return;
    };

    let Some(mut image) = images.get_mut(&fog_texture.0) else {
        return;
    };

    let zeppelin_world_xz = Vec2::new(zeppelin.translation.x, zeppelin.translation.z);

    let pixel = [0, 0, 0, 255];
    image.clear(&pixel);

    let width = image.width();
    let height = image.height();

    // How many pixels one hex spans, so each hex paints a small block rather
    // than a single dot - blocky for now, not true hex-shaped cells.
    let hex_spacing = DEFAULT_HEX_SIZE.x * 3f32.sqrt();
    let half_block_x =
        ((hex_spacing / (FOG_WINDOW_HALF_SIZE.x * 2.0) * width as f32) / 2.0).ceil() as i32;
    let half_block_y =
        ((hex_spacing / (FOG_WINDOW_HALF_SIZE.y * 2.0) * height as f32) / 2.0).ceil() as i32;

    for (coordinates, state) in fog_of_war.revealed.iter() {
        let world = coordinates.as_world_coordinates(DEFAULT_HEX_SIZE);
        let relative_x = world.x - zeppelin_world_xz.x;
        let relative_z = world.z - zeppelin_world_xz.y;

        // Normalize into [0, 1) across the current window (centered on the
        // zeppelin); skip anything that's fallen outside it (e.g. explored
        // long ago, far from here).
        let normalized_x = (relative_x + FOG_WINDOW_HALF_SIZE.x) / (FOG_WINDOW_HALF_SIZE.x * 2.0);
        let normalized_z = (relative_z + FOG_WINDOW_HALF_SIZE.y) / (FOG_WINDOW_HALF_SIZE.y * 2.0);
        if !(0.0..1.0).contains(&normalized_x) || !(0.0..1.0).contains(&normalized_z) {
            continue;
        }

        let center_x = (normalized_x * width as f32) as i32;
        let center_y = (normalized_z * height as f32) as i32;

        let color = match state {
            // Fully clear - nothing hidden where we can currently see.
            FogState::Visible => Color::NONE,
            // Dimmed, not fully opaque - remembered, just not in current sight.
            FogState::Revealed => Color::srgba(0.1, 0.1, 0.1, 0.6),
        };

        for dx in -half_block_x..=half_block_x {
            for dy in -half_block_y..=half_block_y {
                let x = center_x + dx;
                let y = center_y + dy;
                if x < 0 || y < 0 || x as u32 >= width || y as u32 >= height {
                    continue;
                }
                let _ = image.set_color_at(x as u32, y as u32, color);
            }
        }
    }
}

#[cfg(debug_assertions)]
fn debug_fog_of_war(mut gizmos: Gizmos, fog_of_war: Res<FogOfWar>) {
    for (coordinate, state) in fog_of_war.revealed.iter() {
        let color = match state {
            FogState::Revealed => DARK_GRAY.with_alpha(0.5),
            FogState::Visible => LIGHT_GRAY.with_alpha(0.5),
        };
        gizmos.draw_hexagon(
            coordinate.as_world_coordinates(DEFAULT_HEX_SIZE),
            DEFAULT_HEX_SIZE,
            color,
        );
    }
}

pub fn plugin(app: &mut App) {
    app.register_type::<FogOfWar>()
        .register_type::<FogTexture>()
        .init_resource::<FogOfWar>()
        .init_resource::<FogTexture>()
        .add_plugins(post_processing::plugin)
        .add_systems(
            Update,
            (update_fog_of_war, update_texture)
                .chain()
                .run_if(in_state(AppStates::InGame)),
        );

    #[cfg(debug_assertions)]
    app.add_systems(Update, debug_fog_of_war);
}
