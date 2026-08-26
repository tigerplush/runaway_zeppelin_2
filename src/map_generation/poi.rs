use std::{collections::HashMap, f32::consts::PI};

#[cfg(debug_assertions)]
use bevy::picking::hover::{Hovered, PickingInteraction};
use bevy::{asset::LoadedFolder, color::palettes::css::GREEN, prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_rand::prelude::ChaCha8Rng;
use rand::RngExt;
use serde::Deserialize;

use crate::{
    asset_tracking::LoadResource,
    states::AppStates,
    utils::{hex::AxialCoordinates, scale::WorldScale},
    zeppelin::{VisibilityRadius, ZeppelinWrapper},
};

pub(super) struct WorldState;

#[derive(Asset, Clone, Copy, Deserialize, Reflect)]
pub(super) struct PoiContent {
    /// How often can the Poi be reused?
    ///
    /// None means it can be reused infinitely, Some(n) means it can be reused n
    /// times
    remove_after_spawns: Option<usize>,
}

impl PoiContent {
    fn is_valid(&self, _world_state: &WorldState) -> bool {
        true
    }
}

const POI_ASSET_FOLDER: &str = "poi";

/// Tracks the handle to the `poi` asset folder so [`LoadResource`] can gate
/// [`AppStates::InGame`] on every [`PoiContent`] inside it being loaded.
#[derive(Asset, Clone, Reflect, Resource)]
#[reflect(Resource)]
struct PoiFolder {
    #[dependency]
    handle: Handle<LoadedFolder>,
}

impl FromWorld for PoiFolder {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            handle: asset_server.load_folder(POI_ASSET_FOLDER),
        }
    }
}

#[derive(Default, Reflect, Resource)]
#[reflect(Resource)]
pub(super) struct AvailablePois(Vec<PoiContent>);

/// Reads every [`PoiContent`] out of the now fully-loaded `poi` folder and
/// turns it into the [`AvailablePois`] resource. Runs once [`PoiFolder`] has
/// been inserted by [`LoadResource`], which only happens after the folder and
/// all the assets in it have finished loading.
fn build_available_pois(
    folder: Res<PoiFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    poi_contents: Res<Assets<PoiContent>>,
    mut commands: Commands,
) {
    let loaded_folder = loaded_folders
        .get(&folder.handle)
        .expect("poi folder should be loaded by the time build_available_pois runs");

    let pois = loaded_folder
        .handles
        .iter()
        .filter_map(|handle| handle.clone().try_typed::<PoiContent>().ok())
        .filter_map(|handle| poi_contents.get(&handle))
        .cloned()
        .collect::<Vec<PoiContent>>();
    debug!("Loaded {} POIs", pois.len());
    commands.insert_resource(AvailablePois(pois));
}

impl AvailablePois {
    pub(super) fn get(
        &mut self,
        _world_state: &WorldState,
        rng: &mut ChaCha8Rng,
    ) -> Option<PoiContent> {
        // Fetch all allowed indices by enumerating over all pois, then filtering
        // them vor valid state
        let valid_poi_indices = self
            .0
            .iter()
            .enumerate()
            .filter(|&(_, poi)| poi.is_valid(_world_state))
            .map(|(index, _)| index)
            .collect::<Vec<usize>>();

        // if there is no valid index, we cannot spawn another poi
        // this should never happen
        if valid_poi_indices.is_empty() {
            return None;
        }
        let random_index_into_indices = rng.random_range(0..valid_poi_indices.len());
        let random_index = valid_poi_indices[random_index_into_indices];
        let poi = self.0.remove(random_index);

        let reuse_poi = match poi.remove_after_spawns {
            Some(mut uses_left) => {
                uses_left -= 1;
                uses_left > 0
            }
            None => true,
        };

        if reuse_poi {
            self.0.push(poi);
        }

        Some(poi)
    }
}

/// Represents a Point Of Interest on a map
///
/// Points of interest can be of type Landing
/// or Mooring.
/// They contain events.
/// They contain information a player is shown in a tooltip.
/// They contain how they are rendered.
/// They are pulled from a list of available POIs.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Poi(pub AxialCoordinates);

/// Distance between POIs in m.
#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub(super) struct PoiDistance(pub(super) f32);

impl Default for PoiDistance {
    fn default() -> Self {
        Self(50_000f32)
    }
}

#[cfg(debug_assertions)]
fn debug_poi_distance(
    poi_distance: Res<PoiDistance>,
    world_scale: Res<WorldScale>,
    mut gizmos: Gizmos,
    pois: Query<(&Transform, &PickingInteraction), With<Poi>>,
) {
    let distance = world_scale.units(poi_distance.0);
    for (transform, _) in pois
        .iter()
        .filter(|&(_, p)| PickingInteraction::Hovered == *p)
    {
        gizmos.circle(
            Isometry3d::new(transform.translation, Quat::from_rotation_x(-PI / 2.)),
            distance,
            GREEN,
        );
    }
}

#[cfg(debug_assertions)]
fn debug_spawn_distances(
    world_scale: Res<WorldScale>,
    visibility: Single<(&Transform, &VisibilityRadius), With<ZeppelinWrapper>>,
    mut gizmos: Gizmos,
) {
    use bevy::color::palettes::css::RED;

    let (transform, visibility) = visibility.into_inner();
    let distance = world_scale.units(visibility.0);
    gizmos.circle(
        Isometry3d::new(transform.translation, Quat::from_rotation_x(-PI / 2.)),
        distance,
        RED,
    );
    gizmos.circle(
        Isometry3d::new(transform.translation, Quat::from_rotation_x(-PI / 2.)),
        distance * 2.0,
        GREEN,
    );
}

#[derive(Default, Reflect, Resource)]
#[reflect(Resource)]
pub(super) struct PoiMap(pub(super) HashMap<AxialCoordinates, Entity>);

pub(super) fn plugin(app: &mut App) {
    app.register_type::<PoiFolder>()
        .register_type::<AvailablePois>()
        .load_resource::<PoiFolder>()
        .init_resource::<PoiDistance>()
        .init_resource::<PoiMap>()
        .add_plugins(RonAssetPlugin::<PoiContent>::new(&["poi.ron"]))
        .add_systems(OnExit(AppStates::Preloading), build_available_pois);

    #[cfg(debug_assertions)]
    app.add_systems(Update, (debug_spawn_distances, debug_poi_distance));
}
