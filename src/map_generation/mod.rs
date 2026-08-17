use std::f32::consts::PI;

use bevy::prelude::*;

use crate::utils::hex::{AxialCoordinates, DEFAULT_HEX_SIZE};

fn spawn_map(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for q in 0..10 {
        for r in 0..10 {
            let translation = AxialCoordinates::new(q, r).to_world_coordinates(DEFAULT_HEX_SIZE);
            commands.spawn((
                Mesh3d(meshes.add(Extrusion::new(RegularPolygon::default(), 0.1))),
                MeshMaterial3d(materials.add(StandardMaterial::default())),
                Transform::from_rotation(Quat::from_rotation_x(-PI / 2.))
                    .with_translation(translation),
            ));
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_map);
}
