use bevy::prelude::*;

use crate::utils::hex::AxialCoordinates;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Path {
    points: Vec<AxialCoordinates>,
}

impl Path {
    pub fn new(points: Vec<AxialCoordinates>) -> Self {
        Self { points }
    }

    pub fn points(&self) -> &Vec<AxialCoordinates> {
        &self.points
    }
}
