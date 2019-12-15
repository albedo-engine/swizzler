use image;

pub enum Error {
    Image(image::ImageError)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Use `self.number` to refer to each positional data point.
        match self {
            Self::Image(i) => write!(f, "{}", i)
        }
    }
}
