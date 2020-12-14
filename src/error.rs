use crate::layout::Segment;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Board not square {0}")]
    BoardNotSquare(f32),
    #[error("Failed to create layout")]
    LayoutFailed(Segment),
    /// Error decoding image
    #[error("Image {path} could not be decoded")]
    ImageError {
        path: String,
        source: image::error::ImageError,
    },
}
