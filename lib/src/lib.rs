pub mod errors;

mod swizzler;
pub use swizzler::{
    ChannelDescriptor,
    to_rgb,
    to_rgba,
    to_luma,
    to_lumaA,
    to_dynamic
};
