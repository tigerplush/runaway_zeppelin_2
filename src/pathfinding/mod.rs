use bevy::{color::palettes::css::PURPLE, prelude::*};

mod path;
mod pathfinder;
pub use path::*;
pub use pathfinder::*;

use crate::utils::hex::DEFAULT_HEX_SIZE;

fn calculate_paths(mut query: Query<(Entity, &mut Pathfinder)>, mut commands: Commands) {
    for (entity, mut pathfinder) in &mut query {
        match pathfinder.calculate_step() {
            PathfindingState::Calculating => (), // still calculating, do nothing
            PathfindingState::Completed(items) => {
                commands
                    .entity(entity)
                    .remove::<Pathfinder>()
                    .insert(Path::new(items));
            }
            PathfindingState::Failed => todo!(), // handle gracefully,
        }
    }
}

#[cfg(debug_assertions)]
fn debug_path(mut gizmos: Gizmos, query: Query<&Path>) {
    for path in &query {
        for point in path.points().windows(2) {
            let lhs = point[0].as_world_coordinates(DEFAULT_HEX_SIZE);
            let rhs = point[1].as_world_coordinates(DEFAULT_HEX_SIZE);
            gizmos.arrow(lhs, rhs, PURPLE);
        }
    }
}

pub fn plugin(app: &mut App) {
    app.register_type::<Pathfinder>()
        .register_type::<Path>()
        .add_systems(Update, calculate_paths);

    #[cfg(debug_assertions)]
    app.add_systems(Update, debug_path);
}
