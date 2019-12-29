use image;

#[derive(Debug)]
pub enum Error {
    Image(image::ImageError),
    IOError(std::io::Error),
    Invalid
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Image(err) => Some(err),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "{}", self)
    }
}

impl From<image::ImageError> for Error {
    fn from(ie: image::ImageError) -> Self {
        Self::Image(ie)
    }
}

impl From<std::io::Error> for Error {
    fn from(ie: std::io::Error) -> Self {
        Self::IOError(ie)
    }
}
