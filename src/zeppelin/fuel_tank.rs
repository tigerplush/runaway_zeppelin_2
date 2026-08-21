use bevy::prelude::*;

use crate::{
    states::AppStates,
    ui::{self, ResourceSlot},
};

pub(super) struct FuelConsumptionRequest {
    pub(super) fuel_amount: f32,
    pub(super) gas_amount: f32,
}

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

    /// Tries to consume the given amount of fuel.
    /// Will fall back to liquid fuel when no gas is left, even though gas is requested.
    /// Returns a normalized float with how much of the requested fuel could be consumed
    pub(super) fn consume(&mut self, request: FuelConsumptionRequest) -> f32 {
        let total_requested_amount = request.gas_amount + request.fuel_amount;
        if total_requested_amount <= 0.0 {
            return 0.0;
        }

        let gas_amount = self.gas_amount.min(request.gas_amount);
        self.gas_amount -= gas_amount;

        let overflow = request.gas_amount - gas_amount;
        let fuel_amount = self.fuel_amount.min(request.fuel_amount + overflow);

        self.fuel_amount -= fuel_amount;

        let total_amount = gas_amount + fuel_amount;
        total_amount / (request.gas_amount + request.fuel_amount)
    }

    /// Whether the engine could draw *any* power right now. Mirrors the
    /// fallback direction in `consume`: a gas request can spill over into
    /// fuel, but a fuel request never falls back to gas, so which tank(s)
    /// count depends on which resource the engine currently wants.
    pub(super) fn can_supply(&self, gas_consumption_rate: f32) -> bool {
        if gas_consumption_rate > 0.0 {
            self.gas_amount > 0.0 || self.fuel_amount > 0.0
        } else {
            self.fuel_amount > 0.0
        }
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

#[cfg(test)]
mod test {
    use crate::zeppelin::{FuelConsumptionRequest, FuelTank};

    #[test]
    fn test_fuel_request() {
        let mut fuel_tank = FuelTank {
            fuel_amount: 1.0,
            gas_amount: 1.0,
        };

        assert_eq!(
            fuel_tank.consume(FuelConsumptionRequest {
                fuel_amount: 0.5,
                gas_amount: 0.0
            }),
            1.0
        );

        assert_eq!(
            fuel_tank.consume(FuelConsumptionRequest {
                fuel_amount: 0.0,
                gas_amount: 0.5
            }),
            1.0
        );

        assert_eq!(
            fuel_tank.consume(FuelConsumptionRequest {
                fuel_amount: 0.0,
                gas_amount: 1.0
            }),
            1.0
        );

        assert_eq!(
            fuel_tank.consume(FuelConsumptionRequest {
                fuel_amount: 0.0,
                gas_amount: 0.5
            }),
            0.0
        );
    }
}
