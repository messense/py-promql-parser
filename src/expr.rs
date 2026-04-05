use std::time::SystemTime;

use chrono::Duration;
use promql_parser::label::Label;
use promql_parser::parser::{
    self, AggregateExpr, AtModifier, BinaryExpr, Call, Expr, LabelModifier, MatrixSelector,
    NumberLiteral, Offset, ParenExpr, StringLiteral, SubqueryExpr, UnaryExpr,
    VectorMatchCardinality, VectorSelector, token::TokenType, value::ValueType,
};
use pyo3::exceptions::{PyNotImplementedError, PyOverflowError, PyValueError};
use pyo3::{IntoPyObjectExt, prelude::*};

#[pyclass(subclass, name = "Expr", module = "promql_parser", skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct PyExpr {
    pub expr: Expr,
}

impl PyExpr {
    pub fn create(py: Python, expr: Expr) -> PyResult<Py<PyAny>> {
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
    pub fn parse(py: Python, input: &str) -> PyResult<Py<PyAny>> {
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
    expr: Py<PyAny>,
    #[pyo3(get)]
    param: Option<Py<PyAny>>,
    #[pyo3(get)]
    modifier: Option<PyAggModifier>,
}

impl PyAggregateExpr {
    fn create(py: Python, expr: AggregateExpr) -> PyResult<Py<PyAny>> {
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
        Py::new(py, initializer)?.into_py_any(py)
    }
}

#[pymethods]
impl PyAggregateExpr {
    fn __str__(&self, py: Python) -> PyResult<String> {
        let op_str = self.op.__str__();
        let expr_str: String = self.expr.call_method0(py, "__str__")?.extract(py)?;
        let modifier_str = match &self.modifier {
            Some(m) => format!(" {}", m.__str__()),
            None => String::new(),
        };
        let param_str = match &self.param {
            Some(p) => {
                let p_str: String = p.call_method0(py, "__str__")?.extract(py)?;
                format!("{}, ", p_str)
            }
            None => String::new(),
        };
        Ok(format!(
            "{}{}({}{})",
            op_str, modifier_str, param_str, expr_str
        ))
    }
}

#[pyclass(name = "TokenType", module = "promql_parser", skip_from_py_object)]
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

#[pyclass(name = "AggModifier", module = "promql_parser", from_py_object)]
#[derive(Debug, Clone)]
pub struct PyAggModifier {
    #[pyo3(get, set)]
    r#type: PyAggModifierType,
    #[pyo3(get, set)]
    labels: Vec<Label>,
}

#[pymethods]
impl PyAggModifier {
    #[new]
    fn new(r#type: PyAggModifierType, labels: Vec<String>) -> Self {
        PyAggModifier { r#type, labels }
    }

    fn __str__(&self) -> String {
        let keyword = match self.r#type {
            PyAggModifierType::By => "by",
            PyAggModifierType::Without => "without",
        };
        format!("{} ({})", keyword, self.labels.join(", "))
    }
}

#[pyclass(
    name = "AggModifierType",
    module = "promql_parser",
    eq,
    eq_int,
    from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PyAggModifierType {
    By,
    Without,
}

#[pyclass(extends = PyExpr, name = "UnaryExpr", module = "promql_parser")]
pub struct PyUnaryExpr {
    #[pyo3(get)]
    expr: Py<PyAny>,
}

impl PyUnaryExpr {
    fn create(py: Python, expr: UnaryExpr) -> PyResult<Py<PyAny>> {
        let parent = PyExpr {
            expr: Expr::Unary(expr.clone()),
        };
        let UnaryExpr { expr } = expr;
        let initializer = PyClassInitializer::from(parent).add_subclass(PyUnaryExpr {
            expr: PyExpr::create(py, *expr)?,
        });
        Py::new(py, initializer)?.into_py_any(py)
    }
}

#[pymethods]
impl PyUnaryExpr {
    fn __str__(&self, py: Python) -> PyResult<String> {
        let expr_str: String = self.expr.call_method0(py, "__str__")?.extract(py)?;
        Ok(format!("-{}", expr_str))
    }
}

#[pyclass(extends = PyExpr, name = "BinaryExpr", module = "promql_parser")]
pub struct PyBinaryExpr {
    #[pyo3(get)]
    op: PyTokenType,
    #[pyo3(get)]
    lhs: Py<PyAny>,
    #[pyo3(get)]
    rhs: Py<PyAny>,
    #[pyo3(get)]
    modifier: Option<PyBinModifier>,
}

impl PyBinaryExpr {
    fn create(py: Python, expr: BinaryExpr) -> PyResult<Py<PyAny>> {
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
        Py::new(py, initializer)?.into_py_any(py)
    }
}

#[pymethods]
impl PyBinaryExpr {
    fn __str__(&self, py: Python) -> PyResult<String> {
        let lhs_str: String = self.lhs.call_method0(py, "__str__")?.extract(py)?;
        let rhs_str: String = self.rhs.call_method0(py, "__str__")?.extract(py)?;
        let op_str = self.op.__str__();
        let modifier_str = match &self.modifier {
            Some(m) => format!(" {}", m.__str__()),
            None => String::new(),
        };
        Ok(format!(
            "{} {}{} {}",
            lhs_str, op_str, modifier_str, rhs_str
        ))
    }
}

#[pyclass(name = "BinModifier", module = "promql_parser", from_py_object)]
#[derive(Debug, Clone)]
pub struct PyBinModifier {
    #[pyo3(get, set)]
    card: PyVectorMatchCardinality,
    #[pyo3(get, set)]
    matching: Option<PyLabelModifier>,
    #[pyo3(get, set)]
    return_bool: bool,
}

#[pymethods]
impl PyBinModifier {
    #[new]
    #[pyo3(signature = (card, return_bool, matching=None))]
    fn new(
        card: PyVectorMatchCardinality,
        return_bool: bool,
        matching: Option<PyLabelModifier>,
    ) -> Self {
        PyBinModifier {
            card,
            matching,
            return_bool,
        }
    }

    fn __str__(&self) -> String {
        let mut parts = Vec::new();

        if self.return_bool {
            parts.push("bool".to_string());
        }

        if let Some(matching) = &self.matching {
            parts.push(matching.__str__());
        }

        match self.card {
            PyVectorMatchCardinality::ManyToOne => parts.push("group_left".to_string()),
            PyVectorMatchCardinality::OneToMany => parts.push("group_right".to_string()),
            _ => {}
        }

        parts.join(" ")
    }
}

#[pyclass(name = "LabelModifier", module = "promql_parser", from_py_object)]
#[derive(Debug, Clone)]
pub struct PyLabelModifier {
    #[pyo3(get, set)]
    r#type: PyLabelModifierType,
    #[pyo3(get, set)]
    labels: Vec<Label>,
}

#[pymethods]
impl PyLabelModifier {
    #[new]
    fn new(r#type: PyLabelModifierType, labels: Vec<String>) -> Self {
        PyLabelModifier { r#type, labels }
    }

    fn __str__(&self) -> String {
        let keyword = match self.r#type {
            PyLabelModifierType::Include => "on",
            PyLabelModifierType::Exclude => "ignoring",
        };
        format!("{} ({})", keyword, self.labels.join(", "))
    }
}

#[pyclass(
    name = "LabelModifierType",
    module = "promql_parser",
    eq,
    eq_int,
    from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PyLabelModifierType {
    Include,
    Exclude,
}

#[pyclass(
    name = "VectorMatchCardinality",
    module = "promql_parser",
    eq,
    eq_int,
    from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    expr: Py<PyAny>,
}

impl PyParenExpr {
    fn create(py: Python, expr: ParenExpr) -> PyResult<Py<PyAny>> {
        let parent = PyExpr {
            expr: Expr::Paren(expr.clone()),
        };
        let ParenExpr { expr } = expr;
        let initializer = PyClassInitializer::from(parent).add_subclass(PyParenExpr {
            expr: PyExpr::create(py, *expr)?,
        });
        Py::new(py, initializer)?.into_py_any(py)
    }
}

#[pymethods]
impl PyParenExpr {
    fn __str__(&self, py: Python) -> PyResult<String> {
        let expr_str: String = self.expr.call_method0(py, "__str__")?.extract(py)?;
        Ok(format!("({})", expr_str))
    }
}

#[pyclass(extends = PyExpr, name = "SubqueryExpr", module = "promql_parser")]
pub struct PySubqueryExpr {
    #[pyo3(get, set)]
    expr: Py<PyAny>,
    #[pyo3(get, set)]
    offset: Option<Duration>,
    #[pyo3(get, set)]
    at: Option<PyAtModifier>,
    #[pyo3(get, set)]
    range: Duration,
    #[pyo3(get, set)]
    step: Option<Duration>,
}

impl PySubqueryExpr {
    fn create(py: Python, expr: SubqueryExpr) -> PyResult<Py<PyAny>> {
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
        Py::new(py, initializer)?.into_py_any(py)
    }

