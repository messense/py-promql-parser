use promql_parser::parser::Expr;
use pyo3::prelude::*;

#[pyclass(subclass, name = "Expr", module = "promql_parser")]
pub struct PyExpr {
    expr: Expr,
}

impl PyExpr {
    pub fn new(expr: Expr) -> Self {
        Self { expr }
    }

    pub fn into_subclass(self, py: Python) -> PyResult<PyObject> {
        match self.expr {
            Expr::AggregateExpr { .. } => {
                let initializer = PyClassInitializer::from(self).add_subclass(PyAggregateExpr);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::UnaryExpr { .. } => {
                let initializer = PyClassInitializer::from(self).add_subclass(PyUnaryExpr);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::BinaryExpr { .. } => {
                let initializer = PyClassInitializer::from(self).add_subclass(PyBinaryExpr);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::ParenExpr { .. } => {
                let initializer = PyClassInitializer::from(self).add_subclass(PyParenExpr);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::SubqueryExpr { .. } => {
                let initializer = PyClassInitializer::from(self).add_subclass(PySubqueryExpr);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::NumberLiteral { .. } => {
                let initializer = PyClassInitializer::from(self).add_subclass(PyNumberLiteral);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::StringLiteral { .. } => {
                let initializer = PyClassInitializer::from(self).add_subclass(PyStringLiteral);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::VectorSelector { .. } => {
                let initializer = PyClassInitializer::from(self).add_subclass(PyVectorSelector);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::MatrixSelector { .. } => {
                let initializer = PyClassInitializer::from(self).add_subclass(PyMatrixSelector);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::Call { .. } => {
                let initializer = PyClassInitializer::from(self).add_subclass(PyCall);
                Ok(Py::new(py, initializer)?.into_py(py))
            }
        }
    }
}

#[pymethods]
impl PyExpr {
    fn __repr__(&self) -> String {
        format!("{:?}", self.expr)
    }
}

#[pyclass(extends = PyExpr, name = "AggregateExpr", module = "promql_parser")]
pub struct PyAggregateExpr;

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
pub struct PyVectorSelector;

#[pyclass(extends = PyExpr, name = "MatrixSelector", module = "promql_parser")]
pub struct PyMatrixSelector;

#[pyclass(extends = PyExpr, name = "Call", module = "promql_parser")]
pub struct PyCall;
