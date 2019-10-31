//! Various drawing primitives

use std::convert::TryFrom;
use std::ops::{Mul, MulAssign};

use derive_builder::Builder;
use downcast_rs::{impl_downcast, Downcast};

use crate::{
    Error::{self, *},
    Result,
};

#[cfg(feature = "text")]
pub use crate::text::{Alignment, Caption, CaptionBuilder, FontBuilder};

#[cfg(feature = "images")]
pub use crate::image::Image;

/// RGBA color used in many places in the library. Alpha channel is `[0-255]`, not `[0-1]`.
///
/// Can be created from 4-tuple of [`u8`], 3-tuple of [`u8`] (assuming `255` in alpha channel) and hex
/// string:
/// ```
/// # use std::convert::TryInto;
/// # use linfb::shape::Color;
/// // All of these are equivalent:
/// let c1: Color = (128, 128, 128).into();
/// let c2: Color = (128, 128, 128, 255).into();
/// let c3: Color = "#808080".try_into().unwrap();
/// let c4: Color = "#808080ff".try_into().unwrap();
/// # assert_eq!(c1, c2);
/// # assert_eq!(c2, c3);
/// # assert_eq!(c3, c4);
/// ```
///
/// Can be multiplied to `[0, 1]` [`f32`] coefficient, which affects every channel besides alpha:
/// ```
/// # use linfb::shape::Color;
/// let color: Color = (128, 128, 128, 128).into(); // All channels set to 128
/// assert_eq!(color * 0.5, (64, 64, 64, 128).into());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, coeff: f32) -> Self {
        Self {
            red: (self.red as f32 * coeff) as u8,
            green: (self.green as f32 * coeff) as u8,
            blue: (self.blue as f32 * coeff) as u8,
            alpha: self.alpha,
        }
    }
}

impl MulAssign<f32> for Color {
    fn mul_assign(&mut self, coeff: f32) {
        self.red = ((self.red as f32) * coeff) as u8;
        self.green = ((self.green as f32) * coeff) as u8;
        self.blue = ((self.blue as f32) * coeff) as u8;
    }
}

impl Color {
    /// Create [`Color`] object from hex string.
    /// Equivalent to `.try_into()` on string slice:
    /// ```
    /// # use std::convert::TryInto;
    /// # use linfb::shape::Color;
    /// let c1: Color = "#112233".try_into().unwrap();
    /// let c2: Color = Color::hex("#112233").unwrap();
    /// assert_eq!(c1, c2);
    /// let c3: Color = "#11223344".try_into().unwrap();
    /// let c3: Color = Color::hex("#11223344").unwrap();
    /// assert_eq!(c1, c2);
    /// ```
    pub fn hex(color_string: &str) -> Result<Self> {
        if color_string.len() != 7 && color_string.len() != 9 {
            return Err(InvalidColorString(
                color_string.into(),
                "length must be 7 or 9",
            ));
        }
        if color_string.chars().next() != Some('#') {
            return Err(InvalidColorString(
                color_string.into(),
                "first char must be #",
            ));
        }
        if !color_string.chars().skip(1).all(|c| c.is_ascii_hexdigit()) {
            return Err(InvalidColorString(
                color_string.into(),
                "all characters but first must be hex",
            ));
        }

        // We can .unwrap() here, because checked that everything is hexdigits
        Ok(Self {
            red: u8::from_str_radix(&color_string[1..3], 16).unwrap(),
            green: u8::from_str_radix(&color_string[3..5], 16).unwrap(),
            blue: u8::from_str_radix(&color_string[5..7], 16).unwrap(),
            alpha: if color_string.len() == 9 {
                u8::from_str_radix(&color_string[7..9], 16).unwrap()
            } else {
                255
            },
        })
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(rgb: (u8, u8, u8)) -> Self {
        Self {
            red: rgb.0,
            green: rgb.1,
            blue: rgb.2,
            alpha: 255,
        }
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(rgba: (u8, u8, u8, u8)) -> Self {
        Self {
            red: rgba.0,
            green: rgba.1,
            blue: rgba.2,
            alpha: rgba.3,
        }
    }
}

impl TryFrom<&str> for Color {
    type Error = Error;
    fn try_from(color_string: &str) -> Result<Self> {
        Self::hex(color_string)
    }
}

/// Something you can draw on framebuffer
pub trait Shape: Downcast {
    /// Create a two-dimensional array of pixels. Every row must have the same length.
    ///
    /// [`None`] means "no pixel at all" and semantically equivalent to `(0, 0, 0, 0).into()`, but
    /// can have better performance.
    fn render(&self) -> Vec<Vec<Option<Color>>>;

    /// Convert self into [`PositionedShape`], saving position info. Needed for
    /// [`Compositor`](super::Compositor).
    fn at(self, x: usize, y: usize) -> PositionedShape
    where
        Self: Sized + 'static,
    {
        PositionedShape {
            x,
            y,
            shape: Box::new(self),
        }
    }
}
impl_downcast!(Shape);

/// [`Shape`], positioned for placing onto [`Compositor`](super::Compositor)
pub struct PositionedShape {
    pub x: usize,
    pub y: usize,
    pub shape: Box<dyn Shape + 'static>,
}

impl PositionedShape {
    /// Create [`PositionedShape`] from [`Shape`], consuming latter
    pub fn new<T: Shape + 'static>(x: usize, y: usize, shape: T) -> Self {
        Self {
            x,
            y,
            shape: Box::new(shape),
        }
    }

    /// Get shared reference to inner [`Shape`] if it's type matches `T`
    pub fn inner<T: Shape + 'static>(&self) -> Option<&T> {
        self.shape.downcast_ref()
    }

    /// Get exclusive reference to inner [`Shape`] if it's type matches `T`
    pub fn inner_mut<T: Shape + 'static>(&mut self) -> Option<&mut T> {
        self.shape.downcast_mut()
    }
}

/// Simplest of all shapes, just a rectangle
#[derive(Debug, Builder)]
pub struct Rectangle {
    /// Width of rectangle including border
    pub width: usize,
    /// Height of rectangle including border
    pub height: usize,
    /// Border width. Builder default is 1, set to 0 to disable borders
    #[builder(default = "1")]
    pub border_width: usize,
    /// Border color. Builder default is [`None`] (fully transparent)
    #[builder(setter(into, strip_option), default)]
    pub border_color: Option<Color>,
    /// Fill color. Builder default is [`None`] (fully transparent)
    #[builder(setter(into, strip_option), default)]
    pub fill_color: Option<Color>,
}

impl Rectangle {
    /// Create a default [`RectangleBuilder`]
    pub fn builder() -> RectangleBuilder {
        RectangleBuilder::default()
    }
}

impl Shape for Rectangle {
    fn render(&self) -> Vec<Vec<Option<Color>>> {
        (0..self.height)
            .map(|y| {
                (0..self.width)
                    .map(|x| {
                        if x < self.border_width
                            || x >= self.width - self.border_width
                            || y < self.border_width
                            || y >= self.height - self.border_width
                        {
                            self.border_color
                        } else {
                            self.fill_color
                        }
                    })
                    .collect()
            })
            .collect()
    }
}
