use std::collections::HashMap;

use bevy::prelude::*;
use bevy_rand::prelude::ChaCha8Rng;
use rand::RngExt;

use crate::utils::hex::{AxialCoordinates, DEFAULT_HEX_SIZE};

/// Stateless poisson disc sampler
#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub(super) struct PoissonDiscSampler {
    num_samples_before_rejection: usize,
    rng: ChaCha8Rng,
}

impl PoissonDiscSampler {
    pub fn new(num_samples_before_rejection: usize, rng: ChaCha8Rng) -> Self {
        Self {
            num_samples_before_rejection,
            rng,
        }
    }

    /// samples points
    pub(super) fn sample_points(
        &mut self,
        distance_between_points: f32,
        center: Vec2,
        sample_radius: f32,
        exclusion_radius: Option<f32>,
        occupied_points: &HashMap<AxialCoordinates, Entity>,
    ) -> Vec<Vec2> {
        let cell_size = distance_between_points / 2_f32.sqrt();
        let radius_squared = distance_between_points * distance_between_points;

        // Seed the spatial grid with already-occupied points, converted from
        // hex coordinates into the same world-space Vec2 the sampler works in.
        let mut points: Vec<Vec2> = occupied_points
            .keys()
            .map(|coordinates| {
                let world = coordinates.as_world_coordinates(DEFAULT_HEX_SIZE);
                Vec2::new(world.x, world.z)
            })
            .collect();
        let mut grid: HashMap<IVec2, Vec<usize>> = HashMap::new();
        for (index, point) in points.iter().enumerate() {
            grid.entry((*point / cell_size).as_ivec2())
                .or_default()
                .push(index);
        }

        let mut new_points = Vec::new();
        let mut spawn_points = vec![center];

        while let Some(spawn_point) = spawn_points.pop() {
            let mut candidate_accepted = false;

            for _index in 0..self.num_samples_before_rejection {
                let angle = self.rng.random::<f32>() * std::f32::consts::TAU;
                let dir = Vec2::from_angle(angle);
                let distance = self
                    .rng
                    .random_range(distance_between_points..=distance_between_points * 2.0);
                let candidate = spawn_point + dir * distance;

                if Self::is_valid(
                    candidate,
                    center,
                    sample_radius,
                    exclusion_radius,
                    cell_size,
                    radius_squared,
                    &points,
                    &grid,
                ) {
                    new_points.push(candidate);
                    spawn_points.push(candidate);

                    let index = points.len();
                    points.push(candidate);
                    grid.entry((candidate / cell_size).as_ivec2())
                        .or_default()
                        .push(index);

                    candidate_accepted = true;
                    break;
                }
            }

            if candidate_accepted {
                spawn_points.push(spawn_point);
            }
        }

        new_points
    }

    /// A candidate is valid when it falls inside `sample_radius` of `center`,
    /// outside `exclusion_radius` if one is given (e.g. the zeppelin's
    /// current field of vision), and at least `distance_between_points`
    /// (encoded as `radius_squared`) from every already-placed point.
    fn is_valid(
        candidate: Vec2,
        center: Vec2,
        sample_radius: f32,
        exclusion_radius: Option<f32>,
        cell_size: f32,
        radius_squared: f32,
        points: &[Vec2],
        grid: &HashMap<IVec2, Vec<usize>>,
    ) -> bool {
        let distance_from_center = candidate.distance(center);
        if distance_from_center > sample_radius {
            return false;
        }
        if exclusion_radius.is_some_and(|exclusion_radius| distance_from_center < exclusion_radius)
        {
            return false;
        }

        let cell = (candidate / cell_size).as_ivec2();
        for x in (cell.x - 2)..=(cell.x + 2) {
            for y in (cell.y - 2)..=(cell.y + 2) {
                let Some(indices) = grid.get(&IVec2::new(x, y)) else {
                    continue;
                };
                for &index in indices {
                    if candidate.distance_squared(points[index]) < radius_squared {
                        return false;
                    }
                }
            }
        }

        true
    }
}
