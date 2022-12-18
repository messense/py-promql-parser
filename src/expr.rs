use promql_parser::parser::{Expr, TokenType};
use pyo3::prelude::*;

#[pyclass(subclass, name = "Expr", module = "promql_parser")]
pub struct PyExpr;

impl PyExpr {
    pub fn create(py: Python, expr: Expr) -> PyResult<PyObject> {
        match expr {
            Expr::AggregateExpr {
                op, expr, param, ..
            } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PyAggregateExpr {
                    op,
                    expr: Self::create(py, *expr)?,
                    param: Self::create(py, *param)?,
                });
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::UnaryExpr { .. } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PyUnaryExpr);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::BinaryExpr { .. } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PyBinaryExpr);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::ParenExpr { .. } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PyParenExpr);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::SubqueryExpr { .. } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PySubqueryExpr);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::NumberLiteral { .. } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PyNumberLiteral);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::StringLiteral { .. } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PyStringLiteral);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::VectorSelector {
                ref name,
                ref start_or_end,
                ..
            } => {
                let name = name.clone();
                let start_or_end = *start_or_end;
                let initializer = PyClassInitializer::from(Self)
                    .add_subclass(PyVectorSelector { name, start_or_end });
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::MatrixSelector { .. } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PyMatrixSelector);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::Call { .. } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PyCall);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
        }
    }
}

#[pyclass(extends = PyExpr, name = "AggregateExpr", module = "promql_parser")]
pub struct PyAggregateExpr {
    #[pyo3(get)]
    op: TokenType,
    #[pyo3(get)]
    expr: PyObject,
    #[pyo3(get)]
    param: PyObject,
}

#[pyclass(extends = PyExpr, name = "UnaryExpr", module = "promql_parser")]
pub struct PyUnaryExpr;

#[pyclass(extends = PyExpr, name = "BinaryExpr", module = "promql_parser")]
pub struct PyBinaryExpr;

#[pyclass(extends = PyExpr, name = "ParenExpr", module = "promql_parser")]
pub struct PyParenExpr;

#[pyclass(extends = PyExpr, name = "SubqueryExpr", module = "promql_parser")]
pub struct PySubqueryExpr;

#[pyclass(extends = PyExpr, name = "NumberLiteral", module = "promql_parser")]
pub struct PyNumberLiteral;

#[pyclass(extends = PyExpr, name = "StringLiteral", module = "promql_parser")]
pub struct PyStringLiteral;

#[pyclass(extends = PyExpr, name = "VectorSelector", module = "promql_parser")]
pub struct PyVectorSelector {
    #[pyo3(get)]
    name: Option<String>,
    #[pyo3(get)]
    start_or_end: Option<TokenType>,
}

#[pyclass(extends = PyExpr, name = "MatrixSelector", module = "promql_parser")]
pub struct PyMatrixSelector;

#[pyclass(extends = PyExpr, name = "Call", module = "promql_parser")]
pub struct PyCall;
