use bevy::prelude::*;

mod asset_tracking;

use asset_tracking::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ResourceHandles>()
        .add_systems(PreUpdate, asset_tracking::load_resource_assets);
}
