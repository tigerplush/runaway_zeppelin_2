use std::{
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign}, time::Duration,
};

use bevy::{math::FloatPow, prelude::*};

/// Temperature in °K
#[derive(Clone, Copy, Debug, Reflect)]
pub struct Temperature(f32);

/// Length in m
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Reflect)]
pub struct Length(pub f32);

impl Length {
    pub fn max(&self, rhs: Length) -> Length {
        Length(self.0.max(rhs.0))
    }
}

impl Add<Length> for Length {
    type Output = Self;
    fn add(self, rhs: Length) -> Self::Output {
        Length(self.0 + rhs.0)
    }
}

impl Sub<Length> for Length {
    type Output = Self;
    fn sub(self, rhs: Length) -> Self::Output {
        Length(self.0 - rhs.0)
    }
}

impl Div<Velocity> for Length {
    type Output = Duration;
    fn div(self, rhs: Velocity) -> Self::Output {
        let time = self.0 / rhs.0;
        Duration::from_secs_f32(time)
    }
}

/// Velocity in m/s
#[derive(Clone, Copy, Debug, Reflect)]
pub struct Velocity(pub f32);

impl Velocity {
    pub fn squared(&self) -> SquareMeterPerSquareSecond {
        SquareMeterPerSquareSecond(self.0.squared())
    }

    pub fn clamp(&self, min: Velocity, max: Velocity) -> Velocity {
        Velocity(self.0.clamp(min.0, max.0))
    }
}

impl Div<Acceleration> for Velocity {
    type Output = Duration;
    fn div(self, rhs: Acceleration) -> Self::Output {
        let time = self.0 / rhs.0;
        Duration::from_secs_f32(time)
    }
}

impl Mul<Duration> for Velocity {
    type Output = Length;
    fn mul(self, rhs: Duration) -> Self::Output {
        Length(self.0 * rhs.as_secs_f32())
    }
}

impl AddAssign<Velocity> for Velocity {
    fn add_assign(&mut self, rhs: Velocity) {
        self.0 += rhs.0;
    }
}

impl SubAssign<Velocity> for Velocity {
    fn sub_assign(&mut self, rhs: Velocity) {
        self.0 -= rhs.0;
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct SquareMeterPerSquareSecond(pub f32);

impl Div<Acceleration> for SquareMeterPerSquareSecond {
    type Output = Length;
    fn div(self, rhs: Acceleration) -> Self::Output {
        Length(self.0 / rhs.0)
    }
}

/// Acceleration in m/s²
#[derive(Clone, Copy, Debug, Reflect)]
pub struct Acceleration(pub f32);

impl Add<Acceleration> for Acceleration {
    type Output = Self;
    fn add(self, rhs: Acceleration) -> Self::Output {
        Acceleration(self.0 + rhs.0)
    }
}

impl Mul<Acceleration> for f32 {
    type Output = Acceleration;
    fn mul(self, rhs: Acceleration) -> Self::Output {
        Acceleration(self * rhs.0)
    }
}

impl Mul<f32> for Acceleration {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Mul<&Duration> for Acceleration {
    type Output = Velocity;
    fn mul(self, rhs: &Duration) -> Self::Output {
        Velocity(self.0 * rhs.as_secs_f32())
    }
}
