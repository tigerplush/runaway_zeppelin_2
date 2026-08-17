use std::f32::consts::PI;

use bevy::prelude::*;

fn spawn_map(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    commands.spawn((
        Mesh3d(meshes.add(Extrusion::new(RegularPolygon::default(), 0.1))),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::from_rotation(Quat::from_rotation_x(-PI / 2.)),
    ));
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_map);
}
