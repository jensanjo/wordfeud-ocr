use anyhow::{Context, Result};
use wordfeud_ocr::{collage, Board};

fn run() -> Result<()> {
    let path = std::env::args().nth(1).unwrap();

    let gray = image::open(&path)
        .with_context(|| format!("Failed to open {}", path))?
        .into_luma8();
    eprintln!("read image from {}", path);
    let mut board = Board::new(&gray);
    board.layout.segment()?;
    let cells = board.layout.get_cells();
    let index = board.layout.get_tile_index(&cells);
    let mut tiles: Vec<_> = index.iter().map(|&i| cells[i]).collect();

    // get tray tiles and resize them to match the board tiles.
    // let cell = board.cells[0];
    let cells = board.layout.get_tray_cells();
    tiles.extend(cells);

    if let Some(collage) = collage(&gray, &tiles, None) {
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
