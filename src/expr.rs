use std::collections::HashSet;

use chrono::Duration;
use promql_parser::label::Labels;
use promql_parser::parser::{
    self, AggModifier, AggregateExpr, AtModifier, BinaryExpr, Call, Expr, MatrixSelector,
    NumberLiteral, Offset, ParenExpr, StringLiteral, SubqueryExpr, TokenType, UnaryExpr, ValueType,
    VectorMatchCardinality, VectorMatchModifier, VectorSelector,
};
use pyo3::exceptions::{PyOverflowError, PyValueError};
use pyo3::prelude::*;

#[pyclass(subclass, name = "Expr", module = "promql_parser")]
#[derive(Debug, Clone)]
pub struct PyExpr {
    pub expr: Expr,
}

impl PyExpr {
    pub fn create(py: Python, expr: Expr) -> PyResult<PyObject> {
        match expr {
            Expr::Aggregate(agg) => PyAggregateExpr::create(py, agg),
            Expr::Unary(expr) => PyUnaryExpr::create(py, expr),
            Expr::Binary(bin) => PyBinaryExpr::create(py, bin),
            Expr::Paren(expr) => PyParenExpr::create(py, expr),
            Expr::Subquery(subquery) => PySubqueryExpr::create(py, subquery),
            Expr::NumberLiteral(lit) => PyNumberLiteral::create(py, lit),
            Expr::StringLiteral(lit) => PyStringLiteral::create(py, lit),
            Expr::VectorSelector(selector) => PyVectorSelector::create(py, selector),
            Expr::MatrixSelector(selector) => PyMatrixSelector::create(py, selector),
            Expr::Call(call) => PyCall::create(py, call),
        }
    }
}

#[pymethods]
impl PyExpr {
    #[staticmethod]
    pub fn parse(py: Python, input: &str) -> PyResult<PyObject> {
        let expr = parser::parse(input).map_err(PyValueError::new_err)?;
        let py_expr = Self::create(py, expr)?;
        Ok(py_expr)
    }

    fn __repr__(&self) -> String {
        format!("{:#?}", self.expr)
    }
}

#[pyclass(extends = PyExpr, name = "AggregateExpr", module = "promql_parser")]
pub struct PyAggregateExpr {
    #[pyo3(get)]
    op: PyTokenType,
    #[pyo3(get)]
    expr: PyObject,
    #[pyo3(get)]
    param: Option<PyObject>,
    #[pyo3(get)]
    modifier: Option<PyAggModifier>,
}

impl PyAggregateExpr {
    fn create(py: Python, expr: AggregateExpr) -> PyResult<PyObject> {
        let parent = PyExpr {
            expr: Expr::Aggregate(expr.clone()),
        };
        let AggregateExpr {
            op,
            expr,
            param,
            modifier,
        } = expr;
        let initializer = PyClassInitializer::from(parent).add_subclass(PyAggregateExpr {
            op: op.into(),
            expr: PyExpr::create(py, *expr)?,
            param: match param {
                Some(param) => Some(PyExpr::create(py, *param)?),
                None => None,
            },
            modifier: match modifier {
                Some(AggModifier::By(labels)) => Some(PyAggModifier {
                    r#type: PyAggModifierType::By,
                    labels,
                }),
                Some(AggModifier::Without(labels)) => Some(PyAggModifier {
                    r#type: PyAggModifierType::Without,
                    labels,
                }),
                None => None,
            },
        });
        Ok(Py::new(py, initializer)?.into_py(py))
    }
}

#[pyclass(name = "TokenType", module = "promql_parser")]
#[derive(Debug, Clone, Copy)]
pub struct PyTokenType {
    r#type: TokenType,
}

