use bevy::prelude::*;

// use Vec3 as WorldCoordinates;
type WorldCoordinates = Vec3;

pub const SIZE: f32 = 300.0;

pub struct AxialCoordinates {
    pub q: isize,
    pub r: isize,
}

impl AxialCoordinates {
    pub fn from_world_coordinates(world_coordinates: WorldCoordinates) -> Self {
        let q = (3_f32.sqrt() / 3.0 * world_coordinates.x - 1.0 / 3.0 * world_coordinates.z) / SIZE;
        let r = (2.0 / 3.0 * world_coordinates.z) / SIZE;
        let s = -q - r;
        let rounded_cube = CubeCoordinates::round((q, r, s).into());
        rounded_cube.into()
    }

    pub fn to_world_coordinates(&self) -> WorldCoordinates {
        let x = SIZE * (3_f32.sqrt() * self.q as f32 + 3_f32.sqrt() / 2.0 * self.r as f32);
        let z = SIZE * (3.0 / 2.0 * self.r as f32);
        Vec3::new(x, 0.0, z)
    }
}

impl From<CubeCoordinates> for AxialCoordinates {
    fn from(value: CubeCoordinates) -> Self {
        AxialCoordinates {
            q: value.q,
            r: value.r,
        }
    }
}

struct CubeCoordinates {
    q: isize,
    r: isize,
    s: isize,
}

impl CubeCoordinates {
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

        CubeCoordinates {
            q: q as isize,
            r: r as isize,
            s: s as isize,
        }
    }
}
