use bevy::prelude::*;

mod explored;

pub fn plugin(app: &mut App) {
    app.add_plugins(explored::plugin);
}