use std::{collections::HashSet, ops::Sub};

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::{utils::scale::WorldScale, zeppelin::VisibilityRadius};

/// Fog cell size in world units
pub const FOG_CELL_SIZE: f32 = 1.0;

/// Half-width of the fog texture window, in cells. Must stay comfortably
/// larger than the zeppelin's `VisibilityRadius` (in cells) or the reveal
/// circle will exceed the window on the very first frame. At the default
/// `WorldScale`, `VisibilityRadius` is ~22 cells, so 64 leaves headroom.
pub const WINDOW_HALF_SIZE: i32 = 64;

// Kept in lockstep with `WINDOW_HALF_SIZE` so the window and the texture it's
// written into can never drift out of sync.
const FOG_TEXTURE_EXTENT: Extent3d = Extent3d {
    width: (WINDOW_HALF_SIZE * 2) as u32,
    height: (WINDOW_HALF_SIZE * 2) as u32,
    depth_or_array_layers: 1,
};

#[derive(Clone, Copy, Eq, Hash, PartialEq, Reflect)]
pub struct FogCell {
    x: i32,
    z: i32,
}

impl FogCell {
    const ZERO: Self = Self { x: 0, z: 0 };
}

impl Sub<FogCell> for FogCell {
    type Output = Self;
    fn sub(self, rhs: FogCell) -> Self::Output {
        FogCell {
            x: self.x - rhs.x,
            z: self.z - rhs.z,
        }
    }
}

#[derive(Default, Reflect, Resource)]
#[reflect(Resource)]
pub struct FogExploredMap(pub HashSet<FogCell>);

/// Cells inserted into `FogExploredMap` this frame, so `sync_explored_texture`
/// can update just those texels instead of rescanning the whole map every
/// frame. Drained each frame regardless of which path it takes.
#[derive(Default, Reflect, Resource)]
#[reflect(Resource)]
struct NewlyRevealedCells(Vec<FogCell>);

fn reveal_around_zeppelin(
    world_scale: Res<WorldScale>,
    mut fog_map: ResMut<FogExploredMap>,
    mut newly_revealed: ResMut<NewlyRevealedCells>,
    zeppelin: Single<(&Transform, &VisibilityRadius)>,
) {
    let (transform, visibility) = zeppelin.into_inner();

    let visibility_radius = world_scale.units(visibility.0);
    let center = transform.translation;
    let radius_cells = (visibility_radius / FOG_CELL_SIZE).ceil() as i32;
    let center_cell = FogCell {
        x: (center.x / FOG_CELL_SIZE).floor() as i32,
        z: (center.z / FOG_CELL_SIZE).floor() as i32,
    };

    for dz in -radius_cells..=radius_cells {
        for dx in -radius_cells..=radius_cells {
            let cell = FogCell {
                x: center_cell.x + dx,
                z: center_cell.z + dz,
            };
            let cell_world = Vec2::new(dx as f32 * FOG_CELL_SIZE, dz as f32 * FOG_CELL_SIZE);
            if cell_world.length() <= visibility_radius && fog_map.0.insert(cell) {
                newly_revealed.0.push(cell);
            }
        }
    }
}

#[derive(Reflect, Resource)]
#[reflect(Resource)]
struct FogExploredTexture {
    image: Handle<Image>,
    center: FogCell,
}

impl FromWorld for FogExploredTexture {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();

        let pixel = [0, 0, 0, 0];
        let image = Image::new_fill(
            FOG_TEXTURE_EXTENT,
            TextureDimension::D2,
            &pixel,
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );
        FogExploredTexture {
            image: images.add(image),
            center: FogCell::ZERO,
        }
    }
}

const EXPLORED_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);

