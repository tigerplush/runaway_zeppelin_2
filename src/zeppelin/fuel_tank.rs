use bevy::prelude::*;

use crate::{
    states::AppStates,
    ui::{self, ResourceSlot},
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(super) struct FuelTank {
    /// in L
    pub(super) fuel_amount: f32,
    /// in m³
    pub(super) gas_amount: f32,
}

impl FuelTank {
    fn total_amount(&self) -> f32 {
        self.fuel_amount + self.gas_amount
    }
}

impl Default for FuelTank {
    fn default() -> Self {
        Self {
            fuel_amount: 8_000.0,
            gas_amount: 30_000.0,
        }
    }
}

/// Inserts commas... actually periods, to match the existing European-style
/// UI number formatting (e.g. "38.000" rather than "38,000" or "38000").
fn format_amount(value: f32) -> String {
    let value = value.round().max(0.0) as i64;
    let digits = value.to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().rev().enumerate() {
        if index != 0 && index % 3 == 0 {
            grouped.push('.');
        }
        grouped.push(digit);
    }
    grouped.chars().rev().collect()
}

#[derive(Component)]
enum FuelType {
    Total,
    Fuel,
    Gas,
}

fn setup(fuel_tank: Single<&FuelTank>, mut commands: Commands) {
    let parent = commands
        .spawn(ui::resource_label(
            format!("{}", fuel_tank.total_amount()),
            FuelType::Total,
            ResourceSlot::Fuel,
        ))
        .id();
    commands.spawn(ui::tooltip(
        parent,
        children![
            ui::labeled_resource_row(
                "Fuel",
                format!("{} L", format_amount(fuel_tank.fuel_amount)),
                FuelType::Fuel
            ),
            ui::labeled_resource_row(
                "Blaugas",
                format!("{} m³", format_amount(fuel_tank.gas_amount)),
                FuelType::Gas
            ),
        ],
    ));
}

fn update_fuel_label(fuel_tank: Single<&FuelTank>, mut amount: Query<(&FuelType, &mut Text)>) {
    for (fuel_type, mut text) in &mut amount {
        match fuel_type {
            FuelType::Total => text.0 = format_amount(fuel_tank.total_amount()),
            FuelType::Fuel => text.0 = format!("{} L", format_amount(fuel_tank.fuel_amount)),
            FuelType::Gas => text.0 = format!("{} m³", format_amount(fuel_tank.gas_amount)),
        }
    }
}

pub fn plugin(app: &mut App) {
    app.register_type::<FuelTank>()
        .add_systems(OnEnter(AppStates::InGame), setup)
        .add_systems(Update, update_fuel_label);
}
