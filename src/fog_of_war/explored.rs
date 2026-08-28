use std::collections::HashSet;

use bevy::prelude::*;

use crate::{utils::scale::WorldScale, zeppelin::VisibilityRadius};

/// Fog cell size in world units
pub const FOG_CELL_SIZE: f32 = 1.0;

#[derive(Eq, Hash, PartialEq, Reflect)]
pub struct FogCell {
    x: i32,
    z: i32,
}

#[derive(Default, Reflect, Resource)]
#[reflect(Resource)]
pub struct FogExploredMap(pub HashSet<FogCell>);

fn reveal_around_zeppelin(
    world_scale: Res<WorldScale>,
    mut fog_map: ResMut<FogExploredMap>,
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
            let cell_world = Vec3::new(
                cell.x as f32 * FOG_CELL_SIZE,
                0_f32,
                cell.z as f32 * FOG_CELL_SIZE,
            );
            if cell_world.distance(center) <= visibility_radius {
                fog_map.0.insert(cell);
            }
        }
    }
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
        .init_resource::<FogExploredMap>()
        .add_systems(Update, reveal_around_zeppelin);

    #[cfg(debug_assertions)]
    app.add_systems(Update, debug_explored_map);
}
