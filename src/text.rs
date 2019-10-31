#[cfg(feature = "text")]
use derive_builder::Builder;
use font_loader::system_fonts::FontPropertyBuilder;
use rusttype::{point, Font, PositionedGlyph, Scale};
use xi_unicode::LineBreakIterator;

use crate::error::{Error::*, Result};
use crate::shape::{Color, Shape};

/// Builder for [`Font`]. All methods map to corresponding [`FontPropertyBuilder`] methods.
#[derive(Default)]
pub struct FontBuilder {
    italic: bool,
    oblique: bool,
    bold: bool,
    monospace: bool,
    family: String,
}

impl FontBuilder {
    pub fn italic(&mut self) -> &mut Self {
        self.italic = true;
        self
    }

    pub fn oblique(&mut self) -> &mut Self {
        self.oblique = true;
        self
    }

    pub fn bold(&mut self) -> &mut Self {
        self.bold = true;
        self
    }

    pub fn monospace(&mut self) -> &mut Self {
        self.monospace = true;
        self
    }

    pub fn family(&mut self, name: &str) -> &mut Self {
        self.family = String::from(name);
        self
    }

    /// Try to build an owned font with given properties
    pub fn build(&self) -> Result<Font<'static>> {
        let mut property_builder = FontPropertyBuilder::new().family(&self.family);
        if self.italic {
            property_builder = property_builder.italic();
        }
        if self.oblique {
            property_builder = property_builder.oblique();
        }
        if self.bold {
            property_builder = property_builder.bold();
        }
        if self.monospace {
            property_builder = property_builder.monospace();
        }

        let font_data = font_loader::system_fonts::get(&property_builder.build());
        if let Some((font_data, _)) = font_data {
            Ok(Font::from_bytes(font_data)?)
        } else {
            Err(FontNotFound)
        }
    }
}

/// Text alignment for [`Caption`]. Default is [`Alignment::Left`]
#[derive(Debug, Clone)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

impl Default for Alignment {
    fn default() -> Self {
        Self::Left
    }
}

/// Shape containing single- or multi-line text. Text will be soft wrapped if `max_width` is set.
#[derive(Debug, Builder)]
pub struct Caption {
    /// Caption text
    pub text: String,
    /// Font size in px
    pub size: u32,
    /// Font object, built with [`FontBuilder`]
    pub font: Font<'static>,
    /// Font color. Default is black
    #[builder(default = "Color::from((0, 0, 0))")]
    pub color: Color,
    /// Soft wrap width. If not set, soft wrapping will be disabled
    #[builder(setter(strip_option), default)]
    pub max_width: Option<usize>,
    #[builder(default)]
    /// Text alignment
    pub alignment: Alignment,
}

impl Caption {
    /// Create a default [`CaptionBuilder`]
    pub fn builder() -> CaptionBuilder {
        CaptionBuilder::default()
    }

