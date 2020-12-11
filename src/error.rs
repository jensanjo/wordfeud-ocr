use crate::layout::Segment;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Board not square {0}")]
    BoardNotSquare(f32),
    #[error("Failed to create layout")]
    LayoutFailed(Segment),
    /// Error reading wordfile
    #[error("Template \"{path}\" could not be read")]
    ReadError {
        path: String,
        source: std::io::Error,
    },
}
