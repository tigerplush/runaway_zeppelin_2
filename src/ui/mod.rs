use bevy::prelude::*;

#[derive(Component)]
pub struct UiRoot;

pub struct FontHandle {
    header: Handle<Font>,
    text: Handle<Font>,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        UiRoot,
        Name::from("UiRoot"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        }
    ));
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);
}