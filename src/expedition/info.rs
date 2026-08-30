use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, expedition::ExpeditionState};

#[derive(Asset, Clone, Reflect, Resource)]
struct ExpeditionInfoAssets {
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

fn setup(assets: Res<ExpeditionInfoAssets>, mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        ImageNode {
            image: assets.background.clone(),
            ..default()
        },
        DespawnOnExit(ExpeditionState::Info),
    ));
}

pub fn plugin(app: &mut App) {
    app.load_resource::<ExpeditionInfoAssets>()
        .add_systems(OnEnter(ExpeditionState::Info), setup);
}