impl From<TokenType> for PyTokenType {
    fn from(token_type: TokenType) -> Self {
        PyTokenType { r#type: token_type }
    }
}

#[pyclass(name = "AggModifier", module = "promql_parser")]
#[derive(Debug, Clone)]
pub struct PyAggModifier {
    #[pyo3(get)]
    r#type: PyAggModifierType,
    #[pyo3(get)]
    labels: Labels,
}

#[pyclass(name = "AggModifierType", module = "promql_parser")]
#[derive(Debug, Clone, Copy)]
pub enum PyAggModifierType {
    By,
    Without,
}

#[pyclass(extends = PyExpr, name = "UnaryExpr", module = "promql_parser")]
pub struct PyUnaryExpr {
    #[pyo3(get)]
    expr: PyObject,
}

impl PyUnaryExpr {
    fn create(py: Python, expr: UnaryExpr) -> PyResult<PyObject> {
        let parent = PyExpr {
            expr: Expr::Unary(expr.clone()),
        };
        let UnaryExpr { expr } = expr;
        let initializer = PyClassInitializer::from(parent).add_subclass(PyUnaryExpr {
            expr: PyExpr::create(py, *expr)?,
        });
        Ok(Py::new(py, initializer)?.into_py(py))
    }
}

#[pyclass(extends = PyExpr, name = "BinaryExpr", module = "promql_parser")]
pub struct PyBinaryExpr {
    #[pyo3(get)]
    op: PyTokenType,
    #[pyo3(get)]
    lhs: PyObject,
    #[pyo3(get)]
    rhs: PyObject,
    #[pyo3(get)]
    modifier: Option<PyBinModifier>,
}

impl PyBinaryExpr {
    fn create(py: Python, expr: BinaryExpr) -> PyResult<PyObject> {
        let parent = PyExpr {
            expr: Expr::Binary(expr.clone()),
        };
        let BinaryExpr {
            op,
            lhs,
            rhs,
            modifier,
        } = expr;
        let py_modifier = match modifier {
            Some(modifier) => Some(PyBinModifier {
                card: modifier.card.into(),
                matching: match modifier.matching {
                    Some(VectorMatchModifier::On(labels)) => Some(PyVectorMatchModifier {
                        r#type: PyVectorMatchModifierType::On,
                        labels,
                    }),
                    Some(VectorMatchModifier::Ignoring(labels)) => Some(PyVectorMatchModifier {
                        r#type: PyVectorMatchModifierType::Ignoring,
                        labels,
                    }),
                    None => None,
                },
                return_bool: modifier.return_bool,
            }),
            None => None,
        };
        let initializer = PyClassInitializer::from(parent).add_subclass(PyBinaryExpr {
            op: op.into(),
            lhs: PyExpr::create(py, *lhs)?,
            rhs: PyExpr::create(py, *rhs)?,
            modifier: py_modifier,
        });
        Ok(Py::new(py, initializer)?.into_py(py))
    }
}

#[pyclass(name = "BinModifier", module = "promql_parser")]
#[derive(Debug, Clone)]
pub struct PyBinModifier {
    #[pyo3(get)]
    card: PyVectorMatchCardinality,
    #[pyo3(get)]
    matching: Option<PyVectorMatchModifier>,
    #[pyo3(get)]
    return_bool: bool,
}

#[pyclass(name = "VectorMatchModifier", module = "promql_parser")]
#[derive(Debug, Clone)]
pub struct PyVectorMatchModifier {
    #[pyo3(get)]
    r#type: PyVectorMatchModifierType,
    #[pyo3(get)]
    labels: Labels,
}

#[pyclass(name = "VectorMatchModifierType", module = "promql_parser")]
#[derive(Debug, Clone, Copy)]
pub enum PyVectorMatchModifierType {
    On,
    Ignoring,
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
            VectorMatchCardinality::ManyToOne(_) => PyVectorMatchCardinality::ManyToOne,
            VectorMatchCardinality::OneToMany(_) => PyVectorMatchCardinality::OneToMany,
            VectorMatchCardinality::ManyToMany => PyVectorMatchCardinality::ManyToMany,
        }
    }
}

