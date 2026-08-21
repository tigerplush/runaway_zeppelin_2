use bevy::prelude::*;

#[derive(Clone, Component, Copy, Reflect)]
#[reflect(Component)]
pub(super) struct ZeppelinPath {
    start: Vec3,
    pub(super) target: Vec3,
    pub(super) center: Vec3,
    pub(super) radius: f32,
    pub(super) turn_left: bool,
    pub(super) tangent_point: Vec3,
    pub(super) sweep: f32,
    pub(super) arc_length: f32,
    pub(super) straight_length: f32,
    pub(super) distance_traveled: f32,
}

impl ZeppelinPath {
    pub(super) fn new(start: Vec3, heading: Vec3, target: Vec3, radius: f32) -> Result<Self, ()> {
        let to_target = target - start;
        let turn_left = heading.x * to_target.z - heading.z * to_target.x > 0.0;

        let side_normal = if turn_left {
            Vec3::new(-heading.z, 0.0, heading.x)
        } else {
            Vec3::new(heading.z, 0.0, -heading.x)
        };
        let center = start + side_normal * radius;

        let center_to_target = target - center;
        let d = center_to_target.length();
        if d < radius {
            return Err(()); // target's inside the turning circle, no CS solution
        }

        let base_angle = center_to_target.z.atan2(center_to_target.x);
        let theta = (radius / d).acos();

        for angle in [base_angle + theta, base_angle - theta] {
            let tangent_point = center + radius * Vec3::new(angle.cos(), 0.0, angle.sin());
            let radius_dir = (tangent_point - center).normalize();
            let travel_dir = if turn_left {
                Vec3::new(-radius_dir.z, 0.0, radius_dir.x)
            } else {
                Vec3::new(radius_dir.z, 0.0, -radius_dir.x)
            };
            if travel_dir.dot(target - tangent_point) > 0.0 {
                let start_angle = (start - center).z.atan2((start - center).x);
                let sweep = if turn_left {
                    angle - start_angle
                } else {
                    start_angle - angle
                }
                .rem_euclid(std::f32::consts::TAU);
                return Ok(Self {
                    start,
                    target,
                    center,
                    radius,
                    turn_left,
                    tangent_point,
                    sweep,
                    arc_length: radius * sweep,
                    straight_length: (target - tangent_point).length(),
                    distance_traveled: 0.0,
                });
            }
        }
        Err(())
    }

    pub(super) fn start_angle(&self) -> f32 {
        let to_start = self.start - self.center;
        to_start.z.atan2(to_start.x)
    }

    pub(super) fn is_completed(&self) -> bool {
        self.distance_traveled >= self.total_length()
    }

    pub(super) fn total_length(&self) -> f32 {
        self.arc_length + self.straight_length
    }

    pub(super) fn remaining_length(&self) -> f32 {
        self.total_length() - self.distance_traveled
    }
}
