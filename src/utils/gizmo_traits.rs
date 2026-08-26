use bevy::prelude::*;

pub trait DrawHexagon {
    fn draw_hexagon(&mut self, center: Vec3, size: Vec2, color: impl Into<Color>);
}

impl<'w, 's> DrawHexagon for Gizmos<'w, 's> {
    fn draw_hexagon(&mut self, center: Vec3, size: Vec2, color: impl Into<Color>) {
        let color = color.into();
        // Pointy-top corners, matching AxialCoordinates::to_world_coordinates's
        // orientation: angles offset by -30 deg from the axes.
        let corner = |i: u32| {
            let angle = (60.0 * i as f32 - 30.0).to_radians();
            center + Vec3::new(size.x * angle.cos(), 0.0, size.y * angle.sin())
        };
        // 0..=6 rather than 0..6 so the linestrip closes back onto the first corner.
        let points = (0..=6).map(corner);
        self.linestrip(points, color);
    }
}
