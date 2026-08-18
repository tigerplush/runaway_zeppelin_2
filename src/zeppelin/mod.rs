use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{pointer::SelectTileMessage, utils::hex::AxialCoordinates};

#[derive(Component)]
struct ZeppelinWrapper;

fn setup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    commands
        .spawn((
            Name::from("Zeppelin Wrapper"),
            ZeppelinWrapper,
            Visibility::Inherited,
            Transform::default(),
        ))
        .with_child((
            Mesh3d(meshes.add(Capsule3d::default())),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
            Transform::from_xyz(0.0, 1.0, 0.0).with_rotation(Quat::from_rotation_x(-PI / 2.0)),
        ));
}

#[derive(Reflect, Resource)]
#[reflect(Resource)]
struct PossibleCourse(AxialCoordinates);

/// listens for the [`SelectedTileMessage`] and inserts a possible course with the given coordinates
fn read_selected_tiles(mut reader: MessageReader<SelectTileMessage>, mut commands: Commands) {
    for ev in reader.read() {
        commands.insert_resource(PossibleCourse(ev.0));
    }
}

#[cfg(debug_assertions)]
fn debug_course(
    mut gizmos: Gizmos,
    course: Res<PossibleCourse>,
    zeppelin: Single<&Transform, With<ZeppelinWrapper>>,
) {
    use bevy::color::palettes::css::ORANGE;

    use crate::utils::hex::DEFAULT_HEX_SIZE;

    let start = zeppelin.translation;
    let end = course.0.to_world_coordinates(DEFAULT_HEX_SIZE);
    gizmos.arrow(start, end, ORANGE);
}

pub fn plugin(app: &mut App) {
    app.register_type::<PossibleCourse>()
        .add_systems(Startup, setup)
        .add_systems(Update, read_selected_tiles);

    #[cfg(debug_assertions)]
    app.add_systems(
        Update,
        debug_course.run_if(resource_exists::<PossibleCourse>),
    );
}
