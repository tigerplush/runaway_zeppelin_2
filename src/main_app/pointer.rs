use bevy::{
    color::palettes::css::{GREEN, ORANGE},
    prelude::*,
    window::PrimaryWindow,
};

use crate::{hex::AxialCoordinates, states::AppStates};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppStates::MainApp), setup)
        .add_systems(
            Update,
            (update_pointer, draw_pointer)
                .chain()
                .run_if(in_state(AppStates::MainApp)),
        );
}

fn setup(mut commands: Commands) {
    commands.spawn((
        MousePointer {
            world_position: None,
        },
        DespawnOnExit(AppStates::MainApp),
    ));
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct MousePointer {
    world_position: Option<Vec3>,
}

fn update_pointer(
    mut mouse_pointer: Single<&mut MousePointer>,
    interactions: Query<&Interaction>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    mouse_pointer.world_position = None;

    if !interactions.iter().all(|f| Interaction::None == *f) {
        // if there are any interactions outstanding, we don't update the pointer
        return;
    }

    let Some(viewport_position) = window.cursor_position() else {
        // Pointer is not over the main window
        return;
    };

    let (camera, camera_transform) = camera.into_inner();

    let Ok(ray) = camera.viewport_to_world(camera_transform, viewport_position) else {
        // there was a conversion error
        return;
    };

    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else {
        // ray does not intersect with ground plane
        return;
    };
    let world_coordinates = ray.get_point(distance);
    mouse_pointer.world_position = Some(world_coordinates);
}

fn draw_pointer(mut gizmos: Gizmos, mouse_pointer: Single<&MousePointer>) {
    if let Some(world_coordinates) = mouse_pointer.world_position {
        gizmos.arrow(world_coordinates + Vec3::Y, world_coordinates, ORANGE);
        let hex_coordinates =
            AxialCoordinates::from_world_coordinates(world_coordinates).to_world_coordinates();
        gizmos.arrow(hex_coordinates + Vec3::Y, hex_coordinates, GREEN);
    }
}
