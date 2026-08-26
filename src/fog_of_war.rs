use std::collections::HashMap;
#[cfg(debug_assertions)]
use std::f32::consts::PI;

#[cfg(debug_assertions)]
use bevy::color::palettes::css::{DARK_GRAY, GRAY, LIGHT_GRAY, WHITE};
use bevy::prelude::*;

#[cfg(debug_assertions)]
use crate::{fog_of_war, utils::gizmo_traits::DrawHexagon};
use crate::{
    states::AppStates, utils::{hex::{AxialCoordinates, DEFAULT_HEX_SIZE}, scale::WorldScale}, zeppelin::{VisibilityRange, ZeppelinWrapper},
};

#[derive(Debug, Reflect)]
enum FogState {
    Visible,
    Revealed,
}

#[derive(Default, Reflect, Resource)]
struct FogOfWar {
    revealed: HashMap<AxialCoordinates, FogState>,
    last_hex: Option<AxialCoordinates>,
}

fn update_fog_of_war(
    world_scale: Res<WorldScale>,
    mut fog_of_war: ResMut<FogOfWar>,
    zeppelin: Single<(&Transform, &VisibilityRange), With<ZeppelinWrapper>>,
) {
    let (transform, visibility_range) = zeppelin.into_inner();

    let new_coordinates =
        AxialCoordinates::from_world_coordinates(transform.translation, DEFAULT_HEX_SIZE);

    if fog_of_war.last_hex.is_none_or(|hex| hex != new_coordinates) {
        let distance = world_scale.units(visibility_range.0) as isize;
        // reset all tiles to revealed
        fog_of_war
            .revealed
            .iter_mut()
            .for_each(|(_, state)| *state = FogState::Revealed);
        for coordinate in new_coordinates.within_distance(distance) {
            fog_of_war.revealed.insert(coordinate, FogState::Visible);
        }
        fog_of_war.last_hex = Some(new_coordinates);
    }
}

#[cfg(debug_assertions)]
fn debug_fog_of_war(mut gizmos: Gizmos, fog_of_war: Res<FogOfWar>) {
    for (coordinate, state) in fog_of_war.revealed.iter() {
        let color = match state {
            FogState::Revealed => DARK_GRAY.with_alpha(0.5),
            FogState::Visible => LIGHT_GRAY.with_alpha(0.5),
        };
        gizmos.draw_hexagon(coordinate.as_world_coordinates(DEFAULT_HEX_SIZE), DEFAULT_HEX_SIZE, color);
        // gizmos.circle(
        //     Isometry3d::new(
        //         coordinate.as_world_coordinates(DEFAULT_HEX_SIZE),
        //         Quat::from_rotation_x(-PI / 2.),
        //     ),
        //     0.5,
        //     color,
        // );
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