    fn format_duration(d: &Duration) -> String {
        promql_parser::util::duration::display_duration(&std::time::Duration::from_secs(
            d.num_seconds().unsigned_abs(),
        ))
    }
}

#[pymethods]
impl PySubqueryExpr {
    fn __str__(&self, py: Python) -> PyResult<String> {
        let expr_str: String = self.expr.call_method0(py, "__str__")?.extract(py)?;
        let range_str = Self::format_duration(&self.range);
        let step_str = match &self.step {
            Some(s) => format!(":{}", Self::format_duration(s)),
            None => String::new(),
        };
        let offset_str = match &self.offset {
            Some(d) if d.num_seconds() < 0 => {
                format!(" offset -{}", Self::format_duration(d))
            }
            Some(d) => {
                format!(" offset {}", Self::format_duration(d))
            }
            None => String::new(),
        };
        let at_str = match &self.at {
            Some(at) => format!(" {}", at.__str__()),
            None => String::new(),
        };
        Ok(format!(
            "{}[{}{}]{}{}",
            expr_str, range_str, step_str, at_str, offset_str
        ))
    }
}

#[pyclass(name = "AtModifier", module = "promql_parser", from_py_object)]
#[derive(Debug, Clone)]
pub struct PyAtModifier {
    #[pyo3(get, set)]
    r#type: PyAtModifierType,
    #[pyo3(get, set)]
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

#[pymethods]
impl PyAtModifier {
    fn __str__(&self) -> String {
        match self.r#type {
            PyAtModifierType::Start => "@ start()".to_string(),
            PyAtModifierType::End => "@ end()".to_string(),
            PyAtModifierType::At => {
                if let Some(at) = &self.at {
                    let timestamp = at
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_secs_f64())
                        .unwrap_or(0.0);
                    format!("@ {:.3}", timestamp)
                } else {
                    "@ 0".to_string()
                }
            }
        }
    }
}

