use crate::input::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct CameraMovementIntent {
    focal_point: Vec3,
    distance: f32,
    yaw: f32,
    pitch: f32,
}

impl Default for CameraMovementIntent {
    fn default() -> Self {
        Self {
            focal_point: Vec3::ZERO,
            distance: 25.0,
            yaw: 90.0,
            pitch: 22.5,
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::from("Main Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
        pan_orbit_controls(),
        CameraMovementIntent::default(),
    ));
}

fn update_camera(camera: Single<(&mut Transform, &CameraMovementIntent)>) {
    let (mut transform, intent) = camera.into_inner();

    transform.rotation = Quat::from_euler(
        EulerRot::YXZ,
        intent.yaw.to_radians(),
        -intent.pitch.to_radians(),
        0.0,
    );
    transform.translation = intent.focal_point + transform.back() * intent.distance;
}

fn on_pan(trigger: On<Fire<Pan>>, camera: Single<(&mut CameraMovementIntent, &Transform)>) {
    let (mut intent, transform) = camera.into_inner();
    intent.focal_point += transform.right() * trigger.value.x;
    intent.focal_point += Vec3::Y.cross(transform.right().as_vec3()) * trigger.value.y;
}

fn on_zoom(trigger: On<Fire<Zoom>>, mut intent: Single<&mut CameraMovementIntent>) {
    intent.distance = (intent.distance + trigger.value).clamp(5.0, 100.0);
}

fn on_orbit_with_enable(
    trigger: On<Fire<Orbit>>,
    orbit_enabled: Single<&ActionEvents, With<Action<EnableOrbit>>>,
    mut intent: Single<&mut CameraMovementIntent>,
) {
    if orbit_enabled.contains(ActionEvents::FIRE) {
        intent.yaw += trigger.value.x;
        intent.pitch += trigger.value.y;
        intent.pitch = intent.pitch.clamp(-10.0, 90.0);
    }
}

pub fn plugin(app: &mut App) {
    app.register_type::<CameraMovementIntent>()
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, update_camera)
        .add_observer(on_pan)
        .add_observer(on_zoom)
        .add_observer(on_orbit_with_enable);
}
