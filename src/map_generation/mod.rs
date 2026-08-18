use std::{
    f32::consts::PI,
    ops::{Index, IndexMut},
};

use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::ChaCha8Rng};
use rand::RngExt;

use crate::utils::hex::{AxialCoordinates, DEFAULT_HEX_SIZE};

struct Grid {
    contents: Vec<Option<usize>>,
    width: usize,
    height: usize,
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        Self {
            contents: vec![None; width * height],
            width,
            height,
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }
}

impl Index<(usize, usize)> for Grid {
    type Output = Option<usize>;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let index = self.index(index.0, index.1);
        &self.contents[index]
    }
}

impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let index = self.index(index.0, index.1);
        &mut self.contents[index]
    }
}

impl Index<usize> for Grid {
    type Output = Option<usize>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.contents[index]
    }
}

impl IndexMut<usize> for Grid {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.contents[index]
    }
}

fn sample_poisson_disc(
    radius: f32,
    sample_region_size: Vec2,
    num_samples_before_rejection: usize,
    rng: &mut ChaCha8Rng,
) -> Vec<Vec2> {
    let cell_size = radius / 2_f32.sqrt();

    let width = (sample_region_size.x / cell_size).ceil() as usize;
    let height = (sample_region_size.y / cell_size).ceil() as usize;
    let mut grid = Grid::new(width, height);
    let mut points = Vec::new();
    let mut spawn_points = Vec::new();
    let starting_point = rng.random::<Vec2>();
    spawn_points.push(starting_point);
    while let Some(spawn_centre) = spawn_points.pop() {
        let mut candidate_accepted = false;
        for _index in 0..num_samples_before_rejection {
            let angle = rng.random::<f32>() * 2.0 * PI;
            let dir = Vec2::new(angle.sin(), angle.cos());
            let distance = rng.random_range(radius..=radius * 2.0);
            let candidate = spawn_centre + dir * distance;
            if is_valid(
                &candidate,
                &sample_region_size,
                cell_size,
                &points,
                &grid,
                radius,
            ) {
                points.push(candidate.clone());
                spawn_points.push(candidate.clone());

                let x = (candidate.x / cell_size) as usize;
                let y = (candidate.y / cell_size) as usize;
                grid[(x, y)] = Some(points.len() - 1);
                candidate_accepted = true;
                break;
            }
        }

        if candidate_accepted {
            spawn_points.push(spawn_centre);
        }
    }

    points
}

fn is_valid(
    candidate: &Vec2,
    sample_region_size: &Vec2,
    cell_size: f32,
    points: &Vec<Vec2>,
    grid: &Grid,
    radius: f32,
) -> bool {
    if candidate.x >= 0.0
        && candidate.x < sample_region_size.x
        && candidate.y >= 0.0
        && candidate.y < sample_region_size.y
    {
        let cell_x = (candidate.x / cell_size) as isize;
        let cell_y = (candidate.y / cell_size) as isize;
        let x_min = (cell_x - 2).max(0) as usize;
        let x_max = (cell_x + 2).min((grid.width - 1) as isize) as usize;
        let y_min = (cell_y - 2).max(0) as usize;
        let y_max = (cell_y + 2).min((grid.height - 1) as isize) as usize;
        for x in x_min..=x_max {
            for y in y_min..=y_max {
                if let Some(point_index) = grid[(x, y)] {
                    let distance_squared = (candidate - points[point_index]).length_squared();
                    if distance_squared < radius * radius {
                        return false;
                    }
                }
            }
        }
        return true;
    }
    false
}

fn spawn_map(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rng: Single<&mut ChaCha8Rng, With<GlobalRng>>,
    mut commands: Commands,
) {
    let p = sample_poisson_disc(5.0, Vec2::new(20.0, 20.0), 30, &mut rng);

    for point in p {
        let translation = AxialCoordinates::from_world_coordinates(
            Vec3::new(point.x, 0.0, point.y),
            DEFAULT_HEX_SIZE,
        )
        .to_world_coordinates(DEFAULT_HEX_SIZE);

        commands.spawn((
            Mesh3d(meshes.add(Extrusion::new(RegularPolygon::default(), 0.1))),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
            Transform::from_rotation(Quat::from_rotation_x(-PI / 2.)).with_translation(translation),
        ));
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_map);
}
