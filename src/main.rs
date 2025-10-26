use bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(runaway_zeppelin_2::AppPlugin)
        .run()
}
