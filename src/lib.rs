use bevy::prelude::*;

/// Main Plugin for the whole game. This will add all other game-relevant
/// plugins and base systems that are cross-cutting.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
    }
}
