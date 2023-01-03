use promql_parser::parser::{Expr, TokenType, ValueType, VectorMatchCardinality};
use pyo3::prelude::*;

#[pyclass(subclass, name = "Expr", module = "promql_parser")]
pub struct PyExpr;

impl PyExpr {
    pub fn create(py: Python, expr: Expr) -> PyResult<PyObject> {
        match expr {
            Expr::AggregateExpr {
                op,
                expr,
                param,
                grouping,
                without,
            } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PyAggregateExpr {
                    op,
                    expr: Self::create(py, *expr)?,
                    param: Self::create(py, *param)?,
                    grouping,
                    without,
                });
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::UnaryExpr { op, expr } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PyUnaryExpr {
                    op,
                    expr: Self::create(py, *expr)?,
                });
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::BinaryExpr {
                op,
                lhs,
                rhs,
                matching,
                return_bool,
            } => {
                let matching = matching.map(|m| PyVectorMatching {
                    card: m.card.into(),
                    matching_labels: m.matching_labels,
                    on: m.on,
                    include: m.include,
                });
                let initializer = PyClassInitializer::from(Self).add_subclass(PyBinaryExpr {
                    op,
                    lhs: Self::create(py, *lhs)?,
                    rhs: Self::create(py, *rhs)?,
                    matching,
                    return_bool,
                });
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::ParenExpr { expr } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PyParenExpr {
                    expr: Self::create(py, *expr)?,
                });
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::SubqueryExpr {
                expr,
                timestamp,
                start_or_end,
                ..
            } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PySubqueryExpr {
                    expr: Self::create(py, *expr)?,
                    timestamp,
                    start_or_end,
                });
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::NumberLiteral { val, span } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PyNumberLiteral {
                    val,
                    span: PySpan {
                        start: span.start(),
                        end: span.end(),
                    },
                });
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::StringLiteral { val, span } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PyStringLiteral {
                    val,
                    span: PySpan {
                        start: span.start(),
                        end: span.end(),
                    },
                });
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::VectorSelector {
                ref name,
                ref start_or_end,
                ref label_matchers,
                ..
            } => {
                let name = name.clone();
                let start_or_end = *start_or_end;
                let label_matchers = &label_matchers.matchers;
                let mut py_matchers = Vec::with_capacity(label_matchers.len());
                for matcher in label_matchers {
                    py_matchers.push(PyMatcher {
                        name: matcher.name.clone(),
                        value: matcher.value.clone(),
                        op: match matcher.op {
                            promql_parser::label::MatchOp::Equal => PyMatchOp::Equal,
                            promql_parser::label::MatchOp::NotEqual => PyMatchOp::NotEqual,
                            promql_parser::label::MatchOp::Re(_) => PyMatchOp::Re,
                            promql_parser::label::MatchOp::NotRe(_) => PyMatchOp::NotRe,
                        },
                    });
                }

                let initializer = PyClassInitializer::from(Self).add_subclass(PyVectorSelector {
                    name,
                    start_or_end,
                    label_matchers: py_matchers,
                });
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::MatrixSelector {
                vector_selector, ..
            } => {
                let initializer = PyClassInitializer::from(Self).add_subclass(PyMatrixSelector {
                    vector_selector: Self::create(py, *vector_selector)?,
                });
                Ok(Py::new(py, initializer)?.into_py(py))
            }
            Expr::Call { func, args, .. } => {
                let func = PyFunction {
                    name: func.name,
                    arg_types: func.arg_types.into_iter().map(|t| t.into()).collect(),
                    variadic: func.variadic,
                    return_type: func.return_type.into(),
                };
                let args: Result<Vec<_>, _> =
                    args.into_iter().map(|arg| Self::create(py, *arg)).collect();
                let initializer =
                    PyClassInitializer::from(Self).add_subclass(PyCall { func, args: args? });
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
    #[pyo3(get)]
    grouping: Vec<String>,
    #[pyo3(get)]
    without: bool,
}

#[pyclass(extends = PyExpr, name = "UnaryExpr", module = "promql_parser")]
pub struct PyUnaryExpr {
    #[pyo3(get)]
    op: TokenType,
    #[pyo3(get)]
    expr: PyObject,
}

#[pyclass(extends = PyExpr, name = "BinaryExpr", module = "promql_parser")]
pub struct PyBinaryExpr {
    #[pyo3(get)]
    op: TokenType,
    #[pyo3(get)]
    lhs: PyObject,
    #[pyo3(get)]
    rhs: PyObject,
    #[pyo3(get)]
    matching: Option<PyVectorMatching>,
    #[pyo3(get)]
    return_bool: bool,
}

#[pyclass(name = "VectorMatchCardinality", module = "promql_parser")]
#[derive(Debug, Clone, Copy)]
pub enum PyVectorMatchCardinality {
    OneToOne,
    ManyToOne,
    OneToMany,
    ManyToMany,
}

impl From<VectorMatchCardinality> for PyVectorMatchCardinality {
    fn from(value: VectorMatchCardinality) -> Self {
        match value {
            VectorMatchCardinality::OneToOne => PyVectorMatchCardinality::OneToOne,
            VectorMatchCardinality::ManyToOne => PyVectorMatchCardinality::ManyToOne,
            VectorMatchCardinality::OneToMany => PyVectorMatchCardinality::OneToMany,
            VectorMatchCardinality::ManyToMany => PyVectorMatchCardinality::ManyToMany,
        }
    }
}

#[pyclass(name = "VectorMatching", module = "promql_parser")]
#[derive(Debug, Clone)]
pub struct PyVectorMatching {
    #[pyo3(get)]
    card: PyVectorMatchCardinality,
    #[pyo3(get)]
    matching_labels: Vec<String>,
    #[pyo3(get)]
    on: bool,
    #[pyo3(get)]
    include: Vec<String>,
}

#[pyclass(extends = PyExpr, name = "ParenExpr", module = "promql_parser")]
pub struct PyParenExpr {
    #[pyo3(get)]
    expr: PyObject,
}

#[pyclass(extends = PyExpr, name = "SubqueryExpr", module = "promql_parser")]
pub struct PySubqueryExpr {
    #[pyo3(get)]
    expr: PyObject,
    // #[pyo3(get)]
    // range: Duration,
    // #[pyo3(get)]
    // offset: Duration,
    #[pyo3(get)]
    timestamp: Option<i64>,
    #[pyo3(get)]
    start_or_end: TokenType,
    // #[pyo3(get)]
    // step: Duration,
}

#[pyclass(extends = PyExpr, name = "NumberLiteral", module = "promql_parser")]
pub struct PyNumberLiteral {
    #[pyo3(get)]
    val: f64,
    #[pyo3(get)]
    span: PySpan,
}

#[pymethods]
impl PyNumberLiteral {
    fn __repr__(&self) -> String {
        format!("NumberLiteral({}, {})", self.val, self.span.__repr__())
    }
}

#[pyclass(name = "Span", module = "promql_parser")]
#[derive(Clone, Copy, Debug)]
pub struct PySpan {
    #[pyo3(get)]
    start: usize,
    #[pyo3(get)]
    end: usize,
}

#[pymethods]
impl PySpan {
    fn __repr__(&self) -> String {
        format!("Span({}, {})", self.start, self.end)
    }
}

#[pyclass(extends = PyExpr, name = "StringLiteral", module = "promql_parser")]
pub struct PyStringLiteral {
    #[pyo3(get)]
    val: String,
    #[pyo3(get)]
    span: PySpan,
}

#[pymethods]
impl PyStringLiteral {
    fn __repr__(&self) -> String {
        format!("StringLiteral(\"{}\", {})", self.val, self.span.__repr__())
    }
}

#[pyclass(name = "MatchOp", module = "promql_parser")]
#[derive(Debug, Clone, Copy)]
pub enum PyMatchOp {
    Equal,
    NotEqual,
    Re,
    NotRe,
}

#[pymethods]
impl PyMatchOp {
    fn __repr__(&self) -> &'static str {
        match self {
            PyMatchOp::Equal => "MatchOp.Equal",
            PyMatchOp::NotEqual => "MatchOp.NotEqual",
            PyMatchOp::Re => "MatchOp.Re",
            PyMatchOp::NotRe => "MatchOp.NotRe",
        }
    }
}

