use crate::layout::{Layout, THRESHOLD};
use crate::Error;
use image::imageops::{resize, FilterType};
use image::math::Rect;
use image::{GenericImageView, GrayImage};
use imageproc::contrast::threshold;
use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};
use std::fmt;
use std::ops::{Deref, DerefMut};

/// Recognized letters or bonus squares, organized as a two-dimensional grid.
#[derive(Debug, Clone, Default)]
pub struct Ocr(pub Vec<Vec<String>>);

impl Deref for Ocr {
    type Target = Vec<Vec<String>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Ocr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A list of [OcrStat](crate::OcrStat). Can be used to analyze the accuracy of template matching.
pub type OcrStats = Vec<OcrStat>;

/// Results for a single template match
#[derive(Debug, Clone, Default)]
pub struct OcrStat {
    /// The linear cell index (0.. nrows * ncols)
    index: usize,
    /// The tag of the matched template
    tag: String,
    /// The match error (the minimum value of all matched templates)
    min_value: f32,
    /// The location where the best template match was found
    min_value_location: (u32, u32),
}
/// Holds the result of recognize_screenshot: recognized tiles on the board and rack, plus grid with bonus squares.
#[derive(Debug, Clone)]
pub struct OcrResults {
    /// The tiles on the board
    pub tiles_ocr: Ocr,
    /// The grid with bonus squares
    pub grid_ocr: Ocr,
    /// The tiles on the rack
    pub rack_ocr: Ocr,
    /// Stats for tile recognition
    pub tiles_stats: OcrStats,
    /// Stats for grid recognition
    pub grid_stats: OcrStats,
    /// Stats for rack recognition
    pub rack_stats: OcrStats,
    /// Board area bounding rectangle
    pub board_area: Rect,
    /// Rack area bounding rectangle
    pub rack_area: Rect,
}

impl fmt::Display for Ocr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ocr_string = self
            .iter()
            .map(|v| v.join(""))
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}", ocr_string)
    }
}

impl OcrResults {}

const START_SQUARE: usize = 15 * 7 + 7;

/// The templates! macro embeds the templates in the library/
macro_rules! templates {
    ( $( $x:expr ),* ) => {
            [$(
                   ($x, include_bytes!(concat!("templates/", $x, ".png"))),
            )*]
        };
}

const LETTER_TEMPLATES: &[(&str, &[u8])] = &templates![
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z", "Æ", "Å", "Ä", "Ñ", "Ö", "Ø", "CH", "LL", "RR"
];

const BONUS_TEMPLATES: &[(&str, &[u8])] = &templates!["2L", "3L", "2W", "3W"];

fn template_from_buffer(name: &str, buf: &[u8]) -> (String, GrayImage) {
    (
        String::from(name),
        image::load_from_memory(buf).unwrap().to_luma8(), // can not fail because the templates are embedded
    )
}

/// Wordfeud board recognizer
pub struct Board {
    pub templates: Vec<(String, GrayImage)>,
    pub bonus_templates: Vec<(String, GrayImage)>,
}

impl Default for Board {
    fn default() -> Self {
        Board::new()
    }
}

impl Board {
    pub fn new() -> Board {
        let templates = LETTER_TEMPLATES
            .iter()
            .map(|(name, buf)| template_from_buffer(name, buf))
            .collect();
        let bonus_templates = BONUS_TEMPLATES
            .iter()
            .map(|(name, buf)| template_from_buffer(name, buf))
            .collect();
        Board {
            templates,
            bonus_templates,
        }
    }

    /// Recognize a wordfeud board screenshot.
    ///
    /// Returns a result that contains the detected tiles on the board and in the rack, and the detected board with
    /// the bonus tiles locations.
    /// The recognition process consists of these phases:
    /// 1. Segmentation of the board: find the board and rack area, and locate the cells on the board and the rack
    /// 2. Use template matching to recognize the tiles and bonus squares
    ///
    /// # Errors
    /// * The screenshot can not be segmented properly
    ///
    pub fn recognize_screenshot(&self, screenshot: &GrayImage) -> Result<OcrResults, Error> {
        let layout = Layout::new(&screenshot).segment()?;

        let cells = Layout::get_cells(&layout.rows, &layout.cols);
        // println!("{:?}", cells);
        let tile_index = layout.get_tile_index(&cells);
        let (tiles_ocr, tiles_stats) = self.recognize_tiles(
            screenshot,
            &layout,
            &tile_index,
            &cells,
            &self.templates,
            (15, 15),
        );

        let (grid_ocr, grid_stats) =
            self.recognize_board(screenshot, &layout, &cells, &self.bonus_templates, (15, 15));

        let cells = Layout::get_cells(&layout.rack_rows, &layout.rack_cols);
        let index: Vec<usize> = (0..cells.len()).into_iter().collect();
        let (rack_ocr, rack_stats) =
            self.recognize_tiles(screenshot, &layout, &index, &cells, &self.templates, (1, 7));

        let res = OcrResults {
            tiles_ocr,
            grid_ocr,
            rack_ocr,
            tiles_stats,
            grid_stats,
            rack_stats,
            board_area: layout.board_area,
            rack_area: layout.rack_area,
        };

        Ok(res)
    }

    pub fn recognize_screenshot_from_file(
        &self,
        screenshot_filename: &str,
    ) -> Result<OcrResults, Error> {
        let gray = image::open(&screenshot_filename)?.into_luma8();
        self.recognize_screenshot(&gray)
    }

