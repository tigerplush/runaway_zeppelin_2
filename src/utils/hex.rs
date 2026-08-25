use std::ops::{Add, Sub};

use bevy::prelude::*;

// use Vec3 as WorldCoordinates;
type WorldCoordinates = Vec3;

pub const DEFAULT_HEX_SIZE: Vec2 = Vec2::new(0.5, 0.5);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
pub struct AxialCoordinates {
    pub q: isize,
    pub r: isize,
}

impl AxialCoordinates {
    pub const UPPER_LEFT: AxialCoordinates = AxialCoordinates::new(0, -1);
    pub const UPPER_RIGHT: AxialCoordinates = AxialCoordinates::new(1, -1);
    pub const RIGHT: AxialCoordinates = AxialCoordinates::new(1, 0);
    pub const BOTTOM_RIGHT: AxialCoordinates = AxialCoordinates::new(0, 1);
    pub const BOTTOM_LEFT: AxialCoordinates = AxialCoordinates::new(-1, 1);
    pub const LEFT: AxialCoordinates = AxialCoordinates::new(-1, 0);

    pub const ZERO: AxialCoordinates = AxialCoordinates::new(0, 0);

    pub const DIRECTIONS: [AxialCoordinates; 6] = [
        Self::UPPER_LEFT,
        Self::UPPER_RIGHT,
        Self::RIGHT,
        Self::BOTTOM_RIGHT,
        Self::BOTTOM_LEFT,
        Self::LEFT,
    ];

    pub const fn new(q: isize, r: isize) -> Self {
        Self { q, r }
    }

    pub fn from_world_coordinates(
        world_coordinates: WorldCoordinates,
        size: impl Into<Vec2>,
    ) -> Self {
        let size = size.into();
        let q =
            (3_f32.sqrt() / 3.0 * world_coordinates.x - 1.0 / 3.0 * world_coordinates.z) / size.x;
        let r = (2.0 / 3.0 * world_coordinates.z) / size.y;
        let s = -q - r;
        let rounded_cube = ICubeCoordinates::round((q, r, s).into());
        rounded_cube.into()
    }

    pub fn as_world_coordinates(&self, size: impl Into<Vec2>) -> WorldCoordinates {
        let size = size.into();
        let x = size.x * (3_f32.sqrt() * self.q as f32 + 3_f32.sqrt() / 2.0 * self.r as f32);
        let z = size.y * (3.0 / 2.0 * self.r as f32);
        Vec3::new(x, 0.0, z)
    }

    pub fn neighbors(&self) -> Vec<AxialCoordinates> {
        Self::DIRECTIONS.iter().map(|&e| e + *self).collect()
    }

    pub fn distance(&self, rhs: &AxialCoordinates) -> usize {
        let lhs: ICubeCoordinates = self.into();
        let rhs: ICubeCoordinates = rhs.into();
        lhs.distance(&rhs)
    }
}

impl Add for AxialCoordinates {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.q + rhs.q, self.r + rhs.r)
    }
}

impl From<ICubeCoordinates> for AxialCoordinates {
    fn from(value: ICubeCoordinates) -> Self {
        AxialCoordinates {
            q: value.q,
            r: value.r,
        }
    }
}

#[derive(Clone, Copy, Reflect)]
pub struct ICubeCoordinates {
    q: isize,
    r: isize,
    s: isize,
}

impl ICubeCoordinates {
    fn round(frac: Vec3) -> Self {
        let mut q = frac.x.round();
        let mut r = frac.y.round();
        let mut s = frac.z.round();

        let q_diff = (q - frac.x).abs();
        let r_diff = (r - frac.y).abs();
        let s_diff = (s - frac.z).abs();

        if q_diff > r_diff && q_diff > s_diff {
            q = -r - s;
        } else if r_diff > s_diff {
            r = -q - s;
        } else {
            s = -q - r;
        }

        ICubeCoordinates {
            q: q as isize,
            r: r as isize,
            s: s as isize,
        }
    }

    fn distance(&self, rhs: &ICubeCoordinates) -> usize {
        let diff = self - rhs;
        (diff.q.unsigned_abs() + diff.r.unsigned_abs() + diff.s.unsigned_abs()) / 2
    }
}

impl From<&AxialCoordinates> for ICubeCoordinates {
    fn from(value: &AxialCoordinates) -> Self {
        Self {
            q: value.q,
            r: value.r,
            s: -value.q - value.r,
        }
    }
}

impl Sub<&ICubeCoordinates> for &ICubeCoordinates {
    type Output = ICubeCoordinates;
    fn sub(self, rhs: &ICubeCoordinates) -> Self::Output {
        Self::Output {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
            s: self.s - rhs.s,
        }
    }
}