#[pyclass(name = "Matcher", module = "promql_parser")]
#[derive(Debug, Clone)]
pub struct PyMatcher {
    #[pyo3(get)]
    op: PyMatchOp,
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    value: String,
}

#[pymethods]
impl PyMatcher {
    fn __repr__(&self) -> String {
        format!(
            "Matcher({}, \"{}\", {})",
            self.op.__repr__(),
            self.name,
            self.value
        )
    }
}

#[pyclass(extends = PyExpr, name = "VectorSelector", module = "promql_parser")]
pub struct PyVectorSelector {
    #[pyo3(get)]
    name: Option<String>,
    #[pyo3(get)]
    start_or_end: Option<TokenType>,
    #[pyo3(get)]
    label_matchers: Vec<PyMatcher>,
}

#[pymethods]
impl PyVectorSelector {
    fn __repr__(&self) -> String {
        let matchers = self
            .label_matchers
            .iter()
            .map(|m| m.__repr__())
            .collect::<Vec<String>>();
        format!(
            "VectorSelector(\"{}\", {:?}, [{}])",
            self.name.as_ref().unwrap_or(&"".to_string()),
            self.start_or_end,
            matchers.join(", ")
        )
    }
}

#[pyclass(extends = PyExpr, name = "MatrixSelector", module = "promql_parser")]
pub struct PyMatrixSelector {
    #[pyo3(get)]
    vector_selector: PyObject,
    // #[pyo3(get)]
    // range: Duration,
}

#[pyclass(extends = PyExpr, name = "Call", module = "promql_parser")]
pub struct PyCall {
    #[pyo3(get)]
    func: PyFunction,
    #[pyo3(get)]
    args: Vec<PyObject>,
}

#[pyclass(name = "ValueType", module = "promql_parser")]
#[derive(Debug, Clone, Copy)]
pub enum PyValueType {
    Vector,
    Scalar,
    Matrix,
    String,
}

impl From<ValueType> for PyValueType {
    fn from(value: ValueType) -> Self {
        match value {
            ValueType::Vector => PyValueType::Vector,
            ValueType::Scalar => PyValueType::Scalar,
            ValueType::Matrix => PyValueType::Matrix,
            ValueType::String => PyValueType::String,
        }
    }
}

#[pyclass(name = "Function", module = "promql_parser")]
#[derive(Debug, Clone)]
pub struct PyFunction {
    #[pyo3(get)]
    name: &'static str,
    #[pyo3(get)]
    arg_types: Vec<PyValueType>,
    #[pyo3(get)]
    variadic: bool,
    #[pyo3(get)]
    return_type: PyValueType,
}
