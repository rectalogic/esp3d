mod cube;
mod triangle;

#[cfg(feature = "triangle")]
pub use triangle::_render as render;

#[cfg(feature = "cube")]
pub use cube::_render as render;
