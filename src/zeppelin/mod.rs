use bevy::prelude::*;

use crate::states::AppStates;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppStates::MainApp), setup);
}

const RADIUS: f32 = 30.5 / 2.0;
const LENGTH: f32 = 236.6;
pub const HEIGHT: f32 = 200.0;

fn setup(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    commands
        .spawn((
            Name::new("Zeppelin Holder"),
            Transform::default(),
            Visibility::default(),
        ))
        .with_children(|zeppelin| {
            zeppelin.spawn((
                Mesh3d(meshes.add(Capsule3d::new(RADIUS, LENGTH))),
                MeshMaterial3d(materials.add(StandardMaterial { ..default() })),
                Transform::from_xyz(0.0, HEIGHT, 0.0).with_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    90_f32.to_radians(),
                    0.0,
                    0.0,
                )),
            ));
        });
}
