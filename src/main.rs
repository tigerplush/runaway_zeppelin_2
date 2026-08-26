use bevy::prelude::*;

#[cfg(debug_assertions)]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use runaway_zeppelin_2::*;

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins(AppPlugin);

    #[cfg(debug_assertions)]
    app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()));

    app.run()
}
