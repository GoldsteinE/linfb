use derive_builder::Builder;
use crate::shape::{Color, Shape, PositionedShape};


/// Shape that can contain other shapes. Can deal with transparency and overlaps.
#[derive(Builder)]
pub struct Compositor {
    /// Width of compositor in pixels
    pub width: usize,
    /// Height of compositor in pixels
    pub height: usize,
    /// Background color. Transparent backgrounds will be treated as if they're placed over black
    /// background
    pub background: Color,
    #[builder(setter(skip))]
    shapes: Vec<(String, PositionedShape)>,
}


impl Compositor {
    /// Create empty compositor with given size and background
    pub fn new(width: usize, height: usize, background: Color) -> Self {
        Self {
            width,
            height,
            background,
            shapes: Vec::new()
        }
    }

    /// Create a default [`CompositorBuilder`]
    pub fn builder() -> CompositorBuilder {
        CompositorBuilder::default()
    }

    /// Add a [`PositionedShape`] with given name. Later you can get a reference to shape by it's
    /// name.
    ///
    /// Uniqueness of names is not enforced, but recommended
    pub fn add(&mut self, name: &str, shape: PositionedShape) -> &mut Self {
        self.shapes.push((name.into(), shape));
        self
    }

    /// Get a previously added [`PositionedShape`] by it's name. Will return [`None`] if shape
    /// with such name was never added.
    pub fn get_positioned(&mut self, name: &str) -> Option<&mut PositionedShape> {
        self
            .shapes
            .iter_mut()
            .filter_map(|(curr_name, shape)| if curr_name == name { Some(shape) } else { None }).next()
    }

    /// Get inner shape of previously added [`PositionedShape`] by it's name. Will return [`None`]
    /// if shape with such name was never added or has a different type. Use it like this:
    /// ```
    /// # use linfb::Compositor;
    /// # use linfb::shape::{Rectangle, Shape};
    /// # let mut compositor = Compositor::new(100, 100, (0, 0, 0).into());
    /// # compositor.add("rectangle_name", Rectangle::builder()
    /// #     .width(20)
    /// #     .height(20)
    /// #     .build()
    /// #     .unwrap()
    /// #     .at(10, 10));
    /// let rect: &mut Rectangle = compositor.get("rectangle_name").unwrap();
    /// ```
    pub fn get<T: Shape>(&mut self, name: &str) -> Option<&mut T> {
        self
            .get_positioned(name)
            .and_then(|shape| shape.inner_mut::<T>())
    }
}

impl Shape for Compositor {
    fn render(&self) -> Vec<Vec<Option<Color>>> {
        let mut result = vec![vec![Some(self.background); self.width]; self.height];
        for (_name, shape) in &self.shapes {
            for (y, row) in shape.shape.render().iter().enumerate() {
                for (x, color) in row.iter().enumerate() {
                    let real_x = shape.x + x;
                    let real_y = shape.y + y;
                    if real_y >= result.len() || real_x >= result[real_y].len() {
                        continue;
                    }

                    if let Some(color) = color {
                        let opacity = color.alpha as f32 / 255f32;
                        let rev_opacity = 1f32 - opacity;
                        // Can unwrap here because result initialized without None's
                        let mut prev_color = result[real_y][real_x].unwrap(); 
                        if prev_color.alpha != 255 {
                            prev_color *= (prev_color.alpha as f32) / 255f32;
                            prev_color.alpha = 255;
                        }
                        let new_color = Some(Color {
                            red: (color.red as f32 * opacity + prev_color.red as f32 * rev_opacity) as u8,
                            green: (color.green as f32 * opacity + prev_color.green as f32 * rev_opacity) as u8,
                            blue: (color.blue as f32 * opacity + prev_color.blue as f32 * rev_opacity) as u8,
                            alpha: 255
                        });

                        result[real_y][real_x] = new_color;
                    }
                }
            }
        }
        result
    }
}
