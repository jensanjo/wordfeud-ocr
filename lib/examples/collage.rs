use anyhow::{Context, Result};
use wordfeud_ocr::{collage, Layout};

fn run() -> Result<()> {
    let path = std::env::args().nth(1).expect("Usage: collage SCREENSHOT");

    let gray = image::open(&path)
        .with_context(|| format!("Failed to open {}", path))?
        .into_luma8();
    eprintln!("read image from {}", path);
    let layout = Layout::new(&gray).segment()?;
    let cells = Layout::get_cells(&layout.rows, &layout.cols);
    let index = layout.get_tile_index(&cells);
    let mut tiles: Vec<_> = index.iter().map(|&i| cells[i]).collect();

    // get rack tiles and resize them to match the board tiles.
    let cells = Layout::get_cells(&layout.rack_rows, &layout.rack_cols);
    tiles.extend(cells);

    let collage = collage(&gray, &tiles, None);
    collage.save("collage.png")?;

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{:?}", err);
    }
}
