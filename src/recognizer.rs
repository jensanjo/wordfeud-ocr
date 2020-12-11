use crate::error::Error;
use crate::layout::{Layout, variance};
use image::math::Rect;
use image::{
    io::Reader as ImageReader, GenericImage, GenericImageView, GrayImage, ImageBuffer, Luma,
    SubImage,
};
use imageproc::integral_image::sum_image_pixels;
use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};
use std::{fs, io};

pub type Ocr = Vec<Vec<String>>;
pub type OcrResult = Vec<(usize, String, f32, (u32, u32))>;

pub const THRESHOLD: f64 = 0.65;

// #[derive(Debug)]
pub struct Board<'a> {
    pub img: &'a GrayImage,
    pub layout: Layout<'a>,
    pub cells: Vec<Rect>,
    pub tile_index: Vec<usize>,
    templates: Vec<(String, GrayImage)>,
}

impl<'a> Board<'a> {
    pub fn new(img: &'a GrayImage) -> Board<'a> {
        let layout = Layout::new(img);
        Board {
            img,
            layout,
            cells: Vec::new(),
            tile_index: Vec::new(),
            templates: Vec::new(),
        }
    }

    /// Create tile sub images for board and tray
    pub fn get_cells(&self) -> Vec<Rect> {
        let mut cells = Vec::new();
        // make sure we have 15x15 tiles
        assert_eq!(self.layout.rows.len(), 15);
        assert_eq!(self.layout.cols.len(), 15);
        // find out what size our tiles should be
        let tiles_height: usize = self.layout.rows.iter().map(|&(y0, y1)| y1 - y0).sum();
        let tiles_width: usize = self.layout.cols.iter().map(|&(x0, x1)| x1 - x0).sum();
        let (tile_height, tile_width) = ((tiles_height as u32) / 15, (tiles_width as u32) / 15);
        for &(y0, _y1) in self.layout.rows.iter() {
            for &(x0, _x1) in self.layout.cols.iter() {
                let cell = Rect {
                    x: x0 as u32,
                    y: y0 as u32,
                    width: tile_width,
                    height: tile_height,
                };
                cells.push(cell);
            }
        }
        cells
    }

    pub fn get_tile_index(&self) -> Vec<usize> {
        let mut index = Vec::new();
        for (i, cell) in self.cells.iter().enumerate() {
            let (left, top, right, bottom) = (
                cell.x,
                cell.y,
                cell.x + cell.width - 1,
                cell.y + cell.height - 1,
            );
            let sum = sum_image_pixels(&self.layout.integral, left, top, right, bottom);
            let mean = sum[0] as f64 / (cell.width * cell.height) as f64 / 256.;
            if mean > THRESHOLD {
                index.push(i);
            }
            // println!("{:2} {:2} {:.2}", i / 15, i % 15, mean as f64 / 256.);
        }
        index
    }

    /// Create a collage image of all the segmented tiles
    pub fn collage(&self) -> Option<ImageBuffer<Luma<u8>, Vec<u8>>> {
        if self.tile_index.is_empty() {
            return None;
        }
        let nimages = self.tile_index.len() as f64;
        let n = nimages.sqrt().ceil(); // size of collage square
        let (ncols, nrows) = (n as u32, ((nimages / n).ceil()) as u32);

        let cell = self.cells[0];
        let (w, h) = (cell.width, cell.height);
        let mut img: GrayImage = ImageBuffer::new(w * ncols, h * nrows);
        for (i, &idx) in self.tile_index.iter().enumerate() {
            let cell = self.cells[idx];
            // create destination sub image in collage:
            let (row, col) = ((i as u32 / ncols), (i as u32 % ncols));
            let mut dest: SubImage<&mut GrayImage> = img.sub_image(col * w, row * h, w, h);
            // create source sub image
            let src: SubImage<&GrayImage> = self.img.view(cell.x, cell.y, w, h);
            // copy the pixels
            dest.copy_from(&src, 0, 0).unwrap();
        }
        Some(img)
    }

    pub fn read_templates(&mut self) -> Result<(), Error> {
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
        Ok(())
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
        let (left, top, right, bottom) = (rect.x, rect.y, rect.x + rect.width - 1, rect.y + rect.height - 1);
        let sum = sum_image_pixels(
            &self.layout.integral,
            left, top, right, bottom
        );
        let var = variance(
            &self.layout.integral,
            &self.layout.integral_squared,
           left, top, right, bottom
        );
        let count = rect.width * rect.height;
        (sum[0] as f64 / count as f64 / 256., var.sqrt() / 256.)
    }

    pub fn recognize_tiles(
        &self,
        tile_index: &[usize],
        cells: &[Rect],
        size: (usize, usize),
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
        for &index in self.tile_index.iter() {
            let b = cells[index];
            let (dx, dy, w, h) = (7, 12, 43, 55);
            let bounds = Rect {
                x: b.x + dx,
                y: b.y + dy,
                width: w,
                height: h,
            };
            // check if the tile is a wildcard
            let topright = Rect { x: b.x + 49, y: b.y + 4, width: 12, height: 18 };
            let (mean, std) = self.stats(topright);
            println!("{:2} {:.2} {:.2}", index, mean, std);
            let wildcard = mean > 0.9 && std < 0.1;

            let tile: GrayImage = self
                .img
                .view(bounds.x, bounds.y, bounds.width, bounds.height)
                .to_image();
            // // match templates
            let (letter, min_value, min_value_location) =
                Board::match_template(&tile, &self.templates);
            let (row, col) = (index / cols, index % cols);
            ocr[row][col] = if ! wildcard {
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
    // use super::*;
}
