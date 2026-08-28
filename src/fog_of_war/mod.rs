use bevy::prelude::*;

mod explored;
mod post_process;

pub fn plugin(app: &mut App) {
    app.add_plugins((explored::plugin, post_process::plugin));
}