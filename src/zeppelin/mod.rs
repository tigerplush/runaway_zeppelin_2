use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;

use crate::{
    pointer::SelectTileMessage,
    utils::{hex::*, scale::WorldScale, types::*},
    zeppelin::{
        fuel_tank::{FuelConsumptionRequest, FuelTank},
        possible_course::PossibleCourse,
        zeppelin_path::ZeppelinPath,
    },
};

mod fuel_tank;
mod possible_course;
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
    drag: Acceleration,
    maximum_turn_radius: f32,
}

impl ZeppelinMovementSettings {
    fn new(
        cruising_speed: Velocity,
        acceleration: Acceleration,
        deceleration: Acceleration,
        drag: Acceleration,
        maximum_turn_radius: f32,
    ) -> Self {
        Self {
            current_speed: Velocity(0.0),
            cruising_speed,
            acceleration,
            deceleration,
            drag,
            maximum_turn_radius,
        }
    }

    fn braking_distance(&self) -> Length {
        self.current_speed.squared() / (2.0 * self.deceleration)
    }

    fn accelerate(&mut self, delta: &Duration, multiplier: f32) {
        self.current_speed += self.acceleration * multiplier * delta;
        self.current_speed = self.current_speed.clamp(Velocity(0.0), self.cruising_speed);
    }

    fn decelerate(&mut self, delta: &Duration, multiplier: f32) {
        self.current_speed -= self.deceleration * multiplier * delta;
        self.current_speed = self.current_speed.clamp(Velocity(0.0), self.cruising_speed);
    }

