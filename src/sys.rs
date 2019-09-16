use nix::ioctl_read_bad;

/// System structure representing one RGB channel parameters
#[repr(C)]
#[derive(Clone, Default, Debug)]
pub struct fb_bitfield {
    /// Offset in bits from the right of pixel
    pub offset: u32,
    /// Length in bits
    pub length: u32,
    /// Is most significant bit at the right. Should be 0 on modern systems
    pub msb_right: u32,
}

/// System structure representing variable screen info
#[repr(C)]
#[derive(Clone, Default, Debug)]
pub struct fb_var_screeninfo {
    /// Resolution of screen, X axis
    pub xres: u32,
    /// Resolution of screen, Y axis
    pub yres: u32,
    pub xres_virtual: u32,
    pub yres_virtual: u32,
    pub xoffset: u32,
    pub yoffset: u32,
    /// Number of bits per one pixel. Should be 32
    pub bits_per_pixel: u32,
    /// Is framebuffer grayscale. Should be 0 on modern systems
    pub grayscale: u32,
    /// Definition of red channel format
    pub red: fb_bitfield,
    /// Definition of green channel format
    pub green: fb_bitfield,
    /// Definition of blue channel format
    pub blue: fb_bitfield,
    /// Definition of alpha channel format
    pub transp: fb_bitfield,
    pub nonstd: u32,
    pub activate: u32,
    /// Height of the screen, in mm
    pub height: u32,
    /// Width of the screen, in mm
    pub width: u32,
    pub accel_flags: u32,
    pub pixclock: u32,
    pub left_margin: u32,
    pub right_margin: u32,
    pub upper_margin: u32,
    pub lower_margin: u32,
    pub hsync_len: u32,
    pub vsync_len: u32,
    pub sync: u32,
    pub vmode: u32,
    pub rotate: u32,
    pub colorspace: u32,
    pub reserved: [u32; 4],
}

impl fb_var_screeninfo {
    /// Overall size of framebuffer in bytes
    pub fn overall_size(&self) -> usize {
        (self.xres * self.yres * self.bits_per_pixel / 8) as usize
    }
}

ioctl_read_bad!(get_var_screeninfo, 0x4600, fb_var_screeninfo);
