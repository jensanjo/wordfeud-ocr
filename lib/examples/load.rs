use anyhow::{Context, Result};
use image::{io::Reader as ImageReader, GenericImageView};
use imageproc::drawing::draw_antialiased_line_segment_mut;
use imageproc::pixelops::interpolate;
use wordfeud_ocr::Layout;

fn run() -> Result<()> {
    let path = std::env::args().nth(1).expect("Usage: load SCREENSHOT");
    eprintln!("read image from {}", path);
    let img = ImageReader::open(&path)
        .with_context(|| format!("Failed to open {}", path))?
        .decode()?;

    let gray = img.clone().into_luma8();
    let layout = Layout::new(&gray).segment()?;

    // println!("rack stats:");
    // let rackstats = layout.rackstats();
    // for (i, (mean, var)) in rackstats.iter().enumerate() {
    //     println!("{} {} {}", i, mean, var);
    // }

    eprintln!("board area: {:?}", layout.board_area);
    for (i, &(y0, y1)) in layout.rows.iter().enumerate() {
        eprintln!("  Row {}: {},{} {}", i, y0, y1, y1 - y0);
    }
    for (i, &(x0, x1)) in layout.cols.iter().enumerate() {
        eprintln!("  Col {}: {},{} {}", i, x0, x1, x1 - x0);
    }
    eprintln!("rack area: {:?}", layout.rack_area);
    for (i, &(x0, x1)) in layout.rack_cols.iter().enumerate() {
        eprintln!("  Col {}: {},{} {}", i, x0, x1, x1 - x0);
    }

    // draw the tile rows in the image
    let red = image::Rgba([255, 0, 0, 255]);
    let blue = image::Rgba([0, 0, 255, 255]);
    let w = layout.board_area.width as i32;
    let mut img = img;
    for &(y0, y1) in layout.rows.iter() {
        draw_antialiased_line_segment_mut(
            &mut img,
            (0, y0 as i32),
            (w, y0 as i32),
            red,
            interpolate,
        );
        draw_antialiased_line_segment_mut(
            &mut img,
            (0, y1 as i32),
            (w, y1 as i32),
            blue,
            interpolate,
        );
    }
    let (y, h) = (layout.board_area.y, layout.board_area.height);
    let (y0, y1) = (y as i32, (y + h) as i32);
    for &(x0, x1) in layout.cols.iter() {
        draw_antialiased_line_segment_mut(
            &mut img,
            (x0 as i32, y0),
            (x0 as i32, y1),
            red,
            interpolate,
        );
        draw_antialiased_line_segment_mut(
            &mut img,
            (x1 as i32, y0),
            (x1 as i32, y1),
            blue,
            interpolate,
        );
    }
    let (y, h) = (layout.rack_area.y, layout.rack_area.height);
    let (y0, y1) = (y as i32, (y + h) as i32);
    for &(x0, x1) in layout.rack_cols.iter() {
        draw_antialiased_line_segment_mut(
            &mut img,
            (x0 as i32, y0),
            (x0 as i32, y1),
            red,
            interpolate,
        );
        draw_antialiased_line_segment_mut(
            &mut img,
            (x1 as i32, y0),
            (x1 as i32, y1),
            blue,
            interpolate,
        );
    }

    img.save("screenshot.png")?;

    let r = layout.board_area;
    img.view(r.x, r.y, r.width, r.height)
        .to_image()
        .save("board.png")?;

    let r = layout.rack_area;
    img.view(r.x, r.y, r.width, r.height)
        .to_image()
        .save("rack.png")?;
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
    }
}
