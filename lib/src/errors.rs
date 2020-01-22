use image;

#[derive(Debug)]
pub enum ErrorKind {
    Image(image::ImageError),
    IOError(std::io::Error),
    InvalidSize,
    Invalid
}

impl std::error::Error for ErrorKind {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ErrorKind::Image(err) => Some(err),
            _ => None,
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "{}", self)
    }
}

impl From<image::ImageError> for ErrorKind {
    fn from(ie: image::ImageError) -> Self {
        Self::Image(ie)
    }
}

impl From<std::io::Error> for ErrorKind {
    fn from(ie: std::io::Error) -> Self {
        Self::IOError(ie)
    }
}
