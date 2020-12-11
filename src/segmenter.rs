use crate::error::Error;
use image::{DynamicImage, math::Rect};
use image::{io::Reader as ImageReader, ImageBuffer, Luma, /*SubImage,*/ GenericImage, GenericImageView};
use imageproc::integral_image::{integral_image, integral_squared_image, sum_image_pixels};
use std::{fs, io};

pub const THRESHOLD: f64 = 0.65;

#[derive(Debug, PartialEq)]
pub enum Segment {
    LookForTopBorder(usize),
    InTopBorder,
    LookForRisingEdge(usize),
    InTile(usize),
    LookForBottomBorder(usize),
    InBottomBorder,
    LookForTray,
    InTray,
    Done,
}

pub type GrayImage = ImageBuffer<Luma<u8>, Vec<u8>>;

// #[derive(Debug)]
pub struct BoardLayout<'a> {
    pub img: &'a GrayImage,
    pub integral: ImageBuffer<Luma<u64>, Vec<u64>>,
    pub integral_squared: ImageBuffer<Luma<u64>, Vec<u64>>,
    pub board_area: Rect,
    pub tray_area: Rect,
    pub top_border: (usize, usize),
    pub bottom_border: (usize, usize),
    pub rows: Vec<(usize, usize)>,
    pub cols: Vec<(usize, usize)>,
    pub traycols: Vec<(usize, usize)>,
    pub cells: Vec<Rect>,
}

fn close(a: u32, b: u32, tol: u32) -> bool {
    (a as i32 - b as i32).abs() < tol as i32
}

