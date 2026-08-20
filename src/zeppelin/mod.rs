use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;

use crate::{
    pointer::SelectTileMessage,
    ui::{self, AttachToUiSlot},
    utils::{hex::*, scale::WorldScale, types::*},
    zeppelin::zeppelin_path::ZeppelinPath,
};

mod zeppelin_path;

/// Represents the base of the Zeppelin. This is the entity that is moved and
/// rotated.
#[derive(Component)]
struct ZeppelinWrapper;

/// Represents zeppelin movement settings.
#[derive(Component, Reflect)]
#[reflect(Component)]
struct ZeppelinMovementSettings {
    current_speed: Velocity,
    cruising_speed: Velocity,
    acceleration: Acceleration,
    deceleration: Acceleration,
    maximum_turn_radius: f32,
    warmup_duration: Duration,
    /// kg / s
    fuel_consumption_rate: f32,
    /// m³ / s
    gas_consumption_rate: f32,
    /// in kg
    fuel_capacity: f32,
    /// in m³
    gas_capacity: f32,
}

impl ZeppelinMovementSettings {
    fn new(
        cruising_speed: Velocity,
        acceleration: Acceleration,
        deceleration: Acceleration,
        maximum_turn_radius: f32,
        warmup_duration: Duration,
        fuel_consumption_rate: f32,
        gas_consumption_rate: f32,
    ) -> Self {
        Self {
            current_speed: Velocity(0.0),
            cruising_speed,
            acceleration,
            deceleration,
            maximum_turn_radius,
            warmup_duration,
            fuel_consumption_rate,
            gas_consumption_rate,
            fuel_capacity: 8_000.0,
            gas_capacity: 30_000.0,
        }
    }

    fn braking_distance(&self) -> Length {
        self.current_speed.squared() / (2.0 * self.deceleration)
    }

    fn accelerate(&mut self, delta: &Duration) {
        self.current_speed += self.acceleration * delta;
        self.current_speed = self.current_speed.clamp(Velocity(0.0), self.cruising_speed);
    }

    fn decelerate(&mut self, delta: &Duration) {
        self.current_speed -= self.deceleration * delta;
        self.current_speed = self.current_speed.clamp(Velocity(0.0), self.cruising_speed);
    }
}

fn setup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    scale: Res<WorldScale>,
) {
    commands
        .spawn((
            Name::from("Zeppelin Wrapper"),
            ZeppelinWrapper,
            Visibility::Inherited,
            Transform::default(),
            ZeppelinMovementSettings::new(
                Velocity(scale.units(33.0)),     // real LZ127 cruise speed, 33 m/s
                Acceleration(scale.units(0.15)), // real LZ127 acceleration, 0.15 m/s²
                Acceleration(scale.units(0.35)), // real LZ127 deceleration, 0.35 m/s²
                scale.units(100.0),              // real LZ127 turning radius, 100m
                Duration::from_mins(15),
                250.0 / 3600.0,
                250.0 / 3600.0,
            ),
        ))
        .with_child((
            Mesh3d(meshes.add(Capsule3d::default())),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
            Transform::from_xyz(0.0, 10.0, 0.0).with_rotation(Quat::from_rotation_x(-PI / 2.0)),
        ));
}

/// Represents a possible course. There will only ever be one, so this is a
/// resource
#[derive(Reflect, Resource)]
#[reflect(Resource)]
struct PossibleCourse {
    target: AxialCoordinates,
    path: ZeppelinPath,
    duration: Duration,
    fuel_consumption: f32,
    gas_consumption: f32,
}

#[derive(Component)]
struct SetCourseButton;

fn on_insert_course(
    _trigger: On<Insert, PossibleCourse>,
    possible_course: Res<PossibleCourse>,
    previous_buttons: Query<Entity, With<SetCourseButton>>,
    mut commands: Commands,
) {
    info!("course inserted");
    let hours = possible_course.duration.as_secs() / 3600;
    let mins = (possible_course.duration.as_secs()) / 60 % 60;
    let content = format!(
        "Fuel: {:.1}kg | Gas: {:.1}m³ | Time: {}h{}min",
        possible_course.fuel_consumption, possible_course.gas_consumption, hours, mins
    );
    commands.spawn((
        ui::primary_button("Set Course", Some(content), AttachToUiSlot::Action),
        SetCourseButton,
    ));

    for entity in &previous_buttons {
        commands.entity(entity).despawn();
    }
}

