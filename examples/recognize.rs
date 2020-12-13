use anyhow::{Context, Result};
use std::time::Instant;
use wordfeud_ocr::{Board, Layout};
use image::imageops::FilterType;

fn run() -> Result<()> {
    let path = std::env::args().nth(1).unwrap();
    let t0 = Instant::now();
    let img = image::open(&path).with_context(|| format!("Failed to open {}", path))?;
    let gray = img.into_luma8();
    let mut board = Board::new(&gray).read_templates()?;

    board.layout.segment()?;
    let cells = Layout::get_cells(&board.layout.rows, &board.layout.cols);
    let tile_index = board.layout.get_tile_index(&cells);
    let now = Instant::now();
    let (ocr, _matches) = board.recognize_tiles(&tile_index, &cells, &board.templates, (15, 15), None);
    let dt = now.elapsed();
    println!("match templates took {:?} {:?}", dt, t0.elapsed());
    let ocr = ocr
        .into_iter()
        .map(|v| v.join(""))
        .collect::<Vec<String>>()
        .join("\n");
    println!("{}", ocr);

    // recognize tray tiles
    let cells = Layout::get_cells(&board.layout.trayrows, &board.layout.traycols);
    let index: Vec<usize> = (0..cells.len()).into_iter().collect();
    let resize_to = Some((67, 67, FilterType::Lanczos3));
    let (ocr, _matches) = board.recognize_tiles(&index, &cells, &board.templates, (1,7), resize_to);
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
