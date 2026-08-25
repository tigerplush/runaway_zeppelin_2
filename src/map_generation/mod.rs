use std::{collections::HashMap, f32::consts::PI};

use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::ChaCha8Rng};
use rand::RngExt;

use crate::{
    map_generation::poi::{AvailablePois, Poi, WorldState},
    states::AppStates,
    utils::{
        hex::{AxialCoordinates, DEFAULT_HEX_SIZE},
        scale::WorldScale,
    },
    zeppelin::{ReachedCoordinatesMessage, ZeppelinWrapper},
};

mod poi;

fn sample_poisson_disc(
    radius: f32,
    sample_region: Rect,
    occupied_points: &mut Vec<Vec2>,
    num_samples_before_rejection: usize,
    rng: &mut ChaCha8Rng,
) -> Vec<Vec2> {
    let cell_size = radius / 2_f32.sqrt();
    let mut grid = HashMap::new();
    for (index, point) in occupied_points.iter().enumerate() {
        let x = (point.x / cell_size) as i32;
        let y = (point.y / cell_size) as i32;
        grid.insert(IVec2::new(x, y), index);
    }
    let mut new_points = Vec::new();
    let mut spawn_points = Vec::new();
    let starting_point = sample_region.center();
    spawn_points.push(starting_point);
    while let Some(spawn_centre) = spawn_points.pop() {
        let mut candidate_accepted = false;
        for _index in 0..num_samples_before_rejection {
            let angle = rng.random::<f32>() * 2.0 * PI;
            let dir = Vec2::new(angle.sin(), angle.cos());
            let distance = rng.random_range(radius..=radius * 2.0);
            let candidate = spawn_centre + dir * distance;

            if is_valid(
                &candidate,
                &sample_region,
                cell_size,
                &occupied_points,
                &grid,
                radius,
            ) {
                new_points.push(candidate);
                spawn_points.push(candidate);
                occupied_points.push(candidate);

                let x = (candidate.x / cell_size) as i32;
                let y = (candidate.y / cell_size) as i32;
                grid.insert(IVec2::new(x, y), occupied_points.len() - 1);
                candidate_accepted = true;
                break;
            }
        }

        if candidate_accepted {
            spawn_points.push(spawn_centre);
        }
    }

    new_points
}

fn is_valid(
    candidate: &Vec2,
    sample_region: &Rect,
    cell_size: f32,
    points: &[Vec2],
    grid: &HashMap<IVec2, usize>,
    radius: f32,
) -> bool {
    let radius_squared = radius * radius;
    let min = (sample_region.min / cell_size).as_ivec2();
    let max = (sample_region.max / cell_size).as_ivec2();

    if sample_region.contains(*candidate) {
        let cell_x = (candidate.x / cell_size) as i32;
        let cell_y = (candidate.y / cell_size) as i32;
        let x_min = (cell_x - 2).max(min.x);
        let x_max = (cell_x + 2).min(max.x);
        let y_min = (cell_y - 2).max(min.y);
        let y_max = (cell_y + 2).min(max.y);
        for x in x_min..=x_max {
            for y in y_min..=y_max {
                if let Some(point_index) = grid.get(&IVec2::new(x, y)) {
                    let distance_squared = (candidate - points[*point_index]).length_squared();

                    if distance_squared < radius_squared {
                        return false;
                    }
                }
            }
        }
        return true;
    }
    false
}

fn spawn_map(
    scale: Res<WorldScale>,
    mut available_pois: ResMut<AvailablePois>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    current_pois: Query<&Transform, (With<Poi>, Without<ZeppelinWrapper>)>,
    zeppelin: Single<&Transform, (With<ZeppelinWrapper>, Without<Poi>, Changed<Transform>)>,
    mut rng: Single<&mut ChaCha8Rng, With<GlobalRng>>,
    mut commands: Commands,
) {
    // on moved zeppelin:
    // set origin to zeppelin
    let origin = Vec2::new(zeppelin.translation.x, zeppelin.translation.z);
    let region_size = Vec2::splat(scale.units(50_000f32));

    let safe_region = Rect::from_center_size(origin, region_size * 2.0);
    // collect all Pois in range
    let mut previous_pois = current_pois
        .iter()
        // .filter(|&transform| {
        //     safe_region.contains(Vec2::new(transform.translation.x, transform.translation.z))
        // })
        .map(|&transform| Vec2::new(transform.translation.x, transform.translation.z))
        .collect::<Vec<Vec2>>();

    let sample_region = Rect::from_center_size(origin, region_size);
    // sample around zeppelin
    let p = sample_poisson_disc(
        scale.units(25_000f32),
        sample_region,
        &mut previous_pois,
        30,
        &mut rng,
    );

    for point in p {
        let coordinates = AxialCoordinates::from_world_coordinates(
            Vec3::new(point.x, 0.0, point.y),
            DEFAULT_HEX_SIZE,
        );

        let Some(_poi) = available_pois.get(&WorldState, &mut rng) else {
            continue;
        };

        commands.spawn((
            Mesh3d(meshes.add(Extrusion::new(RegularPolygon::default(), 0.1))),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
            Transform::from_rotation(Quat::from_rotation_x(-PI / 2.))
                .with_translation(coordinates.as_world_coordinates(DEFAULT_HEX_SIZE)),
            Poi(coordinates),
        ));
    }
}

#[derive(Message)]
pub struct ReachedPoiMessage(pub Entity);

fn read_reached_coordinates(
    mut reader: MessageReader<ReachedCoordinatesMessage>,
    query: Query<(Entity, &Poi)>,
    mut writer: MessageWriter<ReachedPoiMessage>,
) {
    for ev in reader.read() {
        if let Some((entity, _poi)) = query.iter().find(|&(_entity, poi)| poi.0 == ev.0) {
            writer.write(ReachedPoiMessage(entity));
        }
    }
}

pub fn plugin(app: &mut App) {
    app.register_type::<Poi>()
        .add_message::<ReachedPoiMessage>()
        .add_plugins(poi::plugin)
        .add_systems(
            Update,
            (spawn_map, read_reached_coordinates).run_if(in_state(AppStates::InGame)),
        );
}
