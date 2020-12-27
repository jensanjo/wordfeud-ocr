# Wordfeud OCR
![Crates.io](https://img.shields.io/crates/l/wordfeud-ocr)
![Crates.io](https://img.shields.io/crates/v/wordfeud-ocr)
[![Documentation](https://docs.rs/wordfeud-ocr/badge.svg)

A Rust library that recognizes a screenshot from the Wordfeud game on Android phone. 

Features:

* Locate the board and rack areas on the screen
* Segment the board and rack area to locate the grid "cells"
* Use template matching to identify the tiles on the board an the rack, as well as the bonus cells on the board

The image processing for the screenshot recognition is done with help from the [image](https://github.com/image-rs/image) and [imageproc](https://github.com/image-rs/imageproc) crates.
Currently it has been tested only on Android phones with screen resolutions of 1080x1920 and 1080x2160 pixels.

## Usage

Add this to your `Cargo.toml`:

```
wordfeud-ocr = "0.1"
```

## Example

```Rust
let path = "screenshots/screenshot_english.png";
let gray = image::open(path)?.into_luma8();
let board = Board::new();
let result = board.recognize_screenshot(&gray)?;
println!("Tiles:\n{}", result.tiles_ocr);
```
That would result in this output:

```
Tiles:
...............
...............
............z..
............if.
.........dental
..........v.ex.
.......h..e....
......hedonIc..
....r..d..l....
....o..o..y....
....brent......
....o..i..v....
.gaits.S..e....
....i..munged..
....c.....a....
```
# Locate board and rack areas

Here is an example screenshot, with the grid lines marked in red (start) and blue (end). **NOTE**: the images are shown here in reduced size.

![example screenshot](https://github.com/jensanjo/wordfeud-ocr/raw/master/images/screenshot-resized.png)

## Board area
Here is the resulting board:

![example screenshot](https://github.com/jensanjo/wordfeud-ocr/raw/master/images/board-resized.png)

## Rack area
And the tiles in the rack:

![example screenshot](https://github.com/jensanjo/wordfeud-ocr/raw/master/images/rack-resized.png)

# Template matching

## Find the tiles

After the cells are located in the board each cell is checked if it is a grid cell (possibly with a letter or word bonus) or if it contains a letter tile. The distinction is made by looking at the mean pixel value in the cell. 
The following collage (produced by the `collage.rs` example program) shows the result:

![collage](https://github.com/jensanjo/wordfeud-ocr/raw/master/images/collage.png)

To recognize the tiles, we match each tile with each of a set of letter templates, and find the best match.
The templates have a size of 38x60 (wxh) pixels.

![letter templates](https://github.com/jensanjo/wordfeud-ocr/raw/master/images/templates.png)

For the curious: The collage is produced by the `Imagemagick` [montage](https://legacy.imagemagick.org/Usage/montage/) tool:

```shell
lib$ montage src/templates/[A-Z]*png -geometry 38x60+4+4 -shadow templates.png
```


## Recognize grid cells

In a similar manner, the grid cells are recognized.
First we find the cells that have a bonus, by looking at the mean pixel value.
Then each bonus cell is matched a set of bonus templates:

![bonus templates](https://github.com/jensanjo/wordfeud-ocr/raw/master/images/bonus.png)



