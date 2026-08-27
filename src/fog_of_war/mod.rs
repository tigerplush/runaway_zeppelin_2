use std::collections::HashMap;

#[cfg(debug_assertions)]
use bevy::color::palettes::css::{DARK_GRAY, LIGHT_GRAY};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

#[cfg(debug_assertions)]
use crate::utils::gizmo_traits::DrawHexagon;
use crate::{
    camera::CameraMovementIntent,
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
pub(super) const FOG_WINDOW_HALF_SIZE: Vec2 = Vec2::new(16.0, 9.0);

const TEXTURE_EXTENT: Extent3d = Extent3d {
    width: 640,
    height: 360,
    depth_or_array_layers: 1,
};

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

        let pixel = [255, 0, 0, 0];
        let image = Image::new_fill(
            TEXTURE_EXTENT,
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
    fog_of_war: Res<FogOfWar>,
    fog_texture: Res<FogTexture>,
    mut images: ResMut<Assets<Image>>,
    camera: Single<&CameraMovementIntent>,
) {
    let Some(mut image) = images.get_mut(&fog_texture.0) else {
        return;
    };

    let width = image.width();
    let height = image.height();
    for y in 0..height {
        for x in 0..width {
            let normalized_x = (x as f32 + 0.5) / width as f32;
            let normalized_z = (y as f32 + 0.5) / height as f32;
            let relative = Vec2::new(
                normalized_x * FOG_WINDOW_HALF_SIZE.x * 2.0 - FOG_WINDOW_HALF_SIZE.x,
                normalized_z * FOG_WINDOW_HALF_SIZE.y * 2.0 - FOG_WINDOW_HALF_SIZE.y,
            );
            let world = Vec3::new(
                relative.x + camera.focal_point.x,
                0.0,
                relative.y + camera.focal_point.z,
            );
            let hex = AxialCoordinates::from_world_coordinates(world, DEFAULT_HEX_SIZE);

            let color = match fog_of_war.revealed.get(&hex) {
                Some(FogState::Visible) => Color::srgba(0.0, 0.0, 0.0, 0.0),
                Some(FogState::Revealed) => Color::srgba(0.0, 1.0, 0.0, 0.0),
                None => Color::srgba(1.0, 0.0, 0.0, 0.0),
            };
            _ = image.set_color_at(x, y, color);
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