fn on_remove_course(
    _trigger: On<Remove, PossibleCourse>,
    previous_button: Single<Entity, With<SetCourseButton>>,
    mut commands: Commands,
) {
    commands.entity(previous_button.entity()).despawn();
}

fn calculate_cruise_time(
    total_length: Length,
    cruising_velocity: Velocity,
    acceleration: Acceleration,
    deceleration: Acceleration,
) -> Duration {
    let mut acceleration_distance = cruising_velocity.squared() / (2.0 * acceleration);
    let mut deceleration_distance = cruising_velocity.squared() / (2.0 * deceleration);

    let mut peak_velocity = cruising_velocity;
    if acceleration_distance + deceleration_distance > total_length {
        let nom = 2.0 * acceleration.0 * deceleration.0 * total_length.0;
        let denom = acceleration + deceleration;
        let peak = (nom / denom.0).sqrt();
        peak_velocity = Velocity(peak);
        acceleration_distance = peak_velocity.squared() / (2.0 * acceleration);
        deceleration_distance = peak_velocity.squared() / (2.0 * deceleration);
    }

    let cruise_length =
        (total_length - acceleration_distance - deceleration_distance).max(Length(0.0));
    let accel_time = peak_velocity / acceleration;
    let decel_time = peak_velocity / deceleration;
    let cruise_time = cruise_length / cruising_velocity;
    accel_time + cruise_time + decel_time
}

/// listens for the [`SelectedTileMessage`] and inserts a possible course with
/// the given coordinates.
fn read_selected_tiles(
    mut reader: MessageReader<SelectTileMessage>,
    possible_course_maybe: Option<Res<PossibleCourse>>,
    zeppelin: Single<(&Transform, &ZeppelinMovementSettings), With<ZeppelinWrapper>>,
    mut commands: Commands,
) {
    let (transform, settings) = zeppelin.into_inner();
    for ev in reader.read() {
        if possible_course_maybe
            .as_ref()
            .is_some_and(|course| course.target == ev.0)
        {
            commands.remove_resource::<PossibleCourse>();
        } else {
            let target = ev.0.to_world_coordinates(DEFAULT_HEX_SIZE);
            if let Ok(path) = ZeppelinPath::new(
                transform.translation,
                transform.forward().as_vec3(),
                target,
                settings.maximum_turn_radius,
            ) {
                let time = calculate_cruise_time(
                    Length(path.total_length()),
                    settings.cruising_speed,
                    settings.acceleration,
                    settings.deceleration,
                );
                let fuel_duration = time.min(settings.warmup_duration);
                let gas_duration = time - fuel_duration;

                let fuel_consumption = fuel_duration.as_secs_f32() * settings.fuel_consumption_rate;
                let gas_consumption = gas_duration.as_secs_f32() * settings.gas_consumption_rate;
                commands.insert_resource(PossibleCourse {
                    target: ev.0,
                    path,
                    duration: time,
                    fuel_consumption,
                    gas_consumption,
                });
            }
        }
    }
}

fn control_speed(
    time: Res<Time<Virtual>>,
    mut query: Query<(&ZeppelinPath, &mut ZeppelinMovementSettings)>,
) {
    for (path, mut settings) in &mut query {
        if path.remaining_length() <= settings.braking_distance().0 {
            settings.decelerate(&time.delta());
        } else {
            settings.accelerate(&time.delta());
        }
        settings.fuel_capacity -= settings.fuel_consumption_rate * time.delta_secs();
    }
}

fn tick_path(
    time: Res<Time<Virtual>>,
    mut query: Query<(&mut ZeppelinPath, &ZeppelinMovementSettings)>,
) {
    for (mut path, settings) in &mut query {
        let distance = settings.current_speed * time.delta();
        path.distance_traveled += distance.0;
    }
}

#[derive(Message)]
pub struct ReachedCoordinatesMessage(pub AxialCoordinates);

