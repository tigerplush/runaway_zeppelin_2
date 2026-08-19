use bevy::prelude::*;

use crate::utils::hex::DEFAULT_HEX_SIZE;

/// Maps real-world meters onto the game's world units. Tune `meters_per_unit`
/// (or better, the `meters_per_hex` this is built from) to change the scale
/// of the whole game without having to re-derive every constant by hand.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct WorldScale {
    pub meters_per_unit: f32,
}

impl WorldScale {
    pub fn from_meters_per_hex(meters_per_hex: f32, hex_size: Vec2) -> Self {
        // Center-to-center distance between adjacent hexes, see
        // AxialCoordinates::to_world_coordinates.
        let hex_spacing = hex_size.x * 3f32.sqrt();
        Self {
            meters_per_unit: meters_per_hex / hex_spacing,
        }
    }

    pub fn units(&self, meters: f32) -> f32 {
        meters / self.meters_per_unit
    }

    pub fn meters(&self, units: f32) -> f32 {
        units * self.meters_per_unit
    }
}

impl Default for WorldScale {
    fn default() -> Self {
        Self::from_meters_per_hex(2000.0, DEFAULT_HEX_SIZE)
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<WorldScale>().register_type::<WorldScale>();
}