    fn layout(&self, text: &str) -> Vec<PositionedGlyph<'_>> {
        let scale = Scale::uniform(self.size as f32);
        let offset = point(0f32, self.font.v_metrics(scale).ascent);
        let text: String = text
            .chars()
            .filter(|c| {
                self.font
                    .glyph(*c)
                    .standalone()
                    .get_data()
                    .and_then(|g| Some(g.id != 0))
                    .unwrap_or(false)
            })
            .collect();
        self.font.layout(&text, scale, offset).collect()
    }

    fn width(&self, glyphs: &Vec<PositionedGlyph<'_>>) -> f32 {
        match glyphs.iter().rev().next() {
            Some(glyph) => {
                glyph.position().x as f32 + glyph.unpositioned().h_metrics().advance_width
            }
            None => 0f32,
        }
    }

    fn str_width(&self, text: &str) -> usize {
        self.width(&self.layout(text)).round() as usize
    }

    fn split_text_at_indices(&self, indices: Vec<usize>) -> Vec<&str> {
        let mut last_split = 0;
        let mut result = Vec::with_capacity(indices.len() + 1);
        for idx in indices {
            result.push(&self.text[last_split..idx]);
            last_split = idx;
        }
        result.push(&self.text[last_split..]);
        result
    }

    fn split_text(&self) -> Vec<&str> {
        let mut prev_offset = 0;
        let mut prev_break = None;
        let mut where_to_break = vec![];

        for (offset, hard_break) in LineBreakIterator::new(&self.text) {
            if let Some(max_width) = self.max_width {
                let width = self.str_width(&self.text[prev_offset..offset]);
                if width > max_width {
                    if let Some(mut prev_break) = prev_break {
                        prev_offset = prev_break;

                        // Stripping space from line end
                        // Can .unwrap() here because prev_break is line offset
                        if self
                            .text
                            .chars()
                            .nth(prev_break - 1)
                            .unwrap()
                            .is_whitespace()
                        {
                            prev_break -= 1;
                        }
                        where_to_break.push(prev_break);
                    }
                }
            }

            if hard_break {
                where_to_break.push(offset);
                prev_offset = offset;
            }

            prev_break = Some(offset);
        }

        self.split_text_at_indices(where_to_break)
    }

    fn render_line(&self, line: &str) -> Vec<Vec<Option<Color>>> {
        let glyphs = self.layout(line);
        let width = self.width(&glyphs);

        let mut result = vec![vec![None; width.ceil() as usize]; self.size as usize];
        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    let x = (x + i32::max(0, bounding_box.min.x) as u32) as usize;
                    let y = (y + i32::max(0, bounding_box.min.y) as u32) as usize;
                    if y < result.len() && x < result[0].len() {
                        result[y][x] = Some(Color {
                            red: self.color.red,
                            green: self.color.green,
                            blue: self.color.blue,
                            alpha: (self.color.alpha as f32 * v) as u8,
                        })
                    }
                })
            }
        }

        result
    }

    fn align_line(&self, line: Vec<Vec<Option<Color>>>, width: usize) -> Vec<Vec<Option<Color>>> {
        match self.alignment {
            Alignment::Left => line
                .into_iter()
                .map(|mut row| {
                    row.resize(width, None);
                    row
                })
                .collect(),
            Alignment::Right => line
                .iter()
                .map(|row| {
                    let mut new_row = vec![None; width];
                    let row_len = usize::min(width, row.len());
                    &new_row[width - row_len..].copy_from_slice(&row[..row_len]);
                    new_row
                })
                .collect(),
            Alignment::Center => line
                .iter()
                .map(|row| {
                    let mut new_row = vec![None; width];
                    let row_len = usize::min(width, row.len());
                    let offset = (width - row_len) / 2;
                    &new_row[offset..row_len + offset].copy_from_slice(&row[..row_len]);
                    new_row
                })
                .collect(),
        }
    }
}

impl Shape for Caption {
    fn render(&self) -> Vec<Vec<Option<Color>>> {
        let line_gap = self
            .font
            .v_metrics(Scale::uniform(self.size as f32))
            .line_gap
            .round() as u32;

        let mut lines = vec![];
        let mut max_real_width = None;
        for line in self.split_text() {
            let mut rendered_line = self.render_line(line);
            if let Some(max_width) = rendered_line.iter().map(Vec::len).max() {
                max_real_width = if let Some(old_max_width) = max_real_width {
                    Some(usize::max(old_max_width, max_width))
                } else {
                    Some(max_width)
                }
            }
            for _ in 0..line_gap {
                rendered_line.push(vec![None]);
            }
            lines.push(rendered_line)
        }

        let width = if let Some(max_width) = self.max_width {
            max_width
        } else {
            max_real_width.unwrap_or(0)
        };

        lines
            .into_iter()
            .map(|line| self.align_line(line, width))
            .flatten()
            .collect()
    }
}
