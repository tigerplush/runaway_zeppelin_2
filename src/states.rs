use bevy::prelude::*;

use crate::asset_tracking::ResourceHandles;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum AppStates {
    #[default]
    Preloading,
    InGame,
}

fn advance_state(res: Res<ResourceHandles>, mut next: ResMut<NextState<AppStates>>) {
    info!("{:.2}%", res.progress() * 100.0);
    if res.is_all_done() {
        next.set(AppStates::InGame);
    }
}

pub fn plugin(app: &mut App) {
    app.init_state::<AppStates>().add_systems(
        Update,
        advance_state.run_if(in_state(AppStates::Preloading)),
    );
}
