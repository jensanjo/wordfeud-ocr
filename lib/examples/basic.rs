use anyhow::Result;
use wordfeud_ocr::Board;

fn main() -> Result<()> {
    let path = "screenshots/screenshot_english.png";
    let gray = image::open(path)?.into_luma8();
    let board = Board::new();
    let result = board.recognize_screenshot(&gray)?;
    println!("Tiles:\n{}", result.tiles_ocr);
    Ok(())
}
