use anyhow::{Context, Result};
use std::time::Instant;
use wordfeud_ocr::Board;

fn run() -> Result<()> {
    let path = std::env::args().nth(1).unwrap();
    let img = image::open(&path).with_context(|| format!("Failed to open {}", path))?;

    let gray = img.into_luma8();
    let mut board = Board::new(&gray);
    board.layout.segment()?;
    board.cells = board.get_cells();
    board.tile_index = board.get_tile_index();
    board.read_templates()?;
    let now = Instant::now();
    let (ocr, _matches) = board.recognize_tiles(&board.tile_index, &board.cells, (15, 15));
    let dt = now.elapsed();
    println!("match templates took {:?}", dt);
    let ocr = ocr
        .into_iter()
        .map(|v| v.join(""))
        .collect::<Vec<String>>()
        .join("\n");
    println!("{}", ocr);
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{:?}", err);
    }
}
