use bevy::prelude::*;

use crate::{in_game_time::InGameTime, states::AppStates};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppStates::MainApp), setup).add_systems(Update, update_time.run_if(in_state(AppStates::MainApp)));
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            Name::new("Ui Root"),
            DespawnOnExit(AppStates::MainApp),
        ))
        .with_children(|ui| {
            ui.spawn((Text::new("DateTime"), InGameTimeMarker));
        });
}

#[derive(Component)]
struct InGameTimeMarker;

fn update_time(in_game_time: Single<&InGameTime>, mut text: Single<&mut Text, With<InGameTimeMarker>>) {
    text.0 = in_game_time.to_string();
}
