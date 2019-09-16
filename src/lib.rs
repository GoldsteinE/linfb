use std::io;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

use memmap::{MmapOptions, MmapMut};

mod sys;
pub use sys::fb_var_screeninfo;
pub use sys::get_var_screeninfo;

/// Framebuffer, ready to write to
pub struct Framebuffer {
    /// Virtual "screen" buffer. You should normally use .set_pixel() method to manipulate it
    pub screen: Vec<u8>,
    /// Information about framebuffer
    pub screen_info: fb_var_screeninfo,
    framebuffer: MmapMut
}

impl Framebuffer {
    /// Try to open /dev/fb0 and create Framebuffer object
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

        Ok(Self { screen, framebuffer, screen_info })
    }

    /// Flush internal "screen" contents to the real framebuffer device
    pub fn flush(&mut self) {
        self.framebuffer.copy_from_slice(self.screen.as_slice());
    }

    /// Set pixel at x, y to color r, g, b
    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) {
        let pixel_pos = ((y * self.screen_info.xres + x) * 4) as usize;

        let mut pixel = 0u32;
        pixel |= (r as u32) >> (8 - self.screen_info.red.length) << self.screen_info.red.offset;
        pixel |= (g as u32) >> (8 - self.screen_info.green.length) << self.screen_info.green.offset;
        pixel |= (b as u32) >> (8 - self.screen_info.blue.length) << self.screen_info.blue.offset;
        self.screen[pixel_pos..pixel_pos + 4].copy_from_slice(&pixel.to_ne_bytes());
    }
}
