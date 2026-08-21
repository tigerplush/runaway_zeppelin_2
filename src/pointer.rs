use bevy::prelude::*;
use bevy_enhanced_input::action::events::Start;
#[cfg(debug_assertions)]
use bevy_inspector_egui::bevy_egui::EguiContext;

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
    #[cfg(debug_assertions)] mut egui: Single<&mut EguiContext>,
    camera: Single<(&Camera, &GlobalTransform)>,
    pointer: Single<(&mut Pointer, &mut Transform)>,
) {
    #[cfg(debug_assertions)]
    if egui.get_mut().egui_wants_pointer_input() || egui.get_mut().egui_is_using_pointer() {
        return;
    }

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
    #[cfg(debug_assertions)] mut egui: Single<&mut EguiContext>,
    interactions: Query<&Interaction>,
    pointer: Single<&Pointer>,
    mut writer: MessageWriter<SelectTileMessage>,
) {
    if !interactions.iter().all(|f| *f == Interaction::None) {
        return;
    };

    #[cfg(debug_assertions)]
    if egui.get_mut().egui_wants_pointer_input() {
        return;
    }

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
