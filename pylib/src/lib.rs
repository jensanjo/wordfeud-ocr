use pyo3::{
    create_exception,
    exceptions::PyException,
    prelude::*,
    types::{PyDict, PySlice},
    wrap_pyfunction, PyErr,
};
use wordfeud_ocr::OcrResults;

create_exception!(pywordfeud_ocr, WordfeudOcrException, PyException);

fn process_result(res: &OcrResults, py: Python) -> PyResult<PyObject> {
    // flatten the inner vec to string for convenience
    let state_ocr: Vec<String> = res.tiles_ocr.iter().map(|row| row.join("")).collect();
    let board_ocr: Vec<String> = res.grid_ocr.iter().map(|row| row.join(" ")).collect();
    let rack_ocr: String = res.rack_ocr[0].join("").replace(".", " ");
    let b = res.board_area;
    let board_area = (
        PySlice::new(py, b.y as isize, (b.y + b.height) as isize, 1),
        PySlice::new(py, b.x as isize, (b.x + b.width) as isize, 1),
    );
    let b = res.rack_area;
    let rack_area = (
        PySlice::new(py, b.y as isize, (b.y + b.height) as isize, 1),
        PySlice::new(py, b.x as isize, (b.x + b.width) as isize, 1),
    );
    let dict = PyDict::new(py);
    dict.set_item("state_ocr", state_ocr)?;
    dict.set_item("board_ocr", board_ocr)?;
    dict.set_item("tray_ocr", rack_ocr.clone())?; //TODO for compatibility
    dict.set_item("rack_ocr", rack_ocr)?;
    dict.set_item("board_area", board_area)?;
    dict.set_item("rack_area", rack_area)?;
    Ok(dict.into())
}

#[pyfunction]
fn recognize_screenshot_from_file(screenshot_filename: String, py: Python) -> PyResult<PyObject> {
    let board = wordfeud_ocr::Board::new();
    let res = board
        .recognize_screenshot_from_file(&screenshot_filename)
        .map_err(WordfeudOcrError::from)?;
    process_result(&res, py)
}

#[pyfunction]
fn recognize_screenshot_from_memory(screenshot: &[u8], py: Python) -> PyResult<PyObject> {
    let board = wordfeud_ocr::Board::new();
    let res = board
        .recognize_screenshot_from_memory(&screenshot)
        .map_err(WordfeudOcrError::from)?;
    process_result(&res, py)
}

/// Wrapper around wordfeud_ocr::Error so we convert to PyErr
struct WordfeudOcrError(wordfeud_ocr::Error);

impl From<wordfeud_ocr::Error> for WordfeudOcrError {
    fn from(err: wordfeud_ocr::Error) -> WordfeudOcrError {
        WordfeudOcrError { 0: err }
    }
}

impl From<WordfeudOcrError> for PyErr {
    fn from(err: WordfeudOcrError) -> PyErr {
        PyErr::new::<WordfeudOcrException, String>(err.0.to_string())
    }
}

#[pymodule]

fn pywordfeud_ocr(_py: Python, m: &PyModule) -> PyResult<()> {
    // m.add_class::<Board>()?;
    m.add_function(wrap_pyfunction!(recognize_screenshot_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(recognize_screenshot_from_memory, m)?)?;
    Ok(())
}
