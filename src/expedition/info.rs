use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    expedition::ExpeditionState,
    states::InGameStates,
    ui::{self, FontChoice},
};

#[derive(Asset, Clone, Reflect, Resource)]
struct ExpeditionInfoAssets {
    #[dependency]
    background: Handle<Image>,
}

impl FromWorld for ExpeditionInfoAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            background: asset_server.load("ui/graphics/screen_bg_2.png"),
        }
    }
}

#[derive(Component)]
struct Start;

#[derive(Component)]
struct Cancel;

fn setup(assets: Res<ExpeditionInfoAssets>, mut commands: Commands) {
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
        DespawnOnExit(ExpeditionState::Info),
        children![
            (
                ui::header("Expedition Name"),
                Node {
                    margin: UiRect::top(Val::Percent(5.0)),
                    ..default()
                }
            ),
            (
                Name::from("DangerMeter"),
                Text::from("Danger Meter"),
                FontChoice::Content,
            ),
            (ui::horizontal_divider()),
            (Name::from("Tagline"), Text::from("Tagline")),
            (
                (
                    Name::from("CaptainsRecommendationsHeader"),
                    Node {
                        align_items: AlignItems::Center,
                        ..default()
                    }
                ),
                children![
                    ui::left_divider(),
                    (
                        Name::from("Captains Recommendations"),
                        Text::from("Captains Recommendations"),
                        FontChoice::Content,
                    ),
                    ui::right_divider(),
                ]
            ),
            (
                Name::from("RecommendationsLabel"),
                Text::from("Recommendations"),
            ),
            (
                (
                    Name::from("PotentialRewardsHeader"),
                    Node {
                        align_items: AlignItems::Center,
                        ..default()
                    }
                ),
                children![
                    ui::left_divider(),
                    (
                        Name::from("Potential Rewards"),
                        Text::from("Potential Rewards"),
                        FontChoice::Content,
                    ),
                    ui::right_divider(),
                ]
            ),
            (
                Name::from("RewardsLabel"),
                Text::from("Rewards"),
                Node {
                    margin: UiRect::bottom(Val::Percent(5.0)),
                    ..default()
                }
            ),
            (
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
            )
        ],
    ));
}

fn on_start_expedition(
    trigger: On<Pointer<Release>>,
    start: Query<&Start>,
    mut next_state: ResMut<NextState<ExpeditionState>>,
) {
    if let Ok(_) = start.get(trigger.entity) {
        next_state.set(ExpeditionState::Preparation);
    }
}

fn on_cancel_expedition(
    trigger: On<Pointer<Release>>,
    cancel: Query<&Cancel>,
    mut next_state: ResMut<NextState<InGameStates>>,
) {
    if let Ok(_) = cancel.get(trigger.entity) {
        next_state.set(InGameStates::World);
    }
}

pub fn plugin(app: &mut App) {
    app.load_resource::<ExpeditionInfoAssets>()
        .add_systems(OnEnter(ExpeditionState::Info), setup)
        .add_observer(on_start_expedition)
        .add_observer(on_cancel_expedition);
}
