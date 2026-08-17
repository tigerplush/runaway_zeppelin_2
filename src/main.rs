use bevy::prelude::*;

use runaway_zeppelin_2::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AppPlugin)
        .run()
}
