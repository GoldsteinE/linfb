#[cfg(feature = "images")]
use std::path::Path;

use crate::error::Result;
use crate::shape::{Color, Shape};
use image;

/// Image shape. Can be created from any file, [`image`] crate can parse. Supports transparency
pub struct Image {
    image: image::RgbaImage,
}

impl Image {
    /// Create [`Image`] from file
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            image: image::open(path)?.to_rgba(),
        })
    }

    /// Create [`Image`] from in-memory buffer
    pub fn from_buffer(buffer: &[u8]) -> Result<Self> {
        Ok(Self {
            image: image::load_from_memory(buffer)?.to_rgba(),
        })
    }
}

impl Shape for Image {
    fn render(&self) -> Vec<Vec<Option<Color>>> {
        self.image
            .rows()
            .map(|row| {
                row.map(|rgba| {
                    let [r, g, b, a] = rgba.0;
                    if a == 0 {
                        None
                    } else {
                        Some((r, g, b, a).into())
                    }
                })
                .collect()
            })
            .collect()
    }
}
