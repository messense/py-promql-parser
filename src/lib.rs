use ::promql_parser::parser;
use pyo3::prelude::*;

mod expr;

use self::expr::PyExpr;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn parse(py: Python, input: &str) -> PyResult<PyObject> {
    let expr = parser::parse(input).unwrap();
    let py_expr = PyExpr::create(py, expr)?;
    Ok(py_expr)
}

/// A Python module implemented in Rust.
#[pymodule]
fn promql_parser(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyExpr>()?;
    m.add_class::<expr::PyAggregateExpr>()?;
    m.add_class::<expr::PyUnaryExpr>()?;
    m.add_class::<expr::PyBinaryExpr>()?;
    m.add_class::<expr::PyParenExpr>()?;
    m.add_class::<expr::PySubqueryExpr>()?;
    m.add_class::<expr::PyNumberLiteral>()?;
    m.add_class::<expr::PyStringLiteral>()?;
    m.add_class::<expr::PyVectorSelector>()?;
    m.add_class::<expr::PyMatrixSelector>()?;
    m.add_class::<expr::PyCall>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}