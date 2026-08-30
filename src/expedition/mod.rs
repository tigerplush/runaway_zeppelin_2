//! This module is responsible for handling expeditions.

use bevy::prelude::*;
use bevy_yarnspinner::prelude::*;
use bevy_yarnspinner_example_dialogue_view::ExampleYarnSpinnerDialogueViewPlugin;

use crate::{
    map_generation::ReachedPoiMessage,
    states::InGameStates,
    ui::{AttachToUiSlot, primary_button},
};

mod info;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, SubStates)]
#[source(InGameStates = InGameStates::Expedition)]
enum ExpeditionState {
    #[default]
    Info,
    Preparation,
    Actual,
    Aftermath
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct ExpeditionInQuestion(Entity);

fn on_start_expedition(
    _trigger: On<Pointer<Release>>,
    mut next_state: ResMut<NextState<InGameStates>>,
) {
    next_state.set(InGameStates::Expedition);
}

fn show_expedition_button(mut reader: MessageReader<ReachedPoiMessage>, mut commands: Commands) {
    for ev in reader.read() {
        info!("ev: {}", ev.0);
        commands
            .spawn(primary_button(
                "Start Expedition",
                None,
                AttachToUiSlot::Action,
            ))
            .observe(on_start_expedition);

        commands.insert_resource(ExpeditionInQuestion(ev.0));
    }
}

pub fn plugin(app: &mut App) {
    app.register_type::<ExpeditionInQuestion>()
        .add_sub_state::<ExpeditionState>()
        .add_plugins((
            YarnSpinnerPlugin::new(),
            ExampleYarnSpinnerDialogueViewPlugin::new(),
        ))
        .add_plugins(info::plugin)
        .add_systems(Update, show_expedition_button);
}
