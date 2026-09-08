use bevy::prelude::*;

use crate::expedition::ExpeditionState;

/// On entering the actual expedition state, we should
/// - set time to SpeedX1
/// - show an animation either for landing or for mooring
/// - on landing:
///     - show yarnspinner dialogue
///     - pause game
fn setup() {
    
}

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(ExpeditionState::Actual), setup);
}