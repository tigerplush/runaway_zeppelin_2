use bevy::prelude::*;
use bevy_water::{WaterPlugin, WaterSettings};

use crate::states::AppStates;

pub fn plugin(app: &mut App) {
    app.add_plugins(WaterPlugin)
        .add_systems(OnEnter(AppStates::MainApp), setup)
        .add_systems(OnExit(AppStates::MainApp), teardown);
}

fn setup(mut commands: Commands) {
    commands.insert_resource(WaterSettings {
        height: 0.0,
        ..default()
    });

    commands.spawn((
        Camera3d::default(),
        DespawnOnExit(AppStates::MainApp),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight { ..default() },
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn teardown(mut commands: Commands) {
    commands.remove_resource::<WaterSettings>();
}
