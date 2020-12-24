use anyhow::{Context, Result};
// use image::{imageops::FilterType, GenericImageView};
use std::time::Instant;
use wordfeud_ocr::Board;

fn run() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .expect("Usage: recognize SCREENSHOT");
    let t0 = Instant::now();
    let img = image::open(&path).with_context(|| format!("Failed to open {}", path))?;
    let gray = img.into_luma8();
    // let layout = Layout::new(&gray).segment()?;
    let board = Board::new();

    let res = board.recognize_screenshot(&gray)?;
    println!("recognize screenshot took {:?}", t0.elapsed());
    println!("{:?}", res);

    // save templates
    use wordfeud_ocr::{Layout, save_templates};
    let layout = Layout::new(&gray).segment()?;
    let cells = Layout::get_cells(&layout.rows, &layout.cols);
    save_templates("out", &gray, &cells, &res.tiles_ocr);
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{:?}", err);
    }
}
