use bevy::prelude::*;

use crate::{expedition::ExpeditionState, in_game_time::{GameSpeed::Speedx1, InGameTime}};

/// On entering the actual expedition state, we should
/// - set time to SpeedX1
/// - show an animation either for landing or for mooring
/// - on landing:
///     - show yarnspinner dialogue
///     - pause game
fn setup(mut in_game_time: ResMut<InGameTime>) {
    in_game_time.set_speed(Speedx1);
}

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(ExpeditionState::Actual), setup);
}