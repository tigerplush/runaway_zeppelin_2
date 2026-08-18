use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    pathfinding::{Path, Pathfinder},
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
                maximum_turn_rate: 45.0_f32.to_radians(),
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
    points: Vec<Vec3>,
    current: usize,
}

impl ZeppelinPath {
    const ARRIVAL_RADIUS: f32 = 0.1;
}

impl From<&Path> for ZeppelinPath {
    fn from(value: &Path) -> Self {
        Self {
            points: value
                .points()
                .iter()
                .map(|p| p.to_world_coordinates(DEFAULT_HEX_SIZE))
                .collect(),
            current: 0,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ZeppelinMovementSettings {
    speed: f32,
    maximum_turn_rate: f32,
}

fn transform_path_to_zeppelin_path(
    trigger: On<Insert, Path>,
    query: Query<&Path>,
    zeppelin: Single<Entity, With<ZeppelinWrapper>>,
    mut commands: Commands,
) {
    let Ok(path) = query.get(trigger.entity) else {
        return;
    };

    commands
        .entity(zeppelin.entity())
        .insert(ZeppelinPath::from(path));
    commands.entity(trigger.entity).despawn();
}

#[derive(Reflect, Resource)]
#[reflect(Resource)]
struct PossibleCourse(AxialCoordinates);

/// listens for the [`SelectedTileMessage`] and inserts a possible course with the given coordinates
fn read_selected_tiles(
    mut reader: MessageReader<SelectTileMessage>,
    possible_course_maybe: Option<Res<PossibleCourse>>,
    zeppelin: Single<&Transform, With<ZeppelinWrapper>>,
    mut commands: Commands,
) {
    for ev in reader.read() {
        if possible_course_maybe
            .as_ref()
            .is_some_and(|course| course.0 == ev.0)
        {
            commands.remove_resource::<PossibleCourse>();
        } else {
            commands.insert_resource(PossibleCourse(ev.0));
            commands
                .spawn((
                    Name::from("ZeppelinPath"),
                    Pathfinder::new(
                        AxialCoordinates::from_world_coordinates(
                            zeppelin.translation,
                            DEFAULT_HEX_SIZE,
                        ),
                        ev.0,
                    ),
                ))
                .observe(transform_path_to_zeppelin_path);
        }
    }
}

fn follow_path(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut ZeppelinPath, &ZeppelinMovementSettings)>,
) {
    for (mut transform, mut path, settings) in &mut query {
        let Some(&target) = path.points.get(path.current) else {
            continue;
        };

        let to_target = target - transform.translation;
        if to_target.length() < ZeppelinPath::ARRIVAL_RADIUS {
            path.current += 1;
            continue;
        }

        let desired_forward = to_target.normalize();
        let current_forward = transform.forward().as_vec3();
        let angle = current_forward.angle_between(desired_forward);
        let max_step = settings.maximum_turn_rate * time.delta_secs();

        let new_forward = if angle <= max_step {
            desired_forward
        } else {
            current_forward
                .slerp(desired_forward, max_step / angle)
                .normalize()
        };
        transform.look_to(new_forward, Vec3::Y);

        let forward = transform.forward().as_vec3();
        transform.translation += forward * settings.speed * time.delta_secs();
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
    let current = zeppelin.current;
    for (index, points) in zeppelin.points.windows(2).enumerate() {
        use bevy::color::palettes::css::{GREEN, ORANGE};

        let color = if index == current { GREEN } else { ORANGE };
        gizmos.arrow(points[0], points[1], color);
    }
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
        .add_systems(Update, (read_selected_tiles, follow_path));

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