impl<'a> BoardLayout<'a> {
    pub fn new(img: &'a ImageBuffer<Luma<u8>, Vec<u8>>) -> BoardLayout<'a> {
        let integral = integral_image::<_, u64>(img);
        let integral_squared = integral_squared_image::<_, u64>(img);
        BoardLayout {
            img,
            integral,
            integral_squared,
            board_area: Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            tray_area: Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            top_border: (0, 0),
            bottom_border: (0, 0),
            rows: Vec::new(),
            cols: Vec::new(),
            traycols: Vec::new(),
            cells: Vec::new(),
        }
    }

    pub fn segment(&mut self) -> Result<(), Error> {
        let mut state = Segment::LookForTopBorder(0);
        // let rowstats = BoardLayout::rowstats(img);
        let rowstats = self.rowstats();
        let tol = 2;
        for (i, &(sum, var)) in rowstats.iter().enumerate() {
            match state {
                Segment::LookForTopBorder(n) => {
                    if close(sum, 51, tol) && (var < 25) {
                        self.top_border.0 = i;
                        state = Segment::LookForTopBorder(n + 1);
                    }
                    if n > 3 {
                        state = Segment::InTopBorder;
                    }
                }
                Segment::InTopBorder => {
                    if close(sum, 24, tol) {
                        self.top_border.1 = i;
                        state = Segment::LookForRisingEdge(0);
                    }
                }
                Segment::LookForRisingEdge(n) => {
                    if sum > 24 + tol {
                        self.rows.push((i, 0));
                        state = Segment::InTile(n);
                    }
                }
                Segment::InTile(n) => {
                    if close(sum, 24, tol) {
                        self.rows[n].1 = i - 1;
                        if n < 14 {
                            state = Segment::LookForRisingEdge(n + 1);
                        } else {
                            state = Segment::LookForBottomBorder(0);
                        }
                    }
                }
                Segment::LookForBottomBorder(n) => {
                    if close(sum, 51, tol) && (var < 25) {
                        self.bottom_border.0 = i;
                        state = Segment::LookForBottomBorder(n + 1);
                    }
                    if n > 5 {
                        state = Segment::InBottomBorder;
                    }
                }
                Segment::InBottomBorder => {
                    if close(sum, 24, tol) && (var < 10) {
                        self.bottom_border.1 = i;
                        // println!("{}: Look for tray", i);
                        state = Segment::LookForTray;
                    }
                }
                Segment::LookForTray => {
                    // println!("{}: Look for tray: {} {}", i, sum, var);
                    // if sum > 24 + tol {
                    if var > 100 {
                        self.tray_area.y = i as u32;
                        state = Segment::InTray;
                    }
                }
                Segment::InTray => {
                    // println!("{}: Intray: {} {}", i, sum, var);
                    if close(sum, 24, tol) && (var == 0) {
                        self.tray_area.height = i as u32 - self.tray_area.y;
                        // println!("Done!");
                        state = Segment::Done;
                    }
                }
                Segment::Done => {}
            }
        }
        if state != Segment::Done {
            return Err(Error::LayoutFailed(state));
        }
        self.board_area.x = 0;
        self.board_area.width = self.img.width();
        self.board_area.y = self.rows[0].0 as u32;
        self.board_area.height = self.rows[14].1 as u32 - self.board_area.y;

        self.tray_area.x = 0;
        self.tray_area.width = self.img.width();

        // let mean_tile_width = (self.board.1 - self.board.0) / 15;
        // the board area should be approximately square
        let w = self.board_area.width;
        let h = self.board_area.height;
        let aspect_ratio = h as f32 / w as f32;
        if (aspect_ratio - 1.0).abs() > 0.02 {
            return Err(Error::BoardNotSquare(aspect_ratio));
        }
        self.cols = self.segment_board_columns()?;
        if state != Segment::Done {
            return Err(Error::LayoutFailed(state));
        }

        self.traycols = self.segment_tray_columns()?;
        Ok(())
    }

    fn segment_board_columns(&self) -> Result<Vec<(usize, usize)>, Error> {
        let mut cols = Vec::new();
        let mut state = Segment::LookForRisingEdge(0);
        let colstats = self.colstats();
        let tol = 2;
        for (i, &(sum, _var)) in colstats.iter().enumerate() {
            match state {
                Segment::LookForRisingEdge(n) => {
                    if sum > 24 + tol {
                        cols.push((i, 0));
                        // println!("{}: InTile {}", i, n);
                        state = Segment::InTile(n);
                    }
                }
                Segment::InTile(n) => {
                    if close(sum, 24, tol) {
                        cols[n].1 = i - 1;
                        if n < 14 {
                            state = Segment::LookForRisingEdge(n + 1);
                        } else {
                            state = Segment::Done;
                        }
                    }
                }
                Segment::Done => {}
                _ => panic!("Unexpected segment state"),
            }
        }
        if state != Segment::Done {
            return Err(Error::LayoutFailed(state));
        }
        Ok(cols)
    }

    fn segment_tray_columns(&self) -> Result<Vec<(usize, usize)>, Error> {
        let mut cols = Vec::new();
        let mut state = Segment::LookForRisingEdge(0);
        let traystats = self.traystats();
        let tol = 2;
        for (i, &(sum, var)) in traystats.iter().enumerate() {
            match state {
                Segment::LookForRisingEdge(n) => {
                    if sum > 50 {
                        cols.push((i, 0));
                        // println!("# {}: InTile {}", i, n);
                        state = Segment::InTile(n);
                    }
                }
                Segment::InTile(n) => {
                    if close(sum, 24, tol) && (var == 0) {
                        cols[n].1 = i - 1;
                        if n < 6 {
                            state = Segment::LookForRisingEdge(n + 1);
                        } else {
                            state = Segment::Done;
                        }
                    }
                }
                Segment::Done => {}
                _ => panic!("Unexpected segment state"),
            }
        }
        // if state != Segment::Done {
        //     return Err(Error::LayoutFailed(state));
        // }
        Ok(cols)
    }

    pub fn rowstats(&self) -> Vec<(u32, u32)> {
        let (w, h) = self.img.dimensions();
        self.stats(
            Rect {
                x: 0,
                y: 0,
                width: w,
                height: h,
            },
            true,
        )
    }

    pub fn colstats(&self) -> Vec<(u32, u32)> {
        self.stats(self.board_area, false)
    }

    pub fn traystats(&self) -> Vec<(u32, u32)> {
        self.stats(self.tray_area, false)
    }

    fn stats(&self, bounds: Rect, horizontal: bool) -> Vec<(u32, u32)> {
        let mut stats = Vec::new();
        let (dim, count) = if horizontal {
            (bounds.height, bounds.width)
        } else {
            (bounds.width, bounds.height)
        };
        let area = |i| {
            if horizontal {
                (bounds.x, i, bounds.x + bounds.width - 1, i)
            } else {
                (i, bounds.y, i, bounds.y + bounds.height - 1)
            }
        };
        for i in 0..dim {
            let (left, top, right, bottom) = area(i);
            let sum = sum_image_pixels(&self.integral, left, top, right, bottom);
            let var = variance(
                &self.integral,
                &self.integral_squared,
                left,
                top,
                right,
                bottom,
            );
            stats.push((sum[0] as u32 / count, var as u32));
        }
        stats
    }

    /// Create tile sub images for board and tray
    pub fn get_cells(&mut self) {
        let mut cells = Vec::new();
        // make sure we have 15x15 tiles
        assert_eq!(self.rows.len(), 15);
        assert_eq!(self.cols.len(), 15);
        // find out what size our tiles should be
        let tiles_height: usize = self.rows.iter().map(|&(y0,y1)| y1 - y0).sum();
        let tiles_width: usize = self.cols.iter().map(|&(x0, x1)| x1 - x0).sum();
        let (tile_height, tile_width) = ((tiles_height as u32) / 15, (tiles_width as u32) / 15);
        println!("{} {} {} {}", self.board_area.width, self.board_area.height, tile_width, tile_height);
        for &(y0, _y1) in self.rows.iter() {
            for &(x0, _x1) in self.cols.iter() {
                let cell = Rect{x: x0 as u32, y: y0 as u32, width: tile_width, height: tile_height};
                cells.push(cell);
            }
        }
        self.cells = cells;
    }

    pub fn get_tile_index(&self) -> Vec<usize> {
        let mut index = Vec::new();
        for (i, cell ) in self.cells.iter().enumerate() {
            let (left, top,right,bottom) = (cell.x, cell.y, cell.x + cell.width - 1, cell.y + cell.height - 1);
            let sum = sum_image_pixels(&self.integral, left, top, right, bottom);
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
        let index = self.get_tile_index();
        if index.is_empty() {
            return None;
        }
        let nimages = index.len() as f64;
        let n = nimages.sqrt().ceil(); // size of collage square
        let (ncols, nrows) = (n as u32, ((nimages / n).ceil()) as u32);
       
        let cell = self.cells[0];
        let (w, h) = (cell.width, cell.height);
        let mut img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(w * ncols, h * nrows);
        eprintln!("Create collage {} cols x {} rows, size {},{}", ncols, nrows, img.width(), img.height());
        for (i, &idx) in index.iter().enumerate() {
            let cell = self.cells[idx];
            // create destination sub image in collage:
            let (row, col) = ((i as u32 / ncols), (i as u32 % ncols));
            let mut dest = img.sub_image(col*w, row*h, w, h);
            // create source sub image
            let src = self.img.view(cell.x, cell.y, w, h);
            // copy the pixels
            dest.copy_from(&src, 0, 0).unwrap();
            eprintln!("Tile {}: Copy {}x{} from {},{} to {},{}", i, w,h, cell.x, cell.y, col*w, row*h);
           
        }
        Some(img)
    }

    pub fn read_templates(&mut self) -> Result<Vec<(String, DynamicImage)>,Error> {
        let mut entries = fs::read_dir("templates").unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>().unwrap();
        entries.sort();
        let mut templates = Vec::new();
        for path in entries {
            if let Some(stem) = path.file_stem() {
                let key = stem.to_str().unwrap();
                let template = ImageReader::open(&path).unwrap().decode().unwrap();
                templates.push((String::from(key), template));
            }
        }
        Ok(templates)
    }
}

/// This is a modified copy of [imageproc::integral_image::variance]()
pub fn variance(
    integral_image: &ImageBuffer<Luma<u64>, Vec<u64>>,
    integral_squared_image: &ImageBuffer<Luma<u64>, Vec<u64>>,
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
) -> f64 {
    // TODO: same improvements as for sum_image_pixels, plus check that the given rect is valid.
    let n = (right - left + 1) as f64 * (bottom - top + 1) as f64;
    let sum_sq = sum_image_pixels(integral_squared_image, left, top, right, bottom)[0];
    let sum = sum_image_pixels(integral_image, left, top, right, bottom)[0];
    (sum_sq as f64 - (sum as f64).powi(2) / n) / n
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use image::io::Reader as ImageReader;
    // use image::imageops;

    #[test]
    fn test() -> Result<()> {
        let img = ImageReader::open("screenshots/screenshot_1080x2160_2.png")?.decode()?;
        let gray = img.into_luma8();
        let mut layout = BoardLayout::new(&gray);
        layout.segment()?;
        // let stats = layout.rowstats();
        // for (i, &(mean, var)) in stats.iter().enumerate() {
        //     println!("{} {} {}", i, mean, var);
        // }
        layout.get_cells();
        let tiles = layout.get_tile_index();
        println!("{:?}", tiles);

        if let Some(collage) = layout.collage() {
            // let resized = imageops::resize(&collage, 640, 576, imageops::FilterType::Triangle);
            collage.save("collage.png")?;
        }

        Ok(())
    }

    #[test]
    fn test_templates() -> Result<()> {
        let img = ImageReader::open("screenshots/screenshot_1080x2160_2.png")?.decode()?;
        let gray = img.into_luma8();
        let mut layout = BoardLayout::new(&gray);
        layout.segment()?; 
        layout.read_templates()?;
        Ok(())
    }
}
