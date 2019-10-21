//! linfb is a drawing library that uses Linux' `/dev/fb0` device as it's backend. For most
//! tasks you probably want to use OpenGL or Vulkan backed library. `/dev/fb0` is deprecated but
//! still useful for some specific cases. This library supports framebuffers that use 32 bits per
//! pixel, so (theoretically) most modern systems.
//!
//! Before drawing on framebuffer you should allocate a virtual terminal and switch to it. I
//! recommend using [vt](https://crates.io/crates/vt) crates for this task. You should never draw
//! on virtual terminal used by X.org/Wayland server, this is unsafe and can lead to panics.
//!
//! By default linfb includes text and images drawing capabilities, which brings additional
//! dependencies. You can disable these features if you only need low-level framebuffer
//! interactions and [`Shape`] trait.
//!
//! Basic usage can look like this:
//! ```ignore
//! use linfb::Framebuffer;
//! use linfb::shape::{Color, Shape, Rectangle, Caption, Image, FontBuilder, Alignment};
//! let mut framebuffer = Framebuffer::open()
//!     .expect("Failed to open framebuffer");
//! let mut compositor = framebuffer.compositor((255, 255, 255).into());
//! compositor
//!     .add("rect1", Rectangle::builder()
//!         .width(100)
//!         .height(100)
//!         .fill_color(Color::hex("#ff000099").unwrap())
//!         .build()
//!         .unwrap()
//!         .at(100, 100))
//!     .add("rect2", Rectangle::builder()
//!         .width(100)
//!         .height(100)
//!         .fill_color(Color::hex("#00ff0099").unwrap())
//!         .build()
//!         .unwrap()
//!         .at(150, 150))
//!     .add("image", Image::from_path("image.png")
//!         .unwrap()
//!         .at(500, 500))
//!     .add("wrapped_text", Caption::builder()
//!         .text("Some centered text\nwith newlines".into())
//!         .size(56)
//!         .color(Color::hex("#4066b877").unwrap())
//!         .font(FontBuilder::default()
//!               .family("monospace")
//!               .build()
//!               .unwrap()
//!         )
//!         .alignment(Alignment::Center)
//!         .max_width(650)
//!         .build()
//!         .unwrap()
//!         .at(1000, 300));
//! // Compositor is shape, so we can just draw it at the top left angle
//! framebuffer.draw(0, 0, &compositor);
//! // Really changing screen contents
//! framebuffer.flush();
//! ```

use std::fs::OpenOptions;
use std::io;
use std::os::unix::io::AsRawFd;

use memmap::{MmapMut, MmapOptions};

pub mod sys;
use sys::fb_var_screeninfo;
use sys::get_var_screeninfo;

mod error;
pub use error::{Error, Result};

pub mod shape;
use shape::{Shape, Color};

mod compositor;
pub use compositor::{Compositor, CompositorBuilder};

#[cfg(feature = "text")]
mod text;

#[cfg(feature = "images")]
mod image;

/// Basic object used to manipulate framebuffer.
/// You should normally use [Shape] and [Compositor] to draw on it
pub struct Framebuffer {
    screen: Vec<u8>,
    /// Information about framebuffer
    pub screen_info: fb_var_screeninfo,
    framebuffer: MmapMut,
}

impl Framebuffer {
    /// Try to open `/dev/fb0` and create Framebuffer object.
    /// It requires root privileges on most systems.
    /// This method will panic if `/dev/fb0` is not a framebuffer or it's pixel size is not 32 bits
    pub fn open() -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open("/dev/fb0")?;
        let mut screen_info: fb_var_screeninfo = Default::default();
        unsafe {
            get_var_screeninfo(file.as_raw_fd(), &mut screen_info)
                .expect("Failed to get var_screeninfo")
        };

        if screen_info.bits_per_pixel != 32 {
            panic!("Size of one pixel must be 32 bits for linfb to work");
        }

        let framebuffer = unsafe {
            MmapOptions::new()
                .len(screen_info.overall_size())
                .map_mut(&file)?
        };
        let screen = vec![0u8; framebuffer.len()];

        Ok(Self {
            screen,
            framebuffer,
            screen_info,
        })
    }

    /// Flush internal buffer contents to the real framebuffer device
    pub fn flush(&mut self) {
        self.framebuffer.copy_from_slice(self.screen.as_slice());
    }

    /// Set pixel at x, y to color.
    /// Alpha value of color is probably will be ignored, as it doesn't makes sense in this context
    pub fn set_pixel<C: Into<Color>>(&mut self, x: u32, y: u32, color: C) {
        let color: Color = color.into();
        let pixel_pos = ((y * self.screen_info.xres + x) * 4) as usize;

        let mut pixel = 0u32;
        pixel |= (color.red as u32) >> (8 - self.screen_info.red.length) << self.screen_info.red.offset;
        pixel |= (color.green as u32) >> (8 - self.screen_info.green.length) << self.screen_info.green.offset;
        pixel |= (color.blue as u32) >> (8 - self.screen_info.blue.length) << self.screen_info.blue.offset;
        pixel |= (color.alpha as u32) >> (8 - self.screen_info.transp.length) << self.screen_info.transp.offset;
        self.screen[pixel_pos..pixel_pos + 4].copy_from_slice(&pixel.to_ne_bytes());
    }

    /// Draw shape on internal buffer
    pub fn draw<T: Shape>(&mut self, x: u32, y: u32, shape: &T) {
        for (inner_y, row) in shape.render().iter().enumerate() {
            for (inner_x, color) in row.iter().enumerate() {
                if let Some(color) = color {
                    self.set_pixel(x + (inner_x as u32), y + (inner_y as u32), *color);
                }
            }
        }
    }

    /// Create [Compositor] object with size of a screen and given background color
    pub fn compositor(&self, background: Color) -> Compositor {
        Compositor::new(self.screen_info.xres as usize, self.screen_info.yres as usize, background)
    }
}