    fn decelerate_with_drag(&mut self, delta: &Duration) {
        self.current_speed -= self.drag * delta;
        self.current_speed = self.current_speed.clamp(Velocity(0.0), self.cruising_speed);
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Engine {
    current_temperature: Temperature,
    operating_temperature: Temperature,
    warmup_duration: Duration,
    cooldown_duration: Duration,
    /// kg / s
    fuel_consumption_rate: f32,
    /// m³ / s
    gas_consumption_rate: f32,
    /// Power output in % (from 0.0 to 1.0)
    power_output: f32,
}

impl Engine {
    fn heat_up(&mut self, delta: Duration) {
        let warmup_rate =
            (self.operating_temperature - Temperature::AMBIENT_TEMPERATURE) / self.warmup_duration;
        self.current_temperature += warmup_rate * delta;
        self.current_temperature = self
            .current_temperature
            .clamp(Temperature::AMBIENT_TEMPERATURE, self.operating_temperature);
    }

    fn cool_down(&mut self, delta: Duration) {
        let cooldown_rate = (Temperature::AMBIENT_TEMPERATURE - self.operating_temperature)
            / self.cooldown_duration;
        self.current_temperature += cooldown_rate * delta;
        self.current_temperature = self
            .current_temperature
            .clamp(Temperature::AMBIENT_TEMPERATURE, self.operating_temperature);
    }

    fn fuel_consumption_rate(&self) -> f32 {
        if self.current_temperature.as_celsius() > 69.9 {
            0.0
        } else {
            self.fuel_consumption_rate
        }
    }

    fn gas_consumption_rate(&self) -> f32 {
        if self.current_temperature.as_celsius() > 69.9 {
            self.gas_consumption_rate
        } else {
            0.0
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
enum EngineDemand {
    /// accelerate toward cruise speed
    Thrust,
    /// actively decelerating - approaching target, or arrived and bleeding off speed, or out of fuel mid-course
    Brake,
    /// fully stopped, no active propulsion - drag only from here
    Shutdown,
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
                // real LZ127 cruise speed, 33 m/s
                Velocity(scale.units(33.0)),
                // real LZ127 acceleration, 0.15 m/s²
                Acceleration(scale.units(0.15)),
                Acceleration(scale.units(0.35)), // real LZ127 deceleration, 0.35 m/s²
                Acceleration(scale.units(0.05)),
                scale.units(100.0), // real LZ127 turning radius, 100m
            ),
            Engine {
                current_temperature: Temperature::from_celsius(20.0),
                operating_temperature: Temperature::from_celsius(70.0),
                warmup_duration: Duration::from_mins(15),
                cooldown_duration: Duration::from_hours(8),
                fuel_consumption_rate: 250.0 / 3600.0,
                gas_consumption_rate: 250.0 / 3600.0,
                power_output: 0.0,
            },
            FuelTank::default(),
            EngineDemand::Shutdown,
        ))
        .with_child((
            Mesh3d(meshes.add(Capsule3d::default())),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
            Transform::from_xyz(0.0, 10.0, 0.0).with_rotation(Quat::from_rotation_x(-PI / 2.0)),
        ));
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
    zeppelin: Single<(&Transform, &ZeppelinMovementSettings, &Engine), With<ZeppelinWrapper>>,
    mut commands: Commands,
) {
    let (transform, settings, engine) = zeppelin.into_inner();
    for ev in reader.read() {
        if possible_course_maybe
            .as_ref()
            .is_some_and(|course| course.target == ev.0)
        {
            commands.remove_resource::<PossibleCourse>();
        } else {
            let target = ev.0.as_world_coordinates(DEFAULT_HEX_SIZE);
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
                let fuel_duration = time.min(engine.warmup_duration);
                let gas_duration = time - fuel_duration;

                let fuel_consumption = fuel_duration.as_secs_f32() * engine.fuel_consumption_rate;
                let gas_consumption = gas_duration.as_secs_f32() * engine.gas_consumption_rate;
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

fn decide_engine_demand(
    mut query: Query<(
        &mut EngineDemand,
        Option<&ZeppelinPath>,
        &ZeppelinMovementSettings,
        &FuelTank,
        &Engine,
    )>,
) {
    for (mut demand, path, settings, fuel_tank, engine) in &mut query {
        let has_fuel = fuel_tank.can_supply(engine.gas_consumption_rate());
        *demand = match path {
            Some(_) if !has_fuel => EngineDemand::Shutdown,
            Some(path) if path.remaining_length() <= settings.braking_distance().0 => {
                EngineDemand::Brake
            }
            Some(_) => EngineDemand::Thrust,
            None if settings.current_speed.0 > 0.0 => EngineDemand::Brake,
            None => EngineDemand::Shutdown,
        };
    }
}

fn apply_engine_demand(
    time: Res<Time<Virtual>>,
    mut query: Query<(&EngineDemand, &mut Engine, &mut FuelTank)>,
) {
    for (demand, mut engine, mut fuel_tank) in &mut query {
        match demand {
            EngineDemand::Thrust | EngineDemand::Brake => {
                engine.heat_up(time.delta());
                let fuel_amount = engine.fuel_consumption_rate() * time.delta_secs();
                let gas_amount = engine.gas_consumption_rate() * time.delta_secs();
                let power_output = fuel_tank.consume(FuelConsumptionRequest {
                    fuel_amount,
                    gas_amount,
                });
                engine.power_output = power_output;
            }
            EngineDemand::Shutdown => engine.cool_down(time.delta()),
        }
    }
}

fn apply_speed(
    time: Res<Time<Virtual>>,
    mut query: Query<(&EngineDemand, &Engine, &mut ZeppelinMovementSettings)>,
) {
    for (demand, engine, mut settings) in &mut query {
        match demand {
            EngineDemand::Thrust => settings.accelerate(&time.delta(), engine.power_output),
            EngineDemand::Brake => settings.decelerate(&time.delta(), engine.power_output),
            EngineDemand::Shutdown => settings.decelerate_with_drag(&time.delta()),
        }
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
    app.register_type::<Engine>()
        .register_type::<EngineDemand>()
        .add_plugins((fuel_tank::plugin, possible_course::plugin))
        .add_message::<ReachedCoordinatesMessage>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                read_selected_tiles,
                (
                    decide_engine_demand,
                    apply_engine_demand,
                    apply_speed,
                    tick_path,
                    follow_path,
                )
                    .chain(),
            ),
        );

    #[cfg(debug_assertions)]
    app.add_systems(Update, (debug_zeppelin_path, debug_zeppelin_forward));
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
