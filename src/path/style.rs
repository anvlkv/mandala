use pix::{
    chan::{Ch8, Channel},
    el::Pixel,
    rgb::SRgba8,
    Raster,
};
use serde::ser::SerializeStruct;

use crate::{Angle, Float};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PathStyle {
    pub fill: Option<RasterSrc>,
    pub stroke: Option<Stroke>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub width: Float,
    pub paint: RasterSrc,
    pub position: StrokePosition,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq)]
pub enum StrokePosition {
    /// stroke on the inside of the closed shape
    Inside,
    /// stroke on the outside of the closed shape
    Outside,
    /// stroke evenly distributed along both sides of the path
    #[default]
    Center,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            width: 1.0,
            paint: RasterSrc::Plain(RgbColor(SRgba8::new(0, 0, 0, 255))),
            position: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct RgbColor(pub SRgba8);

#[derive(Clone)]
pub struct RgbRaster(pub Raster<SRgba8>);

/// source for raster drawing
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RasterSrc {
    /// plain color
    Plain(RgbColor),
    /// linear gradient with multiple stops.
    ///
    /// stops between 0.0 - 1.0
    Gradient {
        stops: Vec<(Float, RgbColor)>,
        angle: Angle,
    },
    /// image fill at angle
    Image { raster: RgbRaster, angle: Angle },
}

#[cfg(feature = "serde")]
impl serde::ser::Serialize for RgbColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("clr", 4)?;
        state.serialize_field("one", &self.0.one().to_f32())?;
        state.serialize_field("two", &self.0.two().to_f32())?;
        state.serialize_field("three", &self.0.three().to_f32())?;
        state.serialize_field("four", &self.0.four().to_f32())?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl serde::ser::Serialize for RgbRaster {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("raster", 3)?;
        state.serialize_field("width", &self.0.width())?;
        state.serialize_field("height", &self.0.height())?;
        state.serialize_field("data", &self.0.as_u8_slice())?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for RgbColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Color {
            one: f32,
            two: f32,
            three: f32,
            four: f32,
        }

        let color = Color::deserialize(deserializer)?;
        let mut rgb_clr = SRgba8::default();
        *rgb_clr.one_mut() = Ch8::from(color.one);
        *rgb_clr.two_mut() = Ch8::from(color.two);
        *rgb_clr.three_mut() = Ch8::from(color.three);
        *rgb_clr.four_mut() = Ch8::from(color.four);
        Ok(RgbColor(rgb_clr))
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for RgbRaster {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Raster {
            width: u32,
            height: u32,
            data: Vec<u8>,
        }

        let raster = Raster::deserialize(deserializer)?;

        let pix_raster = pix::Raster::<SRgba8>::with_u8_buffer(
            raster.width,
            raster.height,
            raster.data.as_slice(),
        );
        Ok(RgbRaster(pix_raster))
    }
}

impl PartialEq for RgbRaster {
    fn eq(&self, other: &Self) -> bool {
        self.0.width() == other.0.width()
            && self.0.height() == other.0.height()
            && self.0.as_u8_slice() == other.0.as_u8_slice()
    }
}

impl std::fmt::Debug for RgbRaster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "raster: {} by {} px", self.0.width(), self.0.height(),)
    }
}

#[cfg(test)]
mod style_test {
    use super::*;

    #[cfg(feature = "serde")]
    mod serde_test {
        use super::*;

        #[test]
        fn test_rgb_color_serialization() {
            let color = RgbColor(SRgba8::new(255, 128, 0, 0));
            let serialized = serde_json::to_string(&color).unwrap();
            assert_eq!(
                serialized,
                r#"{"one":1.0,"two":0.5019608,"three":0.0,"four":0.0}"#
            );
        }

        #[test]
        fn test_rgb_color_deserialization() {
            let json = r#"{"one":1.0,"two":0.5019608,"three":0.0,"four":0.0}"#;
            let color: RgbColor = serde_json::from_str(json).unwrap();
            assert_eq!(color, RgbColor(SRgba8::new(255, 128, 0, 0)));
        }

        #[test]
        fn test_rgb_raster_serialization() {
            let raster = Raster::<SRgba8>::with_clear(1, 1);
            let rgb_raster = RgbRaster(raster);
            let serialized = serde_json::to_string(&rgb_raster).unwrap();
            assert_eq!(serialized, r#"{"width":1,"height":1,"data":[0,0,0,0]}"#);
        }

        #[test]
        fn test_rgb_raster_deserialization() {
            let json = r#"{"width":1,"height":1,"data":[255,128,0,255]}"#;
            let rgb_raster: RgbRaster = serde_json::from_str(json).unwrap();
            let expected_raster = Raster::<SRgba8>::with_u8_buffer(1, 1, vec![255, 128, 0, 255]);
            assert_eq!(rgb_raster, RgbRaster(expected_raster));
        }
    }
}
