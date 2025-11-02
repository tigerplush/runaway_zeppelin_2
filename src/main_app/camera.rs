use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{states::AppStates, zeppelin::HEIGHT};

pub fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<CameraActions>::default())
        .add_systems(OnEnter(AppStates::MainApp), setup)
        .add_systems(
            PostUpdate,
            update_camera.run_if(in_state(AppStates::MainApp)),
        );
}

fn setup(mut commands: Commands) {
    let input_map = InputMap::default()
        .with_axis(CameraActions::Zoom, MouseScrollAxis::Y)
        .with_dual_axis(
            CameraActions::Pan,
            VirtualDPad::wasd().with_circle_bounds(1.0),
        )
        .with_dual_axis(
            CameraActions::Orbit,
            MouseMove::default().with_circle_bounds(1.0),
        )
        .with(CameraActions::EnableOrbit, MouseButton::Middle);

    commands.spawn((
        DespawnOnExit(AppStates::MainApp),
        Camera3d::default(),
        input_map,
        CameraSettings {
            focal_point: Vec3::new(0.0, HEIGHT, 0.0),
            distance: 250.0,
            yaw: 90.0,
            pitch: 22.5,
            pan_sensitivity: 10.0,
            zoom_sensitivity: 0.2,
            orbit_sensitivity: 1.0,
            pitch_extents: (-10.0, 90.0),
        },
    ));
}

#[derive(Actionlike, Clone, Debug, Eq, Hash, PartialEq, Reflect)]
enum CameraActions {
    #[actionlike(DualAxis)]
    Pan,
    #[actionlike(Axis)]
    Zoom,
    #[actionlike(DualAxis)]
    Orbit,
    EnableOrbit,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct CameraSettings {
    focal_point: Vec3,
    distance: f32,
    /// rotation around the object
    yaw: f32,
    /// view height
    pitch: f32,
    pan_sensitivity: f32,
    zoom_sensitivity: f32,
    orbit_sensitivity: f32,
    pitch_extents: (f32, f32),
}

fn update_camera(
    camera: Single<(
        &mut Transform,
        &mut CameraSettings,
        &ActionState<CameraActions>,
    )>,
) {
    let (mut camera_transform, mut settings, action_state) = camera.into_inner();

    let pan_vec = action_state.axis_pair(&CameraActions::Pan);
    if Vec2::ZERO != pan_vec {
        let pan_sensitivity = settings.pan_sensitivity;
        settings.focal_point += camera_transform.right() * pan_vec.x * pan_sensitivity;
        settings.focal_point +=
            Vec3::Y.cross(camera_transform.right().as_vec3()) * pan_vec.y * pan_sensitivity;
    }

    let zoom = action_state.value(&CameraActions::Zoom);
    let zoom_sensitivity = settings.zoom_sensitivity;
    settings.distance *= (zoom * zoom_sensitivity).exp();

    let orbit_vec = action_state.axis_pair(&CameraActions::Orbit);
    if action_state.pressed(&CameraActions::EnableOrbit) && Vec2::ZERO != orbit_vec {
        let orbit_sensitivity = settings.orbit_sensitivity;
        let pitch_extents = settings.pitch_extents;
        settings.yaw += orbit_vec.x * orbit_sensitivity;
        settings.pitch += orbit_vec.y * orbit_sensitivity;
        settings.pitch = settings.pitch.clamp(pitch_extents.0, pitch_extents.1);
    }

    camera_transform.rotation = Quat::from_euler(
        EulerRot::YXZ,
        settings.yaw.to_radians(),
        -settings.pitch.to_radians(),
        0.0,
    );
    camera_transform.translation =
        settings.focal_point + camera_transform.back() * settings.distance;
}
