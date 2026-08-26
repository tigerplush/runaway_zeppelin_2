use std::collections::HashMap;

#[cfg(debug_assertions)]
use bevy::color::palettes::css::{DARK_GRAY, LIGHT_GRAY};
use bevy::prelude::*;

#[cfg(debug_assertions)]
use crate::utils::gizmo_traits::DrawHexagon;
use crate::{
    states::AppStates,
    utils::{
        hex::{AxialCoordinates, DEFAULT_HEX_SIZE},
        scale::WorldScale,
    },
    zeppelin::{EnteredCoordinatesMessage, VisibilityRange, ZeppelinWrapper},
};

#[derive(Debug, Reflect)]
enum FogState {
    Visible,
    Revealed,
}

#[derive(Default, Reflect, Resource)]
struct FogOfWar {
    revealed: HashMap<AxialCoordinates, FogState>,
}

fn update_fog_of_war(
    world_scale: Res<WorldScale>,
    mut fog_of_war: ResMut<FogOfWar>,
    mut reader: MessageReader<EnteredCoordinatesMessage>,
    visibility_range: Single<&VisibilityRange, With<ZeppelinWrapper>>,
) {
    for ev in reader.read() {
        let distance = world_scale.units(visibility_range.0) as isize;
        // reset all tiles to revealed
        fog_of_war
            .revealed
            .iter_mut()
            .for_each(|(_, state)| *state = FogState::Revealed);
        for coordinate in ev.0.within_distance(distance) {
            fog_of_war.revealed.insert(coordinate, FogState::Visible);
        }
    }
}

#[cfg(debug_assertions)]
fn debug_fog_of_war(mut gizmos: Gizmos, fog_of_war: Res<FogOfWar>) {
    for (coordinate, state) in fog_of_war.revealed.iter() {
        let color = match state {
            FogState::Revealed => DARK_GRAY.with_alpha(0.5),
            FogState::Visible => LIGHT_GRAY.with_alpha(0.5),
        };
        gizmos.draw_hexagon(
            coordinate.as_world_coordinates(DEFAULT_HEX_SIZE),
            DEFAULT_HEX_SIZE,
            color,
        );
    }
}

pub fn plugin(app: &mut App) {
    app.register_type::<FogOfWar>()
        .init_resource::<FogOfWar>()
        .add_systems(
            Update,
            update_fog_of_war.run_if(in_state(AppStates::InGame)),
        );

    #[cfg(debug_assertions)]
    app.add_systems(Update, debug_fog_of_war);
}