#[pyclass(extends = PyExpr, name = "ParenExpr", module = "promql_parser")]
pub struct PyParenExpr {
    #[pyo3(get)]
    expr: PyObject,
}

impl PyParenExpr {
    fn create(py: Python, expr: ParenExpr) -> PyResult<PyObject> {
        let parent = PyExpr {
            expr: Expr::Paren(expr.clone()),
        };
        let ParenExpr { expr } = expr;
        let initializer = PyClassInitializer::from(parent).add_subclass(PyParenExpr {
            expr: PyExpr::create(py, *expr)?,
        });
        Ok(Py::new(py, initializer)?.into_py(py))
    }
}

#[pyclass(extends = PyExpr, name = "SubqueryExpr", module = "promql_parser")]
pub struct PySubqueryExpr {
    #[pyo3(get)]
    expr: PyObject,
    #[pyo3(get)]
    offset: Option<Duration>,
    #[pyo3(get)]
    at: Option<PyAtModifier>,
    #[pyo3(get)]
    range: Duration,
    #[pyo3(get)]
    step: Option<Duration>,
}

impl PySubqueryExpr {
    fn create(py: Python, expr: SubqueryExpr) -> PyResult<PyObject> {
        let parent = PyExpr {
            expr: Expr::Subquery(expr.clone()),
        };
        let SubqueryExpr {
            expr,
            offset,
            at,
            range,
            step,
        } = expr;
        let initializer = PyClassInitializer::from(parent).add_subclass(PySubqueryExpr {
            expr: PyExpr::create(py, *expr)?,
            offset: match offset {
                Some(Offset::Pos(off)) => Some(
                    Duration::from_std(off).map_err(|e| PyOverflowError::new_err(e.to_string()))?,
                ),
                Some(Offset::Neg(off)) => Some(
                    -Duration::from_std(off)
                        .map_err(|e| PyOverflowError::new_err(e.to_string()))?,
                ),
                None => None,
            },
            at: match at {
                Some(at) => {
                    let typ = match at {
                        AtModifier::Start => PyAtModifierType::Start,
                        AtModifier::End => PyAtModifierType::End,
                        AtModifier::At(_) => PyAtModifierType::At,
                    };
                    Some(PyAtModifier { r#type: typ })
                }
                None => None,
            },
            range: Duration::from_std(range)
                .map_err(|e| PyOverflowError::new_err(e.to_string()))?,
            step: match step {
                Some(step) => Some(
                    Duration::from_std(step)
                        .map_err(|e| PyOverflowError::new_err(e.to_string()))?,
                ),
                None => None,
            },
        });
        Ok(Py::new(py, initializer)?.into_py(py))
    }
}

#[pyclass(name = "AtModifier", module = "promql_parser")]
#[derive(Debug, Clone)]
pub struct PyAtModifier {
    #[pyo3(get)]
    r#type: PyAtModifierType,
    // at: Option<SystemTime>,
}

#[pyclass(name = "AtModifierType", module = "promql_parser")]
#[derive(Debug, Clone, Copy)]
pub enum PyAtModifierType {
    Start,
    End,
    At,
}

#[pyclass(extends = PyExpr, name = "NumberLiteral", module = "promql_parser")]
pub struct PyNumberLiteral {
    #[pyo3(get)]
    val: f64,
}

impl PyNumberLiteral {
    fn create(py: Python, expr: NumberLiteral) -> PyResult<PyObject> {
        let parent = PyExpr {
            expr: Expr::NumberLiteral(expr.clone()),
        };
        let NumberLiteral { val } = expr;
        let initializer = PyClassInitializer::from(parent).add_subclass(PyNumberLiteral { val });
        Ok(Py::new(py, initializer)?.into_py(py))
    }
}

#[pyclass(extends = PyExpr, name = "StringLiteral", module = "promql_parser")]
pub struct PyStringLiteral {
    #[pyo3(get)]
    val: String,
}

impl PyStringLiteral {
    fn create(py: Python, expr: StringLiteral) -> PyResult<PyObject> {
        let parent = PyExpr {
            expr: Expr::StringLiteral(expr.clone()),
        };
        let StringLiteral { val } = expr;
        let initializer = PyClassInitializer::from(parent).add_subclass(PyStringLiteral { val });
        Ok(Py::new(py, initializer)?.into_py(py))
    }
}

#[pyclass(name = "MatchOp", module = "promql_parser")]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
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
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
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
    label_matchers: HashSet<PyMatcher>,
    #[pyo3(get)]
    offset: Option<Duration>,
    #[pyo3(get)]
    at: Option<PyAtModifier>,
}

impl PyVectorSelector {
    fn create(py: Python, expr: VectorSelector) -> PyResult<PyObject> {
        let parent = PyExpr {
            expr: Expr::VectorSelector(expr.clone()),
        };
        let VectorSelector {
            name,
            matchers,
            offset,
            at,
        } = expr;
        let matchers = &matchers.matchers;
        let mut py_matchers = HashSet::with_capacity(matchers.len());
        for matcher in matchers {
            py_matchers.insert(PyMatcher {
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

        let initializer = PyClassInitializer::from(parent).add_subclass(PyVectorSelector {
            name,
            label_matchers: py_matchers,
            offset: match offset {
                Some(Offset::Pos(off)) => Some(
                    Duration::from_std(off).map_err(|e| PyOverflowError::new_err(e.to_string()))?,
                ),
                Some(Offset::Neg(off)) => Some(
                    -Duration::from_std(off)
                        .map_err(|e| PyOverflowError::new_err(e.to_string()))?,
                ),
                None => None,
            },
            at: match at {
                Some(at) => {
                    let typ = match at {
                        AtModifier::Start => PyAtModifierType::Start,
                        AtModifier::End => PyAtModifierType::End,
                        AtModifier::At(_) => PyAtModifierType::At,
                    };
                    Some(PyAtModifier { r#type: typ })
                }
                None => None,
            },
        });
        Ok(Py::new(py, initializer)?.into_py(py))
    }
}

#[pyclass(extends = PyExpr, name = "MatrixSelector", module = "promql_parser")]
pub struct PyMatrixSelector {
    #[pyo3(get)]
    vector_selector: PyObject,
    #[pyo3(get)]
    range: Duration,
}

impl PyMatrixSelector {
    fn create(py: Python, expr: MatrixSelector) -> PyResult<PyObject> {
        let parent = PyExpr {
            expr: Expr::MatrixSelector(expr.clone()),
        };
        let MatrixSelector {
            vector_selector,
            range,
        } = expr;
        let vector_selector = PyVectorSelector::create(py, vector_selector)?;
        let initializer = PyClassInitializer::from(parent).add_subclass(PyMatrixSelector {
            vector_selector,
            range: Duration::from_std(range)
                .map_err(|e| PyOverflowError::new_err(e.to_string()))?,
        });
        Ok(Py::new(py, initializer)?.into_py(py))
    }
}

#[pyclass(extends = PyExpr, name = "Call", module = "promql_parser")]
pub struct PyCall {
    #[pyo3(get)]
    func: PyFunction,
    #[pyo3(get)]
    args: Vec<PyObject>,
}

impl PyCall {
    fn create(py: Python, expr: Call) -> PyResult<PyObject> {
        let parent = PyExpr {
            expr: Expr::Call(expr.clone()),
        };
        let Call { func, args } = expr;
        let func = PyFunction {
            name: func.name,
            arg_types: func.arg_types.into_iter().map(|t| t.into()).collect(),
            variadic: func.variadic,
            return_type: func.return_type.into(),
        };
        let args: Result<Vec<_>, _> = args
            .args
            .into_iter()
            .map(|arg| PyExpr::create(py, *arg))
            .collect();
        let initializer =
            PyClassInitializer::from(parent).add_subclass(PyCall { func, args: args? });
        Ok(Py::new(py, initializer)?.into_py(py))
    }
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
