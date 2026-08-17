use bevy::prelude::*;

mod camera;
mod in_game_time;
mod input;
mod map_generation;
mod states;
mod utils;

fn spawn_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Main Plugin for the whole game. This will add all other game-relevant
/// plugins and base systems that are cross-cutting.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            camera::plugin,
            in_game_time::plugin,
            input::plugin,
            map_generation::plugin,
            states::plugin,
        ))
        .add_systems(Startup, spawn_light);
    }
}
