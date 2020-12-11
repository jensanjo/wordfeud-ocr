use anyhow::{Context, Result};
use wordfeud_ocr::Board;

fn run() -> Result<()> {
    let path = std::env::args().nth(1).unwrap();
    eprintln!("read image from {}", path);
    let gray = image::open(&path)
        .with_context(|| format!("Failed to open {}", path))?
        .into_luma8();
    let mut board = Board::new(&gray);
    board.layout.segment()?;
    board.get_cells();
    board.get_tile_index();

    if let Some(collage) = board.collage() {
        // let resized = imageops::resize(&collage, 640, 576, imageops::FilterType::Triangle);
        collage.save("collage.png")?;
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{:?}", err);
    }
}
