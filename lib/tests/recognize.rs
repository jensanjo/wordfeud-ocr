use anyhow::{Context, Result};
use std::time::Instant;
use wordfeud_ocr::Board;

fn test_recognize_screenshot(screenshot_path: &str, expect: &str) -> Result<()> {
    let img = image::open(&screenshot_path)
        .with_context(|| format!("Failed to open {}", screenshot_path))?;
    let gray = img.into_luma8();
    let board = Board::new();
    let now = Instant::now();
    let res = board.recognize_screenshot(&gray)?;
    println!("Recognize screenshot took {:?}", now.elapsed());
    let ocr = format!(
        "Screenshot: {}\n\nTiles:\n{}\n\nLetters: {}\n\nGrid:\n{}\n",
        screenshot_path, res.tiles_ocr, res.rack_ocr, res.grid_ocr
    );
    println!("{}", ocr);
    assert_eq!(ocr, expect);
    Ok(())
}

#[test]
#[ignore]
fn test_english_screenshot() -> Result<()> {
    test_recognize_screenshot(
        "screenshots/screenshot_english.png",
        include_str!("screenshot_english.expect"),
    )?;
    Ok(())
}

#[test]
#[ignore]
fn test_dutch_screenshot() -> Result<()> {
    test_recognize_screenshot(
        "screenshots/screenshot_dutch.png",
        include_str!("screenshot_dutch.expect"),
    )?;
    Ok(())
}

#[test]
#[ignore]
fn test_swedish_screenshot() -> Result<()> {
    test_recognize_screenshot(
        "screenshots/screenshot_swedish.png",
        include_str!("screenshot_swedish.expect"),
    )?;
    Ok(())
}
