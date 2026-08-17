use bevy::prelude::*;

mod in_game_time;

/// Main Plugin for the whole game. This will add all other game-relevant
/// plugins and base systems that are cross-cutting.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(in_game_time::plugin);
    }
}
