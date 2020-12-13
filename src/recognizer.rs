use crate::error::Error;
use crate::layout::{variance, Layout};
use image::math::Rect;
use image::{io::Reader as ImageReader, GenericImageView, GrayImage};
use image::imageops::{resize, FilterType};
use imageproc::integral_image::sum_image_pixels;
use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};
use std::{fs, io};

pub type Ocr = Vec<Vec<String>>;
pub type OcrResult = Vec<(usize, String, f32, (u32, u32))>;

// #[derive(Debug)]
pub struct Board<'a> {
    pub img: &'a GrayImage,
    pub layout: Layout<'a>,
    pub templates: Vec<(String, GrayImage)>,
}

impl<'a> Board<'a> {
    pub fn new(img: &'a GrayImage) -> Board<'a> {
        let layout = Layout::new(img);
        Board {
            img,
            layout,
            templates: Vec::new(),
        }
    }

    pub fn read_templates(mut self) -> Result<Board<'a>, Error> {
        let mut entries = fs::read_dir("templates")
            .unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()
            .unwrap();
        entries.sort();
        let mut templates = Vec::new();
        for path in entries {
            if let Some(stem) = path.file_stem() {
                let key = stem.to_str().unwrap();
                let template = ImageReader::open(&path)
                    .unwrap()
                    .decode()
                    .unwrap()
                    .into_luma8();
                templates.push((String::from(key), template));
            }
        }
        self.templates = templates;
        Ok(self)
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

    /// calculate mean pixel value in rect
    #[allow(dead_code)]
    fn mean(&self, rect: Rect) -> f64 {
        let sum = sum_image_pixels(
            &self.layout.integral,
            rect.x,
            rect.y,
            rect.x + rect.width - 1,
            rect.y + rect.height - 1,
        );
        let count = rect.width * rect.height;
        sum[0] as f64 / count as f64 / 256.
    }

    /// calculate mean and variance pixel value in rect
    fn stats(&self, rect: Rect) -> (f64, f64) {
        let (left, top, right, bottom) = (
            rect.x,
            rect.y,
            rect.x + rect.width - 1,
            rect.y + rect.height - 1,
        );
        let sum = sum_image_pixels(&self.layout.integral, left, top, right, bottom);
        let var = variance(
            &self.layout.integral,
            &self.layout.integral_squared,
            left,
            top,
            right,
            bottom,
        );
        let count = rect.width * rect.height;
        (sum[0] as f64 / count as f64 / 256., var.sqrt() / 256.)
    }

    fn topright(cell: Rect) -> Rect {
        Rect {
            x: cell.x + (0.73 * cell.width as f64).round() as u32,
            y: cell.y + (0.06 * cell.height as f64).round() as u32,
            width: (0.18 * cell.width as f64).round() as u32,
            height: (0.27 * cell.height as f64).round() as u32,
        }
    }

    pub fn recognize_tiles(
        &self,
        tile_index: &[usize],
        cells: &[Rect],
        templates: &[(String, GrayImage)],
        size: (usize, usize),
        resize_to: Option<(u32, u32, FilterType)>
    ) -> (Ocr, OcrResult) {
        if tile_index.is_empty() {
            println!("No tiles");
            return (Vec::new(), Vec::new());
        }
        // create rows x cols empty grid
        let (rows, cols) = size;
        let row: Vec<String> = (0..cols).into_iter().map(|_| String::from(".")).collect();
        let mut ocr: Vec<Vec<String>> = (0..rows).into_iter().map(|_| row.clone()).collect();

        let mut recognized = Vec::new();
        for &index in tile_index.iter() {
            let cell = cells[index];
            
            // check if the tile is a wildcard
            let topright = Board::topright(cell);
            let (mean, std) = self.stats(topright);
            let wildcard = mean > 0.8 && std < 0.1;

            // create tile image
            let mut tile: GrayImage = self
                .img
                .view(cell.x, cell.y, cell.width, cell.height)
                .to_image();

            // Tiles in the rack must be resized.
            if let Some((nwidth, nheight, filter)) = resize_to {
                tile = resize(&tile, nwidth, nheight, filter);
            }

            // Area for template matching. Cell dimension is wxh = 67 x 67.
            // Template dimension is wxh = 38 x 50
            let area = tile.view(5, 13, 45, 52).to_image();
    
            // // match templates
            let (letter, min_value, min_value_location) =
                Board::match_template(&area, templates);
            let (row, col) = (index / cols, index % cols);
            ocr[row][col] = if !wildcard {
                letter.to_lowercase()
            } else {
                letter.clone()
            };
            recognized.push((index, letter.clone(), min_value, min_value_location));
        }
        (ocr, recognized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topright() {
        let cell = Rect { x: 0, y: 0, width: 67, height: 67 };
        let topright = Board::topright(cell);
        println!("{:?} {:?}", cell, topright);
        assert_eq!(topright, Rect { x:49, y: 4, width: 12, height: 18 });
    }
}
