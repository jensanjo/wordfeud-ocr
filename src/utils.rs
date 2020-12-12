use image::imageops::{resize, FilterType};
use image::math::Rect;
use image::{GenericImage, GenericImageView, GrayImage, ImageBuffer, SubImage};

/// Create a collage from parts of a source image.
/// TODO: All parts must be the same size
pub fn collage(source: &GrayImage, parts: &[Rect], maxrows: Option<u32>) -> Option<GrayImage> {
    if parts.is_empty() {
        return None;
    }
    let nimages = parts.len();
    let mut nrows = (nimages as f64).sqrt().floor() as u32; // size of collage square
    if let Some(maxrows) = maxrows {
        nrows = maxrows;
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
    Some(collage)
}
