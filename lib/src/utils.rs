use image::imageops::{resize, FilterType};
use image::math::Rect;
use image::{GenericImage, GenericImageView, GrayImage, ImageBuffer, Luma, SubImage};
use imageproc::map::map_pixels;
use std::path::{Path, PathBuf};

/// Create a collage from parts of a source image.
/// TODO: All parts must be the same size
pub fn collage(source: &GrayImage, parts: &[Rect], maxrows: Option<u32>) -> GrayImage {
    if parts.is_empty() {
        return GrayImage::new(0,0);
    }
    let nimages = parts.len();
    let mut nrows = (nimages as f64).sqrt().floor() as u32; // size of collage square
    if let Some(maxrows) = maxrows {
        nrows = std::cmp::min(nrows, maxrows);
    }
    let ncols = (nimages as f64 / nrows as f64).ceil() as u32;
    let cell = parts[0];
    let (w, h) = (cell.width, cell.height);
    let mut collage: GrayImage = ImageBuffer::new(w * ncols, h * nrows);
    let filter = FilterType::Lanczos3;
    for (i, &cell) in parts.iter().enumerate() {
        // create destination sub image in collage:
        let (row, col) = ((i as u32 / ncols), (i as u32 % ncols));
        let mut dest: SubImage<&mut GrayImage> = collage.sub_image(col * w, row * h, w, h);

        let src: SubImage<&GrayImage> = source.view(cell.x, cell.y, cell.width, cell.height);
        // create source sub image
        if cell.height != h {
            let resized = resize(&src, w, h, filter);
            dest.copy_from(&resized, 0, 0).unwrap();
        } else {
            // copy the pixels
            dest.copy_from(&src, 0, 0).unwrap();
        }
    }
    collage
}

/// Save tiles as templates
pub fn save_templates<P: AsRef<Path>>(
    savedir: P,
    img: &GrayImage,
    cells: &[Rect],
    state: &[Vec<String>],
) {
    let threshold = 150;
    for (i, cell) in cells.iter().enumerate() {
        let (row, col) = (i / 15, i % 15);
        let letter = &state[row][col].to_uppercase();
        if letter != "." {
            let tile = img.view(cell.x + 7, cell.y + 4, 38, 60).to_image();
            let bw = map_pixels(&tile, |_x, _y, p| {
                if p[0] > threshold {
                    Luma([255u8])
                } else {
                    Luma([0])
                }
            });
            let mut path = PathBuf::new();
            path.push(savedir.as_ref());
            path.push(format!("{}.png", letter));
            if !path.exists() {
                println!("save {}.png", letter);
                bw.save(path).unwrap();
            }
        }
    }
}
