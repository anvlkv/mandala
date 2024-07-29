use pix::{rgb::SRgb8, Raster};

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
    Inside,
    Outside,
    #[default]
    Center,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            width: 1.0,
            paint: RasterSrc::Plain(SRgb8::new(0, 0, 0)),
            position: Default::default(),
        }
    }
}

#[derive(Clone)]
pub enum RasterSrc {
    Plain(SRgb8),
    Gradient {
        stops: Vec<(Float, SRgb8)>,
        angle: Angle,
    },
    Texture {
        raster: Raster<SRgb8>,
        angle: Angle,
    },
}

#[cfg(feature = "serde")]
impl serde::ser::Serialize for RasterSrc {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for RasterSrc {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl PartialEq for RasterSrc {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RasterSrc::Plain(c1), RasterSrc::Plain(c2)) => c1 == c2,
            (
                RasterSrc::Gradient {
                    stops: g1,
                    angle: a1,
                },
                RasterSrc::Gradient {
                    stops: g2,
                    angle: a2,
                },
            ) => a1 == a2 && g1 == g2,
            (
                RasterSrc::Texture {
                    raster: r1,
                    angle: a1,
                },
                RasterSrc::Texture {
                    raster: r2,
                    angle: a2,
                },
            ) => a1 == a2 && r1.as_u8_slice() == r2.as_u8_slice(),
            _ => false,
        }
    }
}

impl std::fmt::Debug for RasterSrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RasterSrc::Plain(c) => write!(f, "plain color: {c:?}"),
            RasterSrc::Gradient { stops, angle } => write!(
                f,
                "gradient: {stops:?}, at angle: {}deg",
                angle.to_degrees()
            ),
            RasterSrc::Texture { raster, angle } => {
                write!(
                    f,
                    "raster: {} by {} px, at angle: {}deg",
                    raster.width(),
                    raster.height(),
                    angle.to_degrees()
                )
            }
        }
    }
}
