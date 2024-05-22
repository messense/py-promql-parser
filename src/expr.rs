use std::time::SystemTime;

use chrono::Duration;
use promql_parser::label::Label;
use promql_parser::parser::{
    self, token::TokenType, value::ValueType, AggregateExpr, AtModifier, BinaryExpr, Call, Expr,
    LabelModifier, MatrixSelector, NumberLiteral, Offset, ParenExpr, StringLiteral, SubqueryExpr,
    UnaryExpr, VectorMatchCardinality, VectorSelector,
};
use pyo3::exceptions::{PyNotImplementedError, PyOverflowError, PyValueError};
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
            Expr::Extension(_ext) => Err(PyNotImplementedError::new_err("extension unimplemented")),
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

    fn prettify(&self) -> String {
        self.expr.prettify()
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
                Some(LabelModifier::Include(labels)) => Some(PyAggModifier {
                    r#type: PyAggModifierType::By,
                    labels: labels.labels,
                }),
                Some(LabelModifier::Exclude(labels)) => Some(PyAggModifier {
                    r#type: PyAggModifierType::Without,
                    labels: labels.labels,
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

#[pymethods]
impl PyTokenType {
    fn __str__(&self) -> String {
        format!("{}", self.r#type)
    }
}

#[pyclass(name = "AggModifier", module = "promql_parser")]
#[derive(Debug, Clone)]
pub struct PyAggModifier {
    #[pyo3(get)]
    r#type: PyAggModifierType,
    #[pyo3(get)]
    labels: Vec<Label>,
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
                    Some(LabelModifier::Include(labels)) => Some(PyLabelModifier {
                        r#type: PyLabelModifierType::Include,
                        labels: labels.labels,
                    }),
                    Some(LabelModifier::Exclude(labels)) => Some(PyLabelModifier {
                        r#type: PyLabelModifierType::Exclude,
                        labels: labels.labels,
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
    matching: Option<PyLabelModifier>,
    #[pyo3(get)]
    return_bool: bool,
}

#[pyclass(name = "LabelModifier", module = "promql_parser")]
#[derive(Debug, Clone)]
pub struct PyLabelModifier {
    #[pyo3(get)]
    r#type: PyLabelModifierType,
    #[pyo3(get)]
    labels: Vec<Label>,
}

#[pyclass(name = "LabelModifierType", module = "promql_parser")]
#[derive(Debug, Clone, Copy)]
pub enum PyLabelModifierType {
    Include,
    Exclude,
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
            at: at.map(|at| at.into()),
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
    #[pyo3(get)]
    at: Option<SystemTime>,
}

impl From<AtModifier> for PyAtModifier {
    fn from(at: AtModifier) -> Self {
        let (typ, at) = match at {
            AtModifier::Start => (PyAtModifierType::Start, None),
            AtModifier::End => (PyAtModifierType::End, None),
            AtModifier::At(at) => (PyAtModifierType::At, Some(at)),
        };
        PyAtModifier { r#type: typ, at }
    }
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

impl From<promql_parser::label::Matcher> for PyMatcher {
    fn from(matcher: promql_parser::label::Matcher) -> Self {
        PyMatcher {
            name: matcher.name,
            value: matcher.value,
            op: match matcher.op {
                promql_parser::label::MatchOp::Equal => PyMatchOp::Equal,
                promql_parser::label::MatchOp::NotEqual => PyMatchOp::NotEqual,
                promql_parser::label::MatchOp::Re(_) => PyMatchOp::Re,
                promql_parser::label::MatchOp::NotRe(_) => PyMatchOp::NotRe,
            },
        }
    }
}

#[pyclass(name = "Matchers", module = "promql_parser")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PyMatchers {
    #[pyo3(get)]
    matchers: Vec<PyMatcher>,
    #[pyo3(get)]
    or_matchers: Vec<Vec<PyMatcher>>,
}

#[pyclass(extends = PyExpr, name = "VectorSelector", module = "promql_parser")]
pub struct PyVectorSelector {
    #[pyo3(get)]
    name: Option<String>,
    #[pyo3(get)]
    matchers: PyMatchers,
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
        let or_matchers = &matchers.or_matchers;
        let matchers = &matchers.matchers;
        let mut py_matchers = PyMatchers {
            matchers: Vec::with_capacity(matchers.len()),
            or_matchers: Vec::with_capacity(or_matchers.len()),
        };
        for matcher in matchers {
            py_matchers.matchers.push(matcher.clone().into());
        }
        for matchers in or_matchers {
            py_matchers.or_matchers.push(
                matchers
                    .iter()
                    .map(|matcher| matcher.clone().into())
                    .collect(),
            );
        }

        let initializer = PyClassInitializer::from(parent).add_subclass(PyVectorSelector {
            name,
            matchers: py_matchers,
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
            at: at.map(|at| at.into()),
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
        let MatrixSelector { vs, range } = expr;
        let vector_selector = PyVectorSelector::create(py, vs)?;
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
