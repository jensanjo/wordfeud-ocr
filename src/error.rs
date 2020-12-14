use crate::layout::Segment;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Board not square {0}")]
    BoardNotSquare(f32),
    #[error("Failed to create layout")]
    LayoutFailed(Segment),
    /// Error reading wordfile
    #[error("Template could not be read")]
    TemplateReadError(#[from] io::Error),
    /// Error decoding image
    #[error("Image {path} could not be decoded")]
    ImageError {
        path: String,
        source: image::error::ImageError,
    },
}