    pub fn recognize_screenshot_from_memory(
        &self,
        screenshot: &[u8],
    ) -> Result<OcrResults, Error> {
        let gray = image::load_from_memory(&screenshot)?.into_luma8();
        self.recognize_screenshot(&gray)
    }

    fn topright(cell: Rect) -> Rect {
        Rect {
            x: cell.x + (0.73 * cell.width as f64).round() as u32,
            y: cell.y + (0.06 * cell.height as f64).round() as u32,
            width: (0.18 * cell.width as f64).round() as u32,
            height: (0.27 * cell.height as f64).round() as u32,
        }
    }

    fn recognize_tiles(
        &self,
        img: &GrayImage,
        layout: &Layout,
        tile_index: &[usize],
        cells: &[Rect],
        templates: &[(String, GrayImage)],
        size: (usize, usize),
    ) -> (Ocr, OcrStats) {
        // create rows x cols empty grid
        let (rows, cols) = size;
        let row: Vec<String> = (0..cols).into_iter().map(|_| String::from(".")).collect();
        let mut ocr = Ocr((0..rows)
            .into_iter()
            .map(|_| row.clone())
            .collect::<Vec<_>>());
        if tile_index.is_empty() {
            println!("No tiles");
            return (ocr, Vec::new());
        }

        let mut stats = Vec::new();
        let thresh = (THRESHOLD * 256.) as u8;
        for &index in tile_index.iter() {
            let cell = cells[index];

            // check if the tile is a blank (in the rack)
            let (mean, std) = layout.area_stats(&cell);
            let is_blank = mean > 0.9 && std < 0.2;
          
            // check if the tile is a wildcard
            let topright = Board::topright(cell);
            let (mean, std) = layout.area_stats(&topright);
            let is_wildcard = mean > 0.8 && std < 0.1;

            // create tile image
            let mut tile: GrayImage = img.view(cell.x, cell.y, cell.width, cell.height).to_image();
            // convert to binary image improves the template match accurarcy
            tile = threshold(&tile, thresh);

            if tile.width() > 67 {
                tile = resize(&tile, 67, 67, FilterType::Lanczos3);
            }

            // Area for template matching. Cell dimension is 67 square
            // Template dimension is wxh = 38 x 50
            let area = tile.view(6, 3, 40, 62).to_image();

            
            // match templates
            let (letter, min_value, min_value_location) = if ! is_blank {
                Board::match_template(&area, templates)
            } else {
                (String::from("*"), 0.0_f32, (0_u32,0_u32))
            };
            let (row, col) = (index / cols, index % cols);
            ocr[row][col] = if !is_wildcard {
                letter.to_lowercase()
            } else {
                letter.clone()
            };
            stats.push(OcrStat {
                index,
                tag: letter.clone(),
                min_value,
                min_value_location,
            });
        }
        (ocr, stats)
    }

    fn recognize_board(
        &self,
        img: &GrayImage,
        layout: &Layout,
        cells: &[Rect],
        templates: &[(String, GrayImage)],
        size: (usize, usize),
    ) -> (Ocr, OcrStats) {
        // create rows x cols empty grid
        let (rows, cols) = size;

        let row: Vec<String> = (0..cols).into_iter().map(|_| String::from("--")).collect();
        let mut ocr = Ocr((0..rows)
            .into_iter()
            .map(|_| row.clone())
            .collect::<Vec<_>>());
        ocr[7][7] = String::from("ss"); // start square
        let mut stats = Vec::new();
        for (index, cell) in cells.iter().enumerate() {
            let mean = layout.mean(&cell);
            if mean > THRESHOLD || mean < 0.25 || index == START_SQUARE {
                continue;
            }

            // create tile image
            let tile: GrayImage = img.view(cell.x, cell.y, cell.width, cell.height).to_image();

            // Area for template matching. Cell dimension is wxh = 67 x 67.
            // Template dimension is wxh = 46x46
            let area = tile.view(8, 21, 48, 28).to_image();

            // // match templates
            let (letter, min_value, min_value_location) = Board::match_template(&area, templates);
            let (row, col) = (index / cols, index % cols);
            ocr[row][col] = letter.to_lowercase();
            stats.push(OcrStat {
                index,
                tag: letter.to_lowercase(),
                min_value,
                min_value_location,
            });
        }
        (ocr, stats)
    }

    fn match_template(
        tile: &GrayImage,
        templates: &[(String, GrayImage)],
    ) -> (String, f32, (u32, u32)) {
         
        let method = MatchTemplateMethod::SumOfSquaredErrorsNormalized;
        let mut matches = templates
            .iter()
            .map(|(letter, template)| {
                (
                    letter.clone(),
                    find_extremes(&match_template(&tile, &template, method)),
                )
            })
            .collect::<Vec<_>>();
        // find the best match
        matches.sort_by(|a, b| a.1.min_value.partial_cmp(&b.1.min_value).unwrap());
        let (letter, extreme) = matches[0].clone();
        (letter, extreme.min_value, extreme.min_value_location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topright() {
        let cell = Rect {
            x: 0,
            y: 0,
            width: 67,
            height: 67,
        };
        let topright = Board::topright(cell);
        println!("{:?} {:?}", cell, topright);
        assert_eq!(
            topright,
            Rect {
                x: 49,
                y: 4,
                width: 12,
                height: 18
            }
        );
    }
}
