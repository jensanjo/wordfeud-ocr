use anyhow::{Context, Result};
use std::time::Instant;
use wordfeud_ocr::Board;

fn run() -> Result<()> {
    let path = std::env::args().nth(1).unwrap();
    let t0 = Instant::now();
    let img = image::open(&path).with_context(|| format!("Failed to open {}", path))?;
    let gray = img.into_luma8();
    let mut board = Board::new(&gray);

    board.layout.segment()?;
    let cells = board.layout.get_cells();
    let tile_index = board.layout.get_tile_index(&cells);
    board.read_templates()?;
    let now = Instant::now();
    let (ocr, _matches) = board.recognize_tiles(&tile_index, &cells, (15, 15));
    let dt = now.elapsed();
    println!("match templates took {:?} {:?}", dt, t0.elapsed());
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
