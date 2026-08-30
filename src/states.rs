use bevy::prelude::*;

use crate::asset_tracking::ResourceHandles;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum AppStates {
    #[default]
    Preloading,
    InGame,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, SubStates)]
#[source(AppStates = AppStates::InGame)]
pub enum InGameStates {
    #[default]
    World,
    Expedition,
}

fn advance_state(res: Res<ResourceHandles>, mut next: ResMut<NextState<AppStates>>) {
    info!("{:.2}%", res.progress() * 100.0);
    if res.is_all_done() {
        next.set(AppStates::InGame);
    }
}

pub fn plugin(app: &mut App) {
    app.init_state::<AppStates>()
        .add_sub_state::<InGameStates>()
        .add_systems(
            Update,
            advance_state.run_if(in_state(AppStates::Preloading)),
        );
}
