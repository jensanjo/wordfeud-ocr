use anyhow::{Context, Result};
// use image::{imageops::FilterType, GenericImageView};
use std::time::Instant;
use wordfeud_ocr::Board;

fn run() -> Result<()> {
    let path = std::env::args().nth(1).unwrap();
    let t0 = Instant::now();
    let img = image::open(&path).with_context(|| format!("Failed to open {}", path))?;
    let gray = img.into_luma8();
    // let layout = Layout::new(&gray).segment()?;
    let board = Board::new();

    let res = board.recognize_screenshot(&gray);
    println!("recognize screenshot took {:?}", t0.elapsed());
    println!("{:?}", res);

    // let cells = Layout::get_cells(&layout.rows, &layout.cols);
    // let tile_index = layout.get_tile_index(&cells);

    // // Recognize tiles
    // let now = Instant::now();
    // let (ocr, _matches) =
    //     board.recognize_tiles(&layout, &tile_index, &cells, &board.templates, (15, 15), None);
    // let dt = now.elapsed();
    // println!("recognize tiles took {:?} {:?}", dt, t0.elapsed());
    // for m in _matches {
    //     println!("{} {:.3} {:?}", m.1, m.2, m.3);
    // }
    // let ocr_string = ocr
    //     .iter()
    //     .map(|v| v.join(""))
    //     .collect::<Vec<String>>()
    //     .join("\n");
    // println!("{}", ocr_string);
    // // save_templates("out", &gray, &cells, &ocr);

    // for (&i, &name) in vec![94, 168].iter().zip(&["Æ", "Ø"]) {
    //     let cell = cells[i];
    //     let img = gray.view(cell.x, cell.y, cell.width, cell.height);
    //     let area = img.view(7, 4, 38, 60);
    //     // let area = img;
    //     area.to_image().save(format!("{}.png", name)).unwrap();
    // }

    // // Recognize board
    // let now = Instant::now();
    // let (ocr, _matches) = board.recognize_board(&layout, &cells, &board.bonus_templates, (15, 15));
    // let dt = now.elapsed();
    // println!("recognize board took {:?} {:?}", dt, t0.elapsed());
    // // for m in _matches {
    // //     println!("{} {:.3} {:?}", m.1, m.2, m.3);
    // // }
    // let ocr_string = ocr
    //     .iter()
    //     .map(|v| v.join(" "))
    //     .collect::<Vec<String>>()
    //     .join("\n");
    // println!("{}", ocr_string);

    // // recognize tray tiles
    // let now = Instant::now();
    // let cells = Layout::get_cells(&layout.trayrows, &layout.traycols);
    // let index: Vec<usize> = (0..cells.len()).into_iter().collect();
    // let resize_to = Some((67, 67, FilterType::Lanczos3));
    // let (ocr, _matches) =
    //     board.recognize_tiles(&layout, &index, &cells, &board.templates, (1, 7), resize_to);
    // let dt = now.elapsed();
    // println!("recognize tray tiles took {:?} {:?}", dt, t0.elapsed());

    // let ocr_string = ocr
    //     .iter()
    //     .map(|v| v.join(""))
    //     .collect::<Vec<String>>()
    //     .join("\n");
    // println!("{}", ocr_string);

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{:?}", err);
    }
}
