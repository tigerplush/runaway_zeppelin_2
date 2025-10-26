use bevy::prelude::*;

mod assets;
mod npc;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((assets::plugin, npc::plugin));
    }
}
