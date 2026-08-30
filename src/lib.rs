use bevy::{color::palettes::tailwind::BLUE_100, prelude::*};
use bevy_rand::{plugin::EntropyPlugin, prelude::ChaCha8Rng};

mod asset_tracking;
mod camera;
mod expedition;
mod fog_of_war;
mod in_game_time;
mod input;
mod map_generation;
mod pathfinding;
mod pointer;
mod states;
mod ui;
mod utils;
mod zeppelin;

fn spawn_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_plane(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commmands: Commands,
) {
    commmands.spawn((
        Mesh3d(meshes.add(Plane3d::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: BLUE_100.into(),
            ..default()
        })),
        Transform::from_scale(Vec3::splat(300.0)),
    ));
}

/// Main Plugin for the whole game. This will add all other game-relevant
/// plugins and base systems that are cross-cutting.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EntropyPlugin::<ChaCha8Rng>::default())
            .add_plugins((
                asset_tracking::plugin,
                camera::plugin,
                expedition::plugin,
                fog_of_war::plugin,
                in_game_time::plugin,
                input::plugin,
                map_generation::plugin,
                pathfinding::plugin,
                pointer::plugin,
                states::plugin,
                ui::plugin,
                utils::plugin,
                zeppelin::plugin,
            ))
            .add_systems(Startup, (spawn_light, spawn_plane));
    }
}
