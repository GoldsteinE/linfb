use std::fmt;
#[cfg(feature = "text")]
use rusttype;
#[cfg(feature = "images")]
use image;

#[derive(Debug)]
pub enum Error {
    InvalidColorString(String, &'static str),
    #[cfg(feature = "text")]
    FontNotFound,
    #[cfg(feature = "text")]
    BadFont(rusttype::Error),
    #[cfg(feature = "images")]
    BadImage(image::ImageError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;

        match self {
            InvalidColorString(color, description) =>
                write!(f, "invalid color string: {}; {}", color, description),
            #[cfg(feature = "text")]
            FontNotFound =>
                write!(f, "font with given constraints is not found"),

            #[cfg(feature = "text")]
            BadFont(err) =>
                write!(f, "bad font loaded: {}", err),

            #[cfg(feature = "images")]
            BadImage(err) =>
                write!(f, "bad image: {}", err),
        }
    }
}

#[cfg(feature = "text")]
impl From<rusttype::Error> for Error {
    fn from(err: rusttype::Error) -> Self {
        Self::BadFont(err)
    }
}

#[cfg(feature = "images")]
impl From<image::ImageError> for Error {
    fn from(err: image::ImageError) -> Self {
        Self::BadImage(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
