use crate::layout::Segment;
use thiserror::Error;

/// Errors that can occur when recognizing the board
#[derive(Debug, Error)]
pub enum Error {
    /// The detected board is not square
    #[error("Board not square {0}")]
    BoardNotSquare(f32),
    /// The board could not be segmented
    #[error("Failed to create layout")]
    LayoutFailed(Segment),
    /// An error from the [image](https://github.com/image-rs/image) library
    #[error("Image error")]
    ImageError(#[from] image::error::ImageError),
    
}
