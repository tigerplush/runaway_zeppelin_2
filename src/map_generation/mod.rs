use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::ChaCha8Rng};

use crate::{
    map_generation::{
        poi::{AvailablePois, Poi, PoiDistance, PoiMap, WorldState},
        poisson_disc_sampler::PoissonDiscSampler,
    },
    states::AppStates,
    utils::{
        hex::{AxialCoordinates, DEFAULT_HEX_SIZE},
        scale::WorldScale,
    },
    zeppelin::{
        EnteredCoordinatesMessage, ReachedCoordinatesMessage, VisibilityRadius, ZeppelinWrapper,
    },
};

mod poi;
mod poisson_disc_sampler;

fn spawn_map(
    scale: Res<WorldScale>,
    poi_distance: Res<PoiDistance>,
    mut available_pois: ResMut<AvailablePois>,
    mut poisson_disc_sampler: ResMut<PoissonDiscSampler>,
    mut poi_map: ResMut<PoiMap>,
    reader: MessageReader<EnteredCoordinatesMessage>,
    zeppelin: Single<(&Transform, &VisibilityRadius), (With<ZeppelinWrapper>, Without<Poi>)>,
    mut rng: Single<&mut ChaCha8Rng, With<GlobalRng>>,
    mut commands: Commands,
    mut first_run_done: Local<bool>,
) {
    if reader.is_empty() {
        return;
    }

    // on moved zeppelin:
    let (transform, visibility_range) = zeppelin.into_inner();
    // set origin to zeppelin
    let center = Vec2::new(transform.translation.x, transform.translation.z);
    let exclusion_radius = if *first_run_done {
        Some(scale.units(visibility_range.0))
    } else {
        None
    };
    let sample_radius = scale.units(visibility_range.0 + poi_distance.0);

    let distance_between_points = scale.units(poi_distance.0);
    // sample around zeppelin
    let p = poisson_disc_sampler.sample_points(
        distance_between_points,
        center,
        sample_radius,
        exclusion_radius,
        &poi_map.0,
    );

    for point in p {
        let coordinates = AxialCoordinates::from_world_coordinates(
            Vec3::new(point.x, 0.0, point.y),
            DEFAULT_HEX_SIZE,
        );

        let Some(_poi) = available_pois.get(&WorldState, &mut rng) else {
            continue;
        };

        let new_poi = commands.spawn(Poi(coordinates)).id();
        poi_map.0.insert(coordinates, new_poi);
    }

    *first_run_done = true;
}

fn on_add_poi_attach_visuals(
    trigger: On<Add, Poi>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&Poi>,
    mut commands: Commands,
) {
    let Ok(poi) = query.get(trigger.entity) else {
        return;
    };

    commands.entity(trigger.entity).insert((
        Mesh3d(meshes.add(Extrusion::new(RegularPolygon::default(), 0.1))),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::from_rotation(Quat::from_rotation_x(-PI / 2.))
            .with_translation(poi.0.as_world_coordinates(DEFAULT_HEX_SIZE)),
        Pickable::default(),
    ));
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
        .register_type::<PoissonDiscSampler>()
        .add_message::<ReachedPoiMessage>()
        .insert_resource(PoissonDiscSampler::new(30, ChaCha8Rng::default()))
        .add_plugins(poi::plugin)
        .add_systems(
            Update,
            (spawn_map, read_reached_coordinates).run_if(in_state(AppStates::InGame)),
        )
        .add_observer(on_add_poi_attach_visuals);
}
