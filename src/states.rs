use bevy::prelude::*;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum AppStates {
    Preload,
    MainMenu,
    #[default]
    MainApp,
}
