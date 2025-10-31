use bevy::prelude::*;

use crate::states::AppStates;

mod assets;
mod hex;
mod in_game_time;
mod main_app;
mod npc;
mod states;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppStates>().add_plugins((
            assets::plugin,
            in_game_time::plugin,
            main_app::plugin,
            npc::plugin,
        ));
    }
}
