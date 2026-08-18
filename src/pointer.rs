use bevy::prelude::*;
use bevy_enhanced_input::action::events::Start;

use crate::{
    input::*,
    utils::hex::{AxialCoordinates, DEFAULT_HEX_SIZE},
};

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Pointer {
    previous_position: Option<AxialCoordinates>,
    current_position: Option<AxialCoordinates>,
}

fn setup(mut commands: Commands) {
    commands.spawn((Pointer::default(), pointer_control(), Transform::default()));
}

fn update_pointer(
    primary_window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    pointer: Single<(&mut Pointer, &mut Transform)>,
) {
    let (mut pointer, mut transform) = pointer.into_inner();
    pointer.previous_position = pointer.current_position;
    let Some(cursor_position) = primary_window.cursor_position() else {
        pointer.current_position = None;
        return;
    };

    let (camera, camera_transform) = camera.into_inner();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        pointer.current_position = None;
        return;
    };

    pointer.current_position = if let Some(point) =
        ray.plane_intersection_point(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
    {
        transform.translation = point;
        let axial_coords = AxialCoordinates::from_world_coordinates(point, DEFAULT_HEX_SIZE);
        Some(axial_coords)
    } else {
        None
    };
}

#[derive(Debug, Message)]
pub struct SelectTileMessage(pub AxialCoordinates);

fn on_select_tile(
    _trigger: On<Start<SelectTile>>,
    pointer: Single<&Pointer>,
    mut writer: MessageWriter<SelectTileMessage>,
) {
    if let Some(current_position) = pointer.current_position {
        writer.write(SelectTileMessage(current_position));
    }
}

#[cfg(debug_assertions)]
fn debug_pointer(mut gizmos: Gizmos, pointer: Single<(&Transform, &Pointer)>) {
    use bevy::color::palettes::css::{GREEN, ORANGE};

    let (transform, pointer) = pointer.into_inner();
    gizmos.arrow(
        transform.translation + Vec3::Y,
        transform.translation,
        GREEN,
    );
    if let Some(point) = pointer.current_position {
        let translation = point.to_world_coordinates(DEFAULT_HEX_SIZE);
        gizmos.arrow(translation + Vec3::Y, translation, ORANGE);
    }
}

pub fn plugin(app: &mut App) {
    app.register_type::<Pointer>()
        .add_message::<SelectTileMessage>()
        .add_systems(Startup, setup)
        .add_systems(Update, update_pointer)
        .add_observer(on_select_tile);

    #[cfg(debug_assertions)]
    app.add_systems(Update, debug_pointer);
}