#[pyclass(
    name = "AtModifierType",
    module = "promql_parser",
    eq,
    eq_int,
    from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PyAtModifierType {
    Start,
    End,
    At,
}

#[pyclass(extends = PyExpr, name = "NumberLiteral", module = "promql_parser")]
pub struct PyNumberLiteral {
    #[pyo3(get, set)]
    val: f64,
}

impl PyNumberLiteral {
    fn create(py: Python, expr: NumberLiteral) -> PyResult<Py<PyAny>> {
        let parent = PyExpr {
            expr: Expr::NumberLiteral(expr.clone()),
        };
        let NumberLiteral { val } = expr;
        let initializer = PyClassInitializer::from(parent).add_subclass(PyNumberLiteral { val });
        Py::new(py, initializer)?.into_py_any(py)
    }
}

#[pymethods]
impl PyNumberLiteral {
    fn __str__(&self) -> String {
        self.val.to_string()
    }
}

#[pyclass(extends = PyExpr, name = "StringLiteral", module = "promql_parser")]
pub struct PyStringLiteral {
    #[pyo3(get, set)]
    val: String,
}

impl PyStringLiteral {
    fn create(py: Python, expr: StringLiteral) -> PyResult<Py<PyAny>> {
        let parent = PyExpr {
            expr: Expr::StringLiteral(expr.clone()),
        };
        let StringLiteral { val } = expr;
        let initializer = PyClassInitializer::from(parent).add_subclass(PyStringLiteral { val });
        Py::new(py, initializer)?.into_py_any(py)
    }
}

#[pymethods]
impl PyStringLiteral {
    fn __str__(&self) -> String {
        format!("\"{}\"", self.val)
    }
}

#[pyclass(name = "MatchOp", module = "promql_parser", eq, eq_int, from_py_object)]
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

#[pyclass(name = "Matcher", module = "promql_parser", from_py_object)]
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PyMatcher {
    #[pyo3(get, set)]
    op: PyMatchOp,
    #[pyo3(get, set)]
    name: String,
    #[pyo3(get, set)]
    value: String,
}

#[pymethods]
impl PyMatcher {
    #[new]
    fn new(op: PyMatchOp, name: String, value: String) -> Self {
        PyMatcher { op, name, value }
    }

    fn __repr__(&self) -> String {
        format!(
            "Matcher({}, \"{}\", {})",
            self.op.__repr__(),
            self.name,
            self.value
        )
    }

    fn __str__(&self) -> String {
        let op_str = match self.op {
            PyMatchOp::Equal => "=",
            PyMatchOp::NotEqual => "!=",
            PyMatchOp::Re => "=~",
            PyMatchOp::NotRe => "!~",
        };
        format!("{}{}\"{}\"", self.name, op_str, self.value)
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

#[pyclass(name = "Matchers", module = "promql_parser", from_py_object)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PyMatchers {
    #[pyo3(get, set)]
    matchers: Vec<PyMatcher>,
    #[pyo3(get, set)]
    or_matchers: Vec<Vec<PyMatcher>>,
}

#[pymethods]
impl PyMatchers {
    #[new]
    fn new(matchers: Vec<PyMatcher>) -> Self {
        PyMatchers {
            matchers,
            or_matchers: Vec::new(),
        }
    }

