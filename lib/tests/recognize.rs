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
fn test_screenshot_english() -> Result<()> {
    test_recognize_screenshot(
        "tests/screenshot_english.png",
        include_str!("screenshot_english.expect"),
    )?;
    Ok(())
}

#[test]
fn test_screenshot_dutch() -> Result<()> {
    test_recognize_screenshot(
        "tests/screenshot_dutch.png",
        include_str!("screenshot_dutch.expect"),
    )?;
    Ok(())
}

#[test]
fn test_screenshot_dutch_1() -> Result<()> {
    test_recognize_screenshot(
        "tests/screenshot_dutch_1.png",
        include_str!("screenshot_dutch_1.expect"),
    )?;
    Ok(())
}

#[test]
fn test_swedish_screenshot() -> Result<()> {
    test_recognize_screenshot(
        "tests/screenshot_swedish.png",
        include_str!("screenshot_swedish.expect"),
    )?;
    Ok(())
}

#[test]
fn test_screenshot_dutch_2() -> Result<()> {
    test_recognize_screenshot(
        "tests/screenshot_dutch_2.png",
        include_str!("screenshot_dutch_2.expect"),
    )?;
    Ok(())
}