use crate::error::Error;
use image::{math::Rect, GrayImage, ImageBuffer, Luma};
use imageproc::integral_image::{integral_image, integral_squared_image, sum_image_pixels};
use log::debug;

pub const THRESHOLD: f64 = 0.65;

type IntegralImage = ImageBuffer<Luma<u64>, Vec<u64>>;

/// Represents the recognized layout of a wordfeud board.
pub struct Layout {
    integral: IntegralImage,
    integral_squared: IntegralImage,
    /// The screen area (the entire screenshot)
    pub screen: Rect,
    /// The board area (a 15 x 15 grid)
    pub board_area: Rect,
    /// The rack area (1 row x 7 column grid)
    pub rack_area: Rect,
    /// The start and end `y` coordinate of the board rows
    pub rows: Vec<(usize, usize)>,
    /// The start and end `x` coordinate of the board columns
    pub cols: Vec<(usize, usize)>,
    /// The start and end `y` coordinate of the rack row
    pub rack_rows: Vec<(usize, usize)>,
    /// The start and end `x` coordinate of the rack columns
    pub rack_cols: Vec<(usize, usize)>,
}

#[derive(Debug, PartialEq)]
pub enum Segment {
    LookForTopBorder(usize),
    InTopBorder,
    LookForRisingEdge(usize),
    InTile(usize),
    LookForBottomBorder(usize),
    InBottomBorder,
    LookForRack,
    InRack,
    Done,
}

fn close(a: u32, b: u32, tol: u32) -> bool {
    (a as i32 - b as i32).abs() <= tol as i32
}

fn bounds(rect: Rect) -> (u32, u32, u32, u32) {
    (rect.x, rect.y, rect.width, rect.height)
}

impl Layout {
    /// Return a new Layout for `img`.
    ///
    /// Only the screen area is set to the image bounding rect. Empty board_area and rack_area.
    /// Empty board and rack rows and columns.
    pub fn new(img: &GrayImage) -> Layout {
        let integral: IntegralImage = integral_image::<_, u64>(img);
        let integral_squared: IntegralImage = integral_squared_image::<_, u64>(img);
        let screen = Rect {
            x: 0,
            y: 0,
            width: img.width(),
            height: img.height(),
        };
        let board_area = Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
        let rack_area = Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
        Layout {
            integral,
            integral_squared,
            screen,
            board_area,
            rack_area,
            rows: Vec::new(),
            cols: Vec::new(),
            rack_rows: Vec::new(),
            rack_cols: Vec::new(),
        }
    }

    /// Segment the screenshot:
    /// - locate the board and rack area
    /// - within board and rack area:
    ///     - locate the start and end of each row and column
    ///
    /// After segmentation all the `Layout` fields are properly set.
    /// # Errors
    /// If the screenshot can not be properly segmented.
    /// # Example
    /// ```
    /// # use wordfeud_ocr::{Layout, Error};
    /// # use anyhow::Result;
    /// let path = "tests/screenshot_english.png";
    /// let gray = image::open(path)?.into_luma8();
    /// let layout = Layout::new(&gray).segment()?;
    /// assert_eq!(layout.cols.len(), 15);
    /// assert_eq!(layout.rows.len(), 15);
    /// # Ok::<(), Error>(())
    /// ``` 
    pub fn segment(mut self) -> Result<Self, Error> {
        let mut state = Segment::LookForTopBorder(0);
        let rowstats = self.stats(bounds(self.screen), true);
        let (mut rack_y, mut rack_height) = (0, 0);
        let tol = 2;
        for (i, &(sum, var)) in rowstats.iter().enumerate() {
            debug!("{} {} {}", i, sum, var);
            match state {
                Segment::LookForTopBorder(n) => {
                    if close(sum, 51, tol) && (var < 25) {
                        state = Segment::LookForTopBorder(n + 1);
                    }
                    if n > 3 {
                        state = Segment::InTopBorder;
                        debug!("# {i} InTopBorder");
                    }
                }
                Segment::InTopBorder => {
                    if close(sum, 24, tol) {
                        state = Segment::LookForRisingEdge(0);
                        debug!("# {i} LookForRisingEdge(0)");
                    }
                }
                Segment::LookForRisingEdge(n) => {
                    if sum > 24 + tol {
                        self.rows.push((i, 0));
                        state = Segment::InTile(n);
                        debug!("# {i} InTile({n})");
                    }
                }
                Segment::InTile(n) => {
                    if close(sum, 24, tol) {
                        self.rows[n].1 = i - 1;
                        if n < 14 {
                            state = Segment::LookForRisingEdge(n + 1);
                            debug!("# {i} LookForRisingEdge({})", n + 1);
                        } else {
                            state = Segment::LookForBottomBorder(0);
                            debug!("# {i} LookForBottomBorder(0)");
                        }
                    }
                }
                Segment::LookForBottomBorder(n) => {
                    if close(sum, 51, tol) && (var < 25) {
                        state = Segment::LookForBottomBorder(n + 1);
                        debug!("# {i} LookForBottomBorder({})", n + 1);
                    }
                    if n > 5 {
                        state = Segment::InBottomBorder;
                        debug!("# {i} InBottomBorder");
                    }
                }
                Segment::InBottomBorder => {
                    if close(sum, 24, tol) && (var < 10) {
                        state = Segment::LookForRack;
                        debug!("# {i} LookForRack");
                    }
                }
                Segment::LookForRack => {
                    if var > 100 {
                        rack_y = i as u32;
                        state = Segment::InRack;
                        debug!("# {i} InRack");
                    }
                }
                Segment::InRack => {
                    if close(sum, 24, tol) && (var == 0) {
                        rack_height = i as u32 - rack_y;
                        debug!("# {i} Done");
                        state = Segment::Done;
                    }
                }
                Segment::Done => {}
            }
        }
        if state != Segment::Done {
            return Err(Error::LayoutFailed(state));
        }
        let y0 = self.rows[0].0 as u32;
        let y1 = self.rows[14].1 as u32;
        self.board_area = Rect {
            x: 0,
            y: y0,
            width: self.screen.width,
            height: y1 - y0,
        };
        self.rack_area = Rect {
            x: 0,
            y: rack_y,
            width: self.screen.width,
            height: rack_height,
        };

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
        self.rack_rows
            .push((rack_y as usize, (rack_y + rack_height - 1) as usize));
        self.rack_cols = self.segment_rack_columns()?;
        Ok(self)
    }

    fn segment_columns(
        threshold: u32,
        maxcols: usize,
        colstats: &[(u32, u32)],
    ) -> Result<Vec<(usize, usize)>, Error> {
        let mut cols = Vec::new();
        let mut state = Segment::LookForRisingEdge(0);
        let tol = 5;
        for (i, &(sum, _var)) in colstats.iter().enumerate() {
            match state {
                Segment::LookForRisingEdge(n) => {
                    if sum > threshold + tol {
                        cols.push((i, 0));
                        // println!("{}: InTile {}", i, n);
                        state = Segment::InTile(n);
                    }
                }
                Segment::InTile(n) => {
                    if close(sum, 24, tol) /*&& (var == 0)*/ {
                        cols[n].1 = i - 1;
                        if n + 1 < maxcols {
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
    fn segment_board_columns(&self) -> Result<Vec<(usize, usize)>, Error> {
        let colstats = self.stats(bounds(self.board_area), false);
        Self::segment_columns(24, 15, &colstats)
    }

    fn segment_rack_columns(&self) -> Result<Vec<(usize, usize)>, Error> {
        let colstats = self.stats(bounds(self.rack_area), false);
        Self::segment_columns(48, 7, &colstats)
    }

    fn stats(&self, bounds: (u32, u32, u32, u32), horizontal: bool) -> Vec<(u32, u32)> {
        let mut stats = Vec::new();
        let (x, y, w, h) = bounds;
        let (dim, count) = if horizontal { (h, w) } else { (w, h) };
        let area = |i| {
            if horizontal {
                (x, i, x + w - 1, i)
            } else {
                (i, y, i, y + h - 1)
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

    /// Create cell bounding rectangles from rows and columns,
    ///
    /// All cells have the same width and height.
    pub fn get_cells(rows: &[(usize, usize)], cols: &[(usize, usize)]) -> Vec<Rect> {
        let mut cells = Vec::new();
        if cols.is_empty() {
            return cells;
        }
        // find out what size our tiles should be
        let tiles_height: usize = rows.iter().map(|&(y0, y1)| y1 - y0).sum();
        let tiles_width: usize = cols.iter().map(|&(x0, x1)| x1 - x0).sum();
        let (tile_height, tile_width) = (
            (tiles_height / rows.len()) as u32,
            (tiles_width / cols.len()) as u32,
        );
        for &(y0, _y1) in rows.iter() {
            for &(x0, _x1) in cols.iter() {
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

    /// Returns the indices of all cells that contain a tile.
    /// 
    /// Whether a cell contains a tile or not is determined from the mean gray value of the cell area.
    /// A gray value of more than 65% (166 / 256) must be a tile.
    pub fn get_tile_index(&self, cells: &[Rect]) -> Vec<usize> {
        let mut index = Vec::new();
        for (i, &cell) in cells.iter().enumerate() {
            let (left, top, right, bottom) = (
                cell.x,
                cell.y,
                cell.x + cell.width - 1,
                cell.y + cell.height - 1,
            );
            let sum = sum_image_pixels(&self.integral, left, top, right, bottom);
            let mean = sum[0] as f64 / (cell.width * cell.height) as f64 / 256.;
            if mean > THRESHOLD {
                index.push(i);
            }
        }
        index
    }

    /// calculate mean pixel value in rect
    #[allow(dead_code)]
    pub fn mean(&self, rect: &Rect) -> f64 {
        let sum = sum_image_pixels(
            &self.integral,
            rect.x,
            rect.y,
            rect.x + rect.width - 1,
            rect.y + rect.height - 1,
        );
        let count = rect.width * rect.height;
        sum[0] as f64 / count as f64 / 256.
    }

    /// Calculate mean and standard deviation for the pixels in `rect`.
    pub fn area_stats(&self, rect: &Rect) -> (f64, f64) {
        let (left, top, right, bottom) = (
            rect.x,
            rect.y,
            rect.x + rect.width - 1,
            rect.y + rect.height - 1,
        );
        let sum = sum_image_pixels(&self.integral, left, top, right, bottom);
        let var = variance(
            &self.integral,
            &self.integral_squared,
            left,
            top,
            right,
            bottom,
        );
        let count = rect.width * rect.height;
        (sum[0] as f64 / count as f64 / 256., var.sqrt() / 256.)
    }
}

/// This is a modified copy of [imageproc::integral_image::variance]()
pub fn variance(
    integral_image: &IntegralImage,
    integral_squared_image: &IntegralImage,
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
    use image::{GenericImageView, GrayImage, ImageBuffer};

    #[test]
    fn test_subimg() {
        let img: GrayImage = ImageBuffer::new(540, 1080);
        let sub = img.view(10, 100, 500, 100);
        // let b = img.bounds();
        println!("{:?} {:?}", img.bounds(), sub.bounds());
    }
}
