use bevy::prelude::*;

use crate::{
    expedition::{ExpeditionAssets, ExpeditionState},
    states::InGameStates,
    ui,
};

#[derive(Component)]
struct Start;

#[derive(Component)]
struct Cancel;

fn setup(assets: Res<ExpeditionAssets>, mut commands: Commands) {
    commands.spawn((
        Name::from("ExpeditionInfo"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        ImageNode {
            image: assets.background.clone(),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        DespawnOnExit(ExpeditionState::Preparation),
        children![(
            Node {
                width: Val::Percent(100.0),
                ..default()
            },
            children![(
                (Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::all(Val::Percent(1.0)),
                    ..default()
                }),
                children![
                    // ghost node so the buttons are aligned properly
                    (ui::primary_button("Cancel", None), Visibility::Hidden),
                    (ui::primary_button("Start Expedition", None), Start),
                    (ui::primary_button("Cancel", None), Cancel),
                ]
            )]
        )],
    ));
}

fn on_start_expedition(
    trigger: On<PointerRelease>,
    start: Query<&Start>,
    mut next_state: ResMut<NextState<ExpeditionState>>,
) {
    if let Ok(_) = start.get(trigger.entity) {
        next_state.set(ExpeditionState::Actual);
    }
}

fn on_cancel_preparation(
    trigger: On<PointerRelease>,
    cancel: Query<&Cancel>,
    mut next_state: ResMut<NextState<InGameStates>>,
) {
    if let Ok(_) = cancel.get(trigger.entity) {
        next_state.set(InGameStates::World);
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(ExpeditionState::Preparation), setup)
        .add_observer(on_start_expedition)
        .add_observer(on_cancel_preparation);
}