fn sync_explored_texture(
    fog_map: Res<FogExploredMap>,
    mut fog_texture: ResMut<FogExploredTexture>,
    mut newly_revealed: ResMut<NewlyRevealedCells>,
    mut images: ResMut<Assets<Image>>,
    zeppelin: Single<(&Transform, &VisibilityRadius)>,
) {
    let Some(mut image) = images.get_mut(&fog_texture.image) else {
        newly_revealed.0.clear();
        return;
    };

    let (transform, _) = zeppelin.into_inner();
    let zeppelin_cell = FogCell {
        x: (transform.translation.x / FOG_CELL_SIZE).floor() as i32,
        z: (transform.translation.z / FOG_CELL_SIZE).floor() as i32,
    };

    let margin = WINDOW_HALF_SIZE / 4;
    let rel = zeppelin_cell - fog_texture.center;
    let needs_recenter =
        rel.x.abs() > WINDOW_HALF_SIZE - margin || rel.z.abs() > WINDOW_HALF_SIZE - margin;

    if needs_recenter {
        // Recompute the window from the *new* center before rebuilding -
        // using the stale window here would write texels at the wrong
        // offset for every cell.
        fog_texture.center = zeppelin_cell;
        let window_min = FogCell {
            x: fog_texture.center.x - WINDOW_HALF_SIZE,
            z: fog_texture.center.z - WINDOW_HALF_SIZE,
        };
        let window_max = FogCell {
            x: fog_texture.center.x + WINDOW_HALF_SIZE,
            z: fog_texture.center.z + WINDOW_HALF_SIZE,
        };

        image.clear(&[0, 0, 0, 0]);

        for cell in &fog_map.0 {
            if cell.x >= window_min.x
                && cell.x < window_max.x
                && cell.z >= window_min.z
                && cell.z < window_max.z
            {
                let tex_x = (cell.x - window_min.x) as u32;
                let tex_z = (cell.z - window_min.z) as u32;
                _ = image.set_color_at(tex_x, tex_z, EXPLORED_COLOR);
            }
        }
    } else {
        // Steady state: the window didn't move, so only the cells revealed
        // this frame (already known to fall inside it) need writing.
        let window_min = FogCell {
            x: fog_texture.center.x - WINDOW_HALF_SIZE,
            z: fog_texture.center.z - WINDOW_HALF_SIZE,
        };
        for cell in &newly_revealed.0 {
            let tex_x = (cell.x - window_min.x) as u32;
            let tex_z = (cell.z - window_min.z) as u32;
            _ = image.set_color_at(tex_x, tex_z, EXPLORED_COLOR);
        }
    }

    newly_revealed.0.clear();
}

#[derive(Default, Reflect, Resource)]
#[reflect(Resource)]
struct FogParams {
    zeppelin_world: Vec2,
    window_origin: Vec2,
    window_size: Vec2,
    elapsed_secs: f32,
    visibility_radius: f32,
}

fn sync_params(
    time: Res<Time<Real>>,
    world_scale: Res<WorldScale>,
    fog_texture: Res<FogExploredTexture>,
    mut fog_params: ResMut<FogParams>,
    zeppelin: Single<(&Transform, &VisibilityRadius)>,
) {
    let (transform, visibility) = zeppelin.into_inner();
    fog_params.zeppelin_world = transform.translation.xz();
    fog_params.elapsed_secs = time.elapsed_secs();
    fog_params.visibility_radius = world_scale.units(visibility.0);
    fog_params.window_origin = Vec2::new(
        (fog_texture.center.x - WINDOW_HALF_SIZE) as f32 * FOG_CELL_SIZE,
        (fog_texture.center.z - WINDOW_HALF_SIZE) as f32 * FOG_CELL_SIZE,
    );
    fog_params.window_size = Vec2::splat(WINDOW_HALF_SIZE as f32 * 2.0 * FOG_CELL_SIZE);
}

#[cfg(debug_assertions)]
fn debug_explored_map(mut gizmos: Gizmos, fog_map: ResMut<FogExploredMap>) {
    for cell in &fog_map.0 {
        use std::f32::consts::PI;

        use bevy::color::palettes::css::WHITE;

        gizmos.rect(
            Isometry3d::new(
                Vec3::new(
                    cell.x as f32 * FOG_CELL_SIZE,
                    0_f32,
                    cell.z as f32 * FOG_CELL_SIZE,
                ),
                Quat::from_rotation_x(-PI / 2.0),
            ),
            Vec2::splat(FOG_CELL_SIZE),
            WHITE,
        );
    }
}

pub fn plugin(app: &mut App) {
    app.register_type::<FogExploredMap>()
        .register_type::<FogExploredTexture>()
        .register_type::<NewlyRevealedCells>()
        .register_type::<FogParams>()
        .init_resource::<FogExploredMap>()
        .init_resource::<FogExploredTexture>()
        .init_resource::<NewlyRevealedCells>()
        .init_resource::<FogParams>()
        .add_systems(
            Update,
            (reveal_around_zeppelin, sync_explored_texture, sync_params).chain(),
        );

    #[cfg(debug_assertions)]
    app.add_systems(Update, debug_explored_map);
}
