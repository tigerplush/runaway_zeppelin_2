use bevy::prelude::*;

use crate::utils::hex::AxialCoordinates;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Poi(pub AxialCoordinates);