fn follow_path(
    mut writer: MessageWriter<ReachedCoordinatesMessage>,
    mut query: Query<(Entity, &mut Transform, &ZeppelinPath)>,
    mut commands: Commands,
) {
    for (entity, mut transform, path) in &mut query {
        let (position, forward) = if path.distance_traveled <= path.arc_length {
            let start_angle = path.start_angle();
            let swept = path.distance_traveled / path.radius;
            let angle = if path.turn_left {
                start_angle + swept
            } else {
                start_angle - swept
            };
            let position = path.center + path.radius * Vec3::new(angle.cos(), 0.0, angle.sin());
            let radius_dir = (position - path.center).normalize();
            let forward = if path.turn_left {
                Vec3::new(-radius_dir.z, 0.0, radius_dir.x)
            } else {
                Vec3::new(radius_dir.z, 0.0, -radius_dir.x)
            };
            (position, forward)
        } else {
            let t =
                (path.distance_traveled - path.arc_length) / path.straight_length.max(f32::EPSILON);
            (
                path.tangent_point.lerp(path.target, t.clamp(0.0, 1.0)),
                (path.target - path.tangent_point).normalize(),
            )
        };

        transform.translation = position;
        transform.look_to(forward, Vec3::Y);

        if path.is_completed() {
            commands.entity(entity).remove::<ZeppelinPath>();
            writer.write(ReachedCoordinatesMessage(
                AxialCoordinates::from_world_coordinates(transform.translation, DEFAULT_HEX_SIZE),
            ));
        }
    }
}

fn brake(
    time: Res<Time<Virtual>>,
    mut query: Query<(&mut Transform, &mut ZeppelinMovementSettings), Without<ZeppelinPath>>,
) {
    for (mut transform, mut settings) in &mut query {
        settings.decelerate(&time.delta());
        let distance = settings.current_speed * time.delta();
        let forward = transform.forward();
        transform.translation += forward * distance.0;
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
    let end = course.target.to_world_coordinates(DEFAULT_HEX_SIZE);
    gizmos.arrow(start, end, ORANGE);
}

#[cfg(debug_assertions)]
fn debug_zeppelin_path(mut gizmos: Gizmos, zeppelin: Single<&ZeppelinPath>) {
    use bevy::color::palettes::css::PURPLE;

    // Drawn manually (not via short_arc_3d_between/long_arc_3d_between) because Bevy's
    // long-arc helper only lands on `to` when the short angle is exactly PI - otherwise
    // it sweeps the same direction as the short arc and ends on a mirrored point instead.
    let start_angle = zeppelin.start_angle();
    let resolution = 32;
    let points = (0..=resolution).map(|i| {
        let t = i as f32 / resolution as f32;
        let angle = if zeppelin.turn_left {
            start_angle + zeppelin.sweep * t
        } else {
            start_angle - zeppelin.sweep * t
        };
        zeppelin.center + zeppelin.radius * Vec3::new(angle.cos(), 0.0, angle.sin())
    });
    gizmos.linestrip(points, PURPLE);
    gizmos.line(zeppelin.tangent_point, zeppelin.target, PURPLE);
}

#[cfg(debug_assertions)]
fn debug_zeppelin_forward(mut gizmos: Gizmos, zeppelin: Single<&Transform, With<ZeppelinWrapper>>) {
    let transform = zeppelin.into_inner();
    gizmos.axes(*transform, 1.0);
}

pub fn plugin(app: &mut App) {
    app.register_type::<PossibleCourse>()
        .add_message::<ReachedCoordinatesMessage>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                read_selected_tiles,
                brake,
                (control_speed, tick_path, follow_path).chain(),
            ),
        )
        .add_observer(on_insert_course)
        .add_observer(on_remove_course);

    #[cfg(debug_assertions)]
    app.add_systems(
        Update,
        (
            debug_course.run_if(resource_exists::<PossibleCourse>),
            debug_zeppelin_path,
            debug_zeppelin_forward,
        ),
    );
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::{
        utils::types::{Acceleration, Length, Velocity},
        zeppelin::calculate_cruise_time,
    };

    #[test]
    fn test_long_travel_time() {
        let time = calculate_cruise_time(
            Length(100.0),
            Velocity(5.0),
            Acceleration(2.0),
            Acceleration(2.0),
        );
        assert_eq!(time, Duration::from_secs_f32(22.5));
    }

    #[test]
    fn test_short_travel_time() {
        let time = calculate_cruise_time(
            Length(10.0),
            Velocity(5.0),
            Acceleration(2.0),
            Acceleration(2.0),
        );
        assert!((time.as_secs_f32() - 20f32.sqrt()).abs() < f32::EPSILON);
    }
}