    fn with_or_matchers(&self, or_matchers: Vec<Vec<PyMatcher>>) -> Self {
        PyMatchers {
            matchers: self.matchers.clone(),
            or_matchers,
        }
    }

    fn __str__(&self) -> String {
        let matchers_str: Vec<String> = self.matchers.iter().map(|m| m.__str__()).collect();
        if self.or_matchers.is_empty() {
            format!("{{{}}}", matchers_str.join(","))
        } else {
            let mut parts = vec![matchers_str.join(",")];
            for or_group in &self.or_matchers {
                let or_str: Vec<String> = or_group.iter().map(|m| m.__str__()).collect();
                parts.push(or_str.join(","));
            }
            format!("{{{}}}", parts.join(" or "))
        }
    }
}

#[pyclass(extends = PyExpr, name = "VectorSelector", module = "promql_parser")]
pub struct PyVectorSelector {
    #[pyo3(get, set)]
    name: Option<String>,
    #[pyo3(get, set)]
    matchers: PyMatchers,
    #[pyo3(get, set)]
    offset: Option<Duration>,
    #[pyo3(get, set)]
    at: Option<PyAtModifier>,
}

impl PyVectorSelector {
    fn create(py: Python, expr: VectorSelector) -> PyResult<Py<PyAny>> {
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
        Py::new(py, initializer)?.into_py_any(py)
    }

    fn format_offset(offset: &Option<Duration>) -> String {
        match offset {
            Some(d) if d.num_seconds() < 0 => {
                format!(
                    " offset -{}",
                    promql_parser::util::duration::display_duration(
                        &std::time::Duration::from_secs((-d.num_seconds()) as u64)
                    )
                )
            }
            Some(d) => {
                format!(
                    " offset {}",
                    promql_parser::util::duration::display_duration(
                        &std::time::Duration::from_secs(d.num_seconds() as u64)
                    )
                )
            }
            None => String::new(),
        }
    }
}

#[pymethods]
impl PyVectorSelector {
    fn __str__(&self) -> String {
        let name = self.name.as_deref().unwrap_or("");
        let matchers_str = self.matchers.__str__();
        let offset_str = Self::format_offset(&self.offset);
        format!("{}{}{}", name, matchers_str, offset_str)
    }
}

#[pyclass(extends = PyExpr, name = "MatrixSelector", module = "promql_parser")]
pub struct PyMatrixSelector {
    #[pyo3(get, set)]
    vector_selector: Py<PyAny>,
    #[pyo3(get, set)]
    range: Duration,
}

impl PyMatrixSelector {
    fn create(py: Python, expr: MatrixSelector) -> PyResult<Py<PyAny>> {
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
        Py::new(py, initializer)?.into_py_any(py)
    }
}

#[pymethods]
impl PyMatrixSelector {
    fn __str__(&self, py: Python) -> PyResult<String> {
        // Get the vector selector's string representation
        let vs_str: String = self
            .vector_selector
            .call_method0(py, "__str__")?
            .extract(py)?;
        // Remove any offset from the vector selector string (it goes after the range in matrix selectors)
        let vs_base = vs_str.split(" offset").next().unwrap_or(&vs_str);
        let range_str = promql_parser::util::duration::display_duration(
            &std::time::Duration::from_secs(self.range.num_seconds() as u64),
        );
        Ok(format!("{}[{}]", vs_base, range_str))
    }
}

#[pyclass(extends = PyExpr, name = "Call", module = "promql_parser")]
pub struct PyCall {
    #[pyo3(get)]
    func: PyFunction,
    #[pyo3(get)]
    args: Vec<Py<PyAny>>,
}

impl PyCall {
    fn create(py: Python, expr: Call) -> PyResult<Py<PyAny>> {
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
        Py::new(py, initializer)?.into_py_any(py)
    }
}

#[pymethods]
impl PyCall {
    fn __str__(&self, py: Python) -> PyResult<String> {
        let args_str: Result<Vec<String>, _> = self
            .args
            .iter()
            .map(|arg| arg.call_method0(py, "__str__")?.extract(py))
            .collect();
        Ok(format!("{}({})", self.func.name, args_str?.join(", ")))
    }
}

#[pyclass(
    name = "ValueType",
    module = "promql_parser",
    eq,
    eq_int,
    skip_from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[pyclass(name = "Function", module = "promql_parser", skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct PyFunction {
    #[pyo3(get)]
    name: &'static str,
    #[pyo3(get)]
    arg_types: Vec<PyValueType>,
    #[pyo3(get)]
    variadic: i32,
    #[pyo3(get)]
    return_type: PyValueType,
}
