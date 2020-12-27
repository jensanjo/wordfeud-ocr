#![doc(
    html_logo_url = "https://github.com/jensanjo/wordfeud-ocr/raw/master/images/logo-ocr.png",
    html_favicon_url = "https://github.com/jensanjo/wordfeud-ocr/raw/master/images/logo-ocr.png"
)]
//! An OCR library that reads the state of a Wordfeud board from a screenshot
//!
//! This library recognizes the tiles on a Wordfeud board and rack, and also the bonus squares on the board.
//!
//! # Basic usage
//! ```no_run
//! # use wordfeud_ocr::{Board, Error};
//! # use anyhow::Result;
//! let path = "screenshots/screenshot_english.png";
//! let gray = image::open(path)?.into_luma8();
//! let board = Board::new();
//! let result = board.recognize_screenshot(&gray)?;
//! println!("Tiles:\n{}", result.tiles_ocr);
//! # Ok::<(), Error>(())
//! ```

mod error;
mod layout;
mod recognizer;
mod utils;

pub use error::Error;
pub use layout::Layout;
pub use recognizer::{Board, Ocr, OcrResults, OcrStat, OcrStats};
pub use utils::{collage, save_templates};
