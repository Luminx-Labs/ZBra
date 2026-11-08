//! Python development bindings for the ZBra chess engine.

use pyo3::prelude::*;

/// Python module exposed by the development bindings.
#[pymodule]
fn zbra_py(_module: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
