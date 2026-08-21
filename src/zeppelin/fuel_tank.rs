use bevy::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(super) struct FuelTank {
    /// in kg
    pub(super) fuel_capacity: f32,
    /// in m³
    pub(super) gas_capacity: f32,
}

impl Default for FuelTank {
    fn default() -> Self {
        Self {
            fuel_capacity: 8_000.0,
            gas_capacity: 30_000.0,
        }
    }
}

pub fn plugin(app: &mut App) {
    app.register_type::<FuelTank>();
}