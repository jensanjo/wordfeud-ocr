use crate::error::Error;
use image::{math::Rect, GrayImage, ImageBuffer, Luma};
use imageproc::integral_image::{integral_image, integral_squared_image, sum_image_pixels};

pub const THRESHOLD: f64 = 0.65;

type IntegralImage = ImageBuffer<Luma<u64>, Vec<u64>>;

// #[derive(Debug)]
pub struct Layout<'a> {
    pub img: &'a GrayImage,
    pub integral: IntegralImage,
    pub integral_squared: IntegralImage,
    pub screen: Rect,
    pub board_area: Rect,
    pub tray_area: Rect,
    pub rows: Vec<(usize, usize)>,
    pub cols: Vec<(usize, usize)>,
    pub trayrows: Vec<(usize, usize)>,
    pub traycols: Vec<(usize, usize)>,
}

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

fn close(a: u32, b: u32, tol: u32) -> bool {
    (a as i32 - b as i32).abs() < tol as i32
}

fn bounds(rect: Rect) -> (u32, u32, u32, u32) {
    (rect.x, rect.y, rect.width, rect.height)
}

impl<'a> Layout<'a> {
    pub fn new(img: &'a GrayImage) -> Layout<'a> {
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
        let tray_area = Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
        Layout {
            img,
            integral,
            integral_squared,
            screen,
            board_area,
            tray_area,
            rows: Vec::new(),
            cols: Vec::new(),
            trayrows: Vec::new(),
            traycols: Vec::new(),
        }
    }

    pub fn segment(&mut self) -> Result<(), Error> {
        let mut state = Segment::LookForTopBorder(0);
        let rowstats = self.stats(bounds(self.screen), true);
        let (mut tray_y, mut tray_height) = (0, 0);
        let tol = 2;
        for (i, &(sum, var)) in rowstats.iter().enumerate() {
            match state {
                Segment::LookForTopBorder(n) => {
                    if close(sum, 51, tol) && (var < 25) {
                        state = Segment::LookForTopBorder(n + 1);
                    }
                    if n > 3 {
                        state = Segment::InTopBorder;
                    }
                }
                Segment::InTopBorder => {
                    if close(sum, 24, tol) {
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
                        state = Segment::LookForBottomBorder(n + 1);
                    }
                    if n > 5 {
                        state = Segment::InBottomBorder;
                    }
                }
                Segment::InBottomBorder => {
                    if close(sum, 24, tol) && (var < 10) {
                        state = Segment::LookForTray;
                    }
                }
                Segment::LookForTray => {
                    if var > 100 {
                        tray_y = i as u32;
                        state = Segment::InTray;
                    }
                }
                Segment::InTray => {
                    // println!("{}: Intray: {} {}", i, sum, var);
                    if close(sum, 24, tol) && (var == 0) {
                        tray_height = i as u32 - tray_y;
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
        let y0 = self.rows[0].0 as u32;
        let y1 = self.rows[14].1 as u32;
        self.board_area = Rect {
            x: 0,
            y: y0,
            width: self.screen.width,
            height: y1 - y0,
        };
        self.tray_area = Rect {
            x: 0,
            y: tray_y,
            width: self.screen.width,
            height: tray_height,
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
        self.trayrows
            .push((tray_y as usize, (tray_y + tray_height - 1) as usize));
        self.traycols = self.segment_tray_columns()?;
        Ok(())
    }

    fn segment_columns(
        threshold: u32,
        maxcols: usize,
        colstats: &[(u32, u32)],
    ) -> Result<Vec<(usize, usize)>, Error> {
        let mut cols = Vec::new();
        let mut state = Segment::LookForRisingEdge(0);
        let tol = 2;
        for (i, &(sum, var)) in colstats.iter().enumerate() {
            match state {
                Segment::LookForRisingEdge(n) => {
                    if sum > threshold + tol {
                        cols.push((i, 0));
                        // println!("{}: InTile {}", i, n);
                        state = Segment::InTile(n);
                    }
                }
                Segment::InTile(n) => {
                    if close(sum, 24, tol) && (var == 0) {
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

    fn segment_tray_columns(&self) -> Result<Vec<(usize, usize)>, Error> {
        let colstats = self.stats(bounds(self.tray_area), false);
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

    /// Create tile sub images for board
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
