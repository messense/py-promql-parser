use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDelta, PyDeltaAccess};

mod expr;

use self::expr::PyExpr;

/// Parse the input PromQL and return the AST.
#[pyfunction]
fn parse(py: Python, input: &str) -> PyResult<PyObject> {
    PyExpr::parse(py, input)
}

#[pyfunction]
fn parse_duration<'p>(py: Python<'p>, duration: &str) -> PyResult<Bound<'p, PyDelta>> {
    let duration =
        ::promql_parser::util::duration::parse_duration(duration).map_err(PyValueError::new_err)?;
    PyDelta::new(
        py,
        0,
        duration.as_secs().try_into().unwrap(),
        duration.subsec_millis().try_into().unwrap(),
        false,
    )
}

#[pyfunction]
fn display_duration(delta: Bound<'_, PyDelta>) -> String {
    let duration = std::time::Duration::new(
        delta.get_days() as u64 * 24 * 60 * 60 + delta.get_seconds() as u64,
        delta.get_microseconds() as u32 * 1000,
    );
    ::promql_parser::util::duration::display_duration(&duration)
}

/// A Python module implemented in Rust.
#[pymodule(gil_used = false)]
fn promql_parser(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyExpr>()?;
    m.add_class::<expr::PyAggregateExpr>()?;
    m.add_class::<expr::PyTokenType>()?;
    m.add_class::<expr::PyAggModifier>()?;
    m.add_class::<expr::PyAggModifierType>()?;
    m.add_class::<expr::PyUnaryExpr>()?;
    m.add_class::<expr::PyBinaryExpr>()?;
    m.add_class::<expr::PyBinModifier>()?;
    m.add_class::<expr::PyLabelModifier>()?;
    m.add_class::<expr::PyLabelModifierType>()?;
    m.add_class::<expr::PyVectorMatchCardinality>()?;
    m.add_class::<expr::PyParenExpr>()?;
    m.add_class::<expr::PySubqueryExpr>()?;
    m.add_class::<expr::PyAtModifier>()?;
    m.add_class::<expr::PyAtModifierType>()?;
    m.add_class::<expr::PyNumberLiteral>()?;
    m.add_class::<expr::PyStringLiteral>()?;
    m.add_class::<expr::PyMatchOp>()?;
    m.add_class::<expr::PyMatcher>()?;
    m.add_class::<expr::PyMatchers>()?;
    m.add_class::<expr::PyVectorSelector>()?;
    m.add_class::<expr::PyMatrixSelector>()?;
    m.add_class::<expr::PyCall>()?;
    m.add_class::<expr::PyValueType>()?;
    m.add_class::<expr::PyFunction>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_duration, m)?)?;
    m.add_function(wrap_pyfunction!(display_duration, m)?)?;
    Ok(())
}
