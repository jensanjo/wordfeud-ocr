use crate::segmenter::Segment;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Board not square {0}")]
    BoardNotSquare(f32),
    #[error("Failed to create layout")]
    LayoutFailed(Segment),
}
