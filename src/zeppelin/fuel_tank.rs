use bevy::prelude::*;
use pyri_tooltip::prelude::*;

use crate::{states::AppStates, ui::{self, ResourceSlot}};

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

fn total_label(tank: &FuelTank) -> String {
    format_amount(tank.fuel_capacity + tank.gas_capacity)
}

fn tooltip_content(tank: &FuelTank) -> String {
    format!(
        "Petrol: {} L\nBlaugas: {} m3",
        format_amount(tank.fuel_capacity),
        format_amount(tank.gas_capacity),
    )
}

/// Marks the whole row (icon + amount), which is where the `Tooltip` lives -
/// hovering the icon should open the tooltip just as much as hovering the
/// number, so it can't sit only on the inner text entity.
#[derive(Component)]
struct FuelRow;

/// Marks just the amount text, so it can be updated without digging through
/// `Children` to find it.
#[derive(Component)]
struct FuelAmount;

fn setup(fuel_tank: Single<&FuelTank>, mut commands: Commands) {
    commands
        .spawn(ui::resource_label(ResourceSlot::Fuel))
        .insert((
            FuelRow,
            // pyri_tooltip's activation delay counts down on Res<Time>, which
            // mirrors Time<Virtual> - and the game starts paused, so any
            // nonzero delay would never elapse. IMMEDIATE skips that whole
            // codepath (see TooltipState::Delayed in pyri_tooltip's context.rs).
            Tooltip::cursor(tooltip_content(&fuel_tank))
                .with_activation(TooltipActivation::IMMEDIATE),
        ))
        .with_child((
            FuelAmount,
            Text(total_label(&fuel_tank)),
            // Same reason as the icon in ui::resource_label: without this,
            // hovering the number itself steals Interaction::Hovered away
            // from FuelRow, and pyri_tooltip never sees the Tooltip on it.
            Pickable::IGNORE,
        ));
}

fn update_fuel_label(
    fuel_tank: Single<&FuelTank>,
    mut amount: Single<&mut Text, With<FuelAmount>>,
    mut tooltip: Single<&mut Tooltip, With<FuelRow>>,
) {
    amount.0 = total_label(&fuel_tank);
    tooltip.content = tooltip_content(&fuel_tank).into();
}

pub fn plugin(app: &mut App) {
    app.register_type::<FuelTank>()
        .add_systems(OnEnter(AppStates::InGame), setup)
        .add_systems(Update, update_fuel_label);
}
