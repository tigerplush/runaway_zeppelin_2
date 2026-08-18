use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    pointer::SelectTileMessage,
    utils::hex::{AxialCoordinates, DEFAULT_HEX_SIZE},
};

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
            ZeppelinMovementSettings {
                speed: 0.5,
                maximum_turn_radius: 1.0,
            },
        ))
        .with_child((
            Mesh3d(meshes.add(Capsule3d::default())),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
            Transform::from_xyz(0.0, 1.0, 0.0).with_rotation(Quat::from_rotation_x(-PI / 2.0)),
        ));
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ZeppelinPath {
    start: Vec3,
    target: Vec3,
    center: Vec3,
    radius: f32,
    turn_left: bool,
    tangent_point: Vec3,
    sweep: f32,
    arc_length: f32,
    straight_length: f32,
}

impl ZeppelinPath {
    fn new(start: Vec3, heading: Vec3, target: Vec3, radius: f32) -> Result<Self, ()> {
        let to_target = target - start;
        let turn_left = heading.x * to_target.z - heading.z * to_target.x > 0.0;

        let side_normal = if turn_left {
            Vec3::new(-heading.z, 0.0, heading.x)
        } else {
            Vec3::new(heading.z, 0.0, -heading.x)
        };
        let center = start + side_normal * radius;

        let center_to_target = target - center;
        let d = center_to_target.length();
        if d < radius {
            return Err(()); // target's inside the turning circle, no CS solution
        }

        let base_angle = center_to_target.z.atan2(center_to_target.x);
        let theta = (radius / d).acos();

        for angle in [base_angle + theta, base_angle - theta] {
            let tangent_point = center + radius * Vec3::new(angle.cos(), 0.0, angle.sin());
            let radius_dir = (tangent_point - center).normalize();
            let travel_dir = if turn_left {
                Vec3::new(-radius_dir.z, 0.0, radius_dir.x)
            } else {
                Vec3::new(radius_dir.z, 0.0, -radius_dir.x)
            };
            if travel_dir.dot(target - tangent_point) > 0.0 {
                let start_angle = (start - center).z.atan2((start - center).x);
                let sweep = if turn_left {
                    angle - start_angle
                } else {
                    start_angle - angle
                }
                .rem_euclid(std::f32::consts::TAU);
                return Ok(Self {
                    start,
                    target,
                    center,
                    radius,
                    turn_left,
                    tangent_point,
                    sweep,
                    arc_length: radius * sweep,
                    straight_length: (target - tangent_point).length(),
                });
            }
        }
        Err(())
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ZeppelinMovementSettings {
    speed: f32,
    maximum_turn_radius: f32,
}

#[derive(Reflect, Resource)]
#[reflect(Resource)]
struct PossibleCourse(AxialCoordinates);

/// listens for the [`SelectedTileMessage`] and inserts a possible course with the given coordinates
fn read_selected_tiles(
    mut reader: MessageReader<SelectTileMessage>,
    possible_course_maybe: Option<Res<PossibleCourse>>,
    zeppelin: Single<(Entity, &Transform, &ZeppelinMovementSettings), With<ZeppelinWrapper>>,
    mut commands: Commands,
) {
    let (zeppelin, transform, settings) = zeppelin.into_inner();
    for ev in reader.read() {
        if possible_course_maybe
            .as_ref()
            .is_some_and(|course| course.0 == ev.0)
        {
            commands.remove_resource::<PossibleCourse>();
        } else {
            commands.insert_resource(PossibleCourse(ev.0));
            let target = ev.0.to_world_coordinates(DEFAULT_HEX_SIZE);
            if let Ok(zeppelin_path) = ZeppelinPath::new(
                transform.translation,
                transform.forward().as_vec3(),
                target,
                settings.maximum_turn_radius,
            ) {
                commands.entity(zeppelin).insert(zeppelin_path);
            }
        }
    }
}

fn tick_path() {

}

fn follow_path(
    mut query: Query<(&mut Transform, &ZeppelinPath)>,
) {
    for (mut transform, path) in &mut query {
        
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

#[cfg(debug_assertions)]
fn debug_zeppelin_path(mut gizmos: Gizmos, zeppelin: Single<&ZeppelinPath>) {
    use bevy::color::palettes::css::PURPLE;

    // Drawn manually (not via short_arc_3d_between/long_arc_3d_between) because Bevy's
    // long-arc helper only lands on `to` when the short angle is exactly PI - otherwise
    // it sweeps the same direction as the short arc and ends on a mirrored point instead.
    let to_start = zeppelin.start - zeppelin.center;
    let start_angle = to_start.z.atan2(to_start.x);
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
    use bevy::color::palettes::css::BLUE;

    gizmos.arrow(
        zeppelin.translation,
        zeppelin.translation + 1.0 * zeppelin.forward(),
        BLUE,
    );
}

pub fn plugin(app: &mut App) {
    app.register_type::<PossibleCourse>()
        .add_systems(Startup, setup)
        .add_systems(Update, (read_selected_tiles, tick_path, follow_path));

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
