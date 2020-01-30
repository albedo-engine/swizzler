pub mod errors;

mod swizzler;
pub use swizzler::{
    ChannelDescriptor,
    to_rgb,
    to_rgba,
};
