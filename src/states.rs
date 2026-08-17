use bevy::prelude::*;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum AppStates {
    #[default]
    InGame,
}

pub fn plugin(app: &mut App) {
    app.init_state::<AppStates>();
}
