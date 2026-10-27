//! This module is responsible for handling expeditions.

use bevy::prelude::*;
use bevy_yarnspinner::prelude::*;
use bevy_yarnspinner_example_dialogue_view::ExampleYarnSpinnerDialogueViewPlugin;

use crate::{
    asset_tracking::LoadResource,
    in_game_time::{InGameTime, TimeControlMessage},
    map_generation::ReachedPoiMessage,
    states::InGameStates,
    ui::{AttachToUiSlot, primary_button},
};

mod actual;
mod info;
mod preparation;

#[derive(Asset, Clone, Reflect, Resource)]
struct ExpeditionAssets {
    #[dependency]
    background: Handle<Image>,
}

impl FromWorld for ExpeditionAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            background: asset_server.load("ui/graphics/screen_bg_2.png"),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, SubStates)]
#[source(InGameStates = InGameStates::Expedition)]
enum ExpeditionState {
    #[default]
    Info,
    Preparation,
    Actual,
    Aftermath,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct ExpeditionInQuestion(Entity);

fn on_start_expedition(
    _trigger: On<PointerRelease>,
    mut next_state: ResMut<NextState<InGameStates>>,
) {
    next_state.set(InGameStates::Expedition);
}

fn show_expedition_button(mut reader: MessageReader<ReachedPoiMessage>, mut commands: Commands) {
    for ev in reader.read() {
        info!("ev: {}", ev.0);
        commands
            .spawn((
                primary_button("Start Expedition", None),
                AttachToUiSlot::Action,
            ))
            .observe(on_start_expedition);

        commands.insert_resource(ExpeditionInQuestion(ev.0));
    }
}

fn disable_time_controls(mut in_game_time: ResMut<InGameTime>, mut commands: Commands) {
    in_game_time.pause();
    commands.write_message(TimeControlMessage::Disable);
}

fn enable_time_controls(mut in_game_time: ResMut<InGameTime>, mut commands: Commands) {
    in_game_time.unpause();
    commands.write_message(TimeControlMessage::Enable);
}

pub fn plugin(app: &mut App) {
    app.register_type::<ExpeditionInQuestion>()
        .load_resource::<ExpeditionAssets>()
        .add_sub_state::<ExpeditionState>()
        .add_plugins((
            YarnSpinnerPlugin::new(),
            ExampleYarnSpinnerDialogueViewPlugin::new(),
        ))
        .add_plugins((actual::plugin, info::plugin, preparation::plugin))
        .add_systems(OnEnter(InGameStates::Expedition), disable_time_controls)
        .add_systems(Update, show_expedition_button)
        .add_systems(OnExit(InGameStates::Expedition), enable_time_controls);
}
