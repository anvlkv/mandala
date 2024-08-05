use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign};

use crate::{Float, Vector};

/// Angle value
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Angle(Float);

impl Angle {
    pub const ZERO: Self = Self(0.0);

    #[cfg(feature = "f64")]
    pub const TAU: Self = Self(std::f64::consts::TAU);

    #[cfg(feature = "f64")]
    pub const PI: Self = Self(std::f64::consts::PI);

    #[cfg(feature = "f64")]
    pub const FRAC_PI_2: Self = Self(std::f64::consts::FRAC_PI_2);

    #[cfg(feature = "f64")]
    pub const FRAC_PI_3: Self = Self(std::f64::consts::FRAC_PI_3);

    #[cfg(feature = "f64")]
    pub const FRAC_PI_4: Self = Self(std::f64::consts::FRAC_PI_4);

    #[cfg(feature = "f64")]
    pub const FRAC_PI_6: Self = Self(std::f64::consts::FRAC_PI_6);

    #[cfg(feature = "f64")]
    pub const FRAC_PI_8: Self = Self(std::f64::consts::FRAC_PI_8);

    #[cfg(feature = "f32")]
    pub const TAU: Self = Self(std::f32::consts::TAU);

    #[cfg(feature = "f32")]
    pub const PI: Self = Self(std::f32::consts::PI);

    #[cfg(feature = "f32")]
    pub const FRAC_PI_2: Self = Self(std::f32::consts::FRAC_PI_2);

    #[cfg(feature = "f32")]
    pub const FRAC_PI_3: Self = Self(std::f32::consts::FRAC_PI_3);

    #[cfg(feature = "f32")]
    pub const FRAC_PI_4: Self = Self(std::f32::consts::FRAC_PI_4);

    #[cfg(feature = "f32")]
    pub const FRAC_PI_6: Self = Self(std::f32::consts::FRAC_PI_6);

    #[cfg(feature = "f32")]
    pub const FRAC_PI_8: Self = Self(std::f32::consts::FRAC_PI_8);

    pub fn from_degrees(deg: Float) -> Self {
        Self(deg.to_radians()).wrapped()
    }

    pub fn from_radians(rad: Float) -> Self {
        Self(rad).wrapped()
    }

    pub fn to_degrees(&self) -> Float {
        self.0.to_degrees()
    }

    pub fn to_radians(&self) -> Float {
        self.0
    }

    pub fn cos(&self) -> Float {
        self.0.cos()
    }

    pub fn sin(&self) -> Float {
        self.0.sin()
    }

    pub fn radians_mut(&mut self) -> &mut Float {
        &mut self.0
    }

    fn wrapped(self) -> Self {
        Self(self.0.rem_euclid(Self::TAU.0))
    }
}

impl From<Vector> for Angle {
    fn from(value: Vector) -> Self {
        Self::from_radians(value.y.atan2(value.x))
    }
}

impl Add for Angle {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0).wrapped()
    }
}

impl AddAssign for Angle {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        *self = self.wrapped();
    }
}

impl Mul<Float> for Angle {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self(self.0 * rhs).wrapped()
    }
}

impl MulAssign<Float> for Angle {
    fn mul_assign(&mut self, rhs: Float) {
        self.0 *= rhs;
        *self = self.wrapped();
    }
}

impl Div<Float> for Angle {
    type Output = Self;

    fn div(self, rhs: Float) -> Self::Output {
        Self(self.0 / rhs).wrapped()
    }
}

impl Div<Angle> for Angle {
    type Output = Float;

    fn div(self, rhs: Self) -> Self::Output {
        self.0 / rhs.0
    }
}

impl DivAssign<Float> for Angle {
    fn div_assign(&mut self, rhs: Float) {
        self.0 /= rhs;
        *self = self.wrapped();
    }
}

#[cfg(test)]
mod angle_tests {
    use cfg_if::cfg_if;

    use super::*;

    #[test]
    fn test_from_degrees() {
        let angle = Angle::from_degrees(180.0);
        cfg_if! {
            if #[cfg(feature = "f64")] {
                assert_eq!(angle.to_radians(), std::f64::consts::PI);
            } else if #[cfg(feature = "f32")] {
                assert_eq!(angle.to_radians(), std::f32::consts::PI);
            }
        }
    }

    #[test]
    fn test_from_radians() {
        cfg_if! {
            if #[cfg(feature = "f64")] {
                let angle = Angle::from_radians(std::f64::consts::PI);
                assert_eq!(angle.to_degrees(), 180.0);
            } else if #[cfg(feature = "f32")] {
                let angle = Angle::from_radians(std::f32::consts::PI);
                assert_eq!(angle.to_degrees(), 180.0);
            }
        }
    }

    #[test]
    fn test_add() {
        let angle1 = Angle::from_degrees(90.0);
        let angle2 = Angle::from_degrees(90.0);
        let result = angle1 + angle2;
        assert_eq!(result.to_degrees(), 180.0);
    }

    #[test]
    fn test_add_assign() {
        let mut angle = Angle::from_degrees(90.0);
        angle += Angle::from_degrees(90.0);
        assert_eq!(angle.to_degrees(), 180.0);
    }

    #[test]
    fn test_mul() {
        let angle = Angle::from_degrees(90.0);
        let result = angle * 2.0;
        assert_eq!(result.to_degrees(), 180.0);
    }

    #[test]
    fn test_mul_assign() {
        let mut angle = Angle::from_degrees(90.0);
        angle *= 2.0;
        assert_eq!(angle.to_degrees(), 180.0);
    }

    #[test]
    fn test_div() {
        let angle = Angle::from_degrees(180.0);
        let result = angle / 2.0;
        assert_eq!(result.to_degrees(), 90.0);
    }

    #[test]
    fn test_div_assign() {
        let mut angle = Angle::from_degrees(180.0);
        angle /= 2.0;
        assert_eq!(angle.to_degrees(), 90.0);
    }

    #[test]
    fn test_div_angle() {
        let angle1 = Angle::from_degrees(180.0);
        let angle2 = Angle::from_degrees(90.0);
        let result = angle1 / angle2;
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_angle_wrapping() {
        let angle = Angle::from_degrees(360.0);
        assert_eq!(angle.to_degrees(), 0.0);
    }

    #[test]
    fn test_negative_angle_wrapping() {
        let angle = Angle::from_degrees(-90.0);
        assert_eq!(angle.to_degrees(), 270.0);
    }
}
