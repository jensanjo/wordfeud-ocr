# The `pywordfeud_ocr` Python package

The `pywordfeud_ocr` package is a python wrapper for the Rust [wordfeud-ocr](https://github.com/jensanjo/wordfeud-ocr) crate.

## Installation

```
pip install pywordfeud_ocr
```

## Usage

Here is an example:

```python
import pywordfeud_ocr
board = pywordfeud_ocr.Board()
board.recognize_screenshot_from_file("screenshot.png")
```
