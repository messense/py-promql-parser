use ::promql_parser::parser;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

mod expr;

use self::expr::PyExpr;

/// Parse the input PromQL and return the AST.
#[pyfunction]
fn parse(py: Python, input: &str) -> PyResult<PyObject> {
    PyExpr::parse(py, input)
}

#[pyfunction]
fn check_ast(py: Python, ast: PyExpr) -> PyResult<PyObject> {
    let expr = parser::check_ast(ast.expr).map_err(PyValueError::new_err)?;
    let py_expr = PyExpr::create(py, expr)?;
    Ok(py_expr)
}

/// A Python module implemented in Rust.
#[pymodule]
fn promql_parser(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyExpr>()?;
    m.add_class::<expr::PyAggregateExpr>()?;
    m.add_class::<expr::PyTokenType>()?;
    m.add_class::<expr::PyAggModifier>()?;
    m.add_class::<expr::PyAggModifierType>()?;
    m.add_class::<expr::PyUnaryExpr>()?;
    m.add_class::<expr::PyBinaryExpr>()?;
    m.add_class::<expr::PyBinModifier>()?;
    m.add_class::<expr::PyVectorMatchModifier>()?;
    m.add_class::<expr::PyVectorMatchModifierType>()?;
    m.add_class::<expr::PyVectorMatchCardinality>()?;
    m.add_class::<expr::PyParenExpr>()?;
    m.add_class::<expr::PySubqueryExpr>()?;
    m.add_class::<expr::PyAtModifier>()?;
    m.add_class::<expr::PyAtModifierType>()?;
    m.add_class::<expr::PyNumberLiteral>()?;
    m.add_class::<expr::PyStringLiteral>()?;
    m.add_class::<expr::PyMatchOp>()?;
    m.add_class::<expr::PyMatcher>()?;
    m.add_class::<expr::PyVectorSelector>()?;
    m.add_class::<expr::PyMatrixSelector>()?;
    m.add_class::<expr::PyCall>()?;
    m.add_class::<expr::PyValueType>()?;
    m.add_class::<expr::PyFunction>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(check_ast, m)?)?;
    Ok(())
}
