use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDelta, PyDeltaAccess};

mod expr;

use self::expr::PyExpr;
use ::promql_parser::parser::{
    AggregateExpr, BinaryExpr, Call, Expr, MatrixSelector, ParenExpr, SubqueryExpr, UnaryExpr,
};

/// Parse the input PromQL and return the AST.
#[pyfunction]
fn parse(py: Python, input: &str) -> PyResult<Py<PyAny>> {
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

fn extract_expr(expr: &Bound<'_, PyAny>) -> PyResult<Expr> {
    Ok(expr.extract::<PyRef<'_, PyExpr>>()?.expr.clone())
}

fn callback_continue(callback: Option<&Bound<'_, PyAny>>, expr: Py<PyAny>) -> PyResult<bool> {
    match callback {
        Some(callback) => {
            let result = callback.call1((expr,))?;
            if result.is_none() {
                Ok(true)
            } else {
                result.extract()
            }
        }
        None => Ok(true),
    }
}

fn walk_expr_py(
    py: Python,
    expr: &Expr,
    pre_visit: Option<&Bound<'_, PyAny>>,
    post_visit: Option<&Bound<'_, PyAny>>,
) -> PyResult<bool> {
    if !callback_continue(pre_visit, PyExpr::create(py, expr.clone())?)? {
        return Ok(false);
    }

    let recurse = match expr {
        Expr::Aggregate(AggregateExpr { expr, param, .. }) => {
            if let Some(param) = param {
                if !walk_expr_py(py, param, pre_visit, post_visit)? {
                    return Ok(false);
                }
            }
            walk_expr_py(py, expr, pre_visit, post_visit)?
        }
        Expr::Unary(UnaryExpr { expr }) => walk_expr_py(py, expr, pre_visit, post_visit)?,
        Expr::Binary(BinaryExpr { lhs, rhs, .. }) => {
            walk_expr_py(py, lhs, pre_visit, post_visit)?
                && walk_expr_py(py, rhs, pre_visit, post_visit)?
        }
        Expr::Paren(ParenExpr { expr }) => walk_expr_py(py, expr, pre_visit, post_visit)?,
        Expr::Subquery(SubqueryExpr { expr, .. }) => walk_expr_py(py, expr, pre_visit, post_visit)?,
        Expr::Call(Call { args, .. }) => {
            for arg in &args.args {
                if !walk_expr_py(py, arg, pre_visit, post_visit)? {
                    return Ok(false);
                }
            }
            true
        }
        Expr::MatrixSelector(MatrixSelector { vs, .. }) => {
            walk_expr_py(py, &Expr::VectorSelector(vs.clone()), pre_visit, post_visit)?
        }
        Expr::NumberLiteral(_) | Expr::StringLiteral(_) | Expr::VectorSelector(_) => true,
        Expr::Extension(_) => {
            return Err(PyValueError::new_err(
                "extension expressions are not supported",
            ));
        }
    };

    if !recurse {
        return Ok(false);
    }

    callback_continue(post_visit, PyExpr::create(py, expr.clone())?)
}

/// Walk an expression AST in depth-first order.
///
/// `pre_visit` is called before children are visited, and `post_visit` after children are
/// visited. Returning `False` from either callback stops traversal and makes this function
/// return `False`; returning `True` or `None` continues traversal.
#[pyfunction]
#[pyo3(signature = (expr, pre_visit=None, post_visit=None))]
fn walk(
    py: Python,
    expr: Bound<'_, PyAny>,
    pre_visit: Option<Bound<'_, PyAny>>,
    post_visit: Option<Bound<'_, PyAny>>,
) -> PyResult<bool> {
    let expr = extract_expr(&expr)?;
    walk_expr_py(py, &expr, pre_visit.as_ref(), post_visit.as_ref())
}

fn callback_replacement(
    callback: Option<&Bound<'_, PyAny>>,
    expr: Py<PyAny>,
) -> PyResult<Option<Expr>> {
    match callback {
        Some(callback) => {
            let result = callback.call1((expr,))?;
            if result.is_none() {
                Ok(None)
            } else {
                Ok(Some(extract_expr(&result)?))
            }
        }
        None => Ok(None),
    }
}

fn transform_expr_py(
    py: Python,
    mut expr: Expr,
    pre_visit: Option<&Bound<'_, PyAny>>,
    post_visit: Option<&Bound<'_, PyAny>>,
) -> PyResult<Expr> {
    if let Some(replacement) = callback_replacement(pre_visit, PyExpr::create(py, expr.clone())?)? {
        expr = replacement;
    }

    match &mut expr {
        Expr::Aggregate(AggregateExpr { expr, param, .. }) => {
            if let Some(param) = param {
                **param = transform_expr_py(py, (**param).clone(), pre_visit, post_visit)?;
            }
            **expr = transform_expr_py(py, (**expr).clone(), pre_visit, post_visit)?;
        }
        Expr::Unary(UnaryExpr { expr }) => {
            **expr = transform_expr_py(py, (**expr).clone(), pre_visit, post_visit)?;
        }
        Expr::Binary(BinaryExpr { lhs, rhs, .. }) => {
            **lhs = transform_expr_py(py, (**lhs).clone(), pre_visit, post_visit)?;
            **rhs = transform_expr_py(py, (**rhs).clone(), pre_visit, post_visit)?;
        }
        Expr::Paren(ParenExpr { expr }) => {
            **expr = transform_expr_py(py, (**expr).clone(), pre_visit, post_visit)?;
        }
        Expr::Subquery(SubqueryExpr { expr, .. }) => {
            **expr = transform_expr_py(py, (**expr).clone(), pre_visit, post_visit)?;
        }
        Expr::Call(Call { args, .. }) => {
            for arg in &mut args.args {
                **arg = transform_expr_py(py, (**arg).clone(), pre_visit, post_visit)?;
            }
        }
        Expr::MatrixSelector(MatrixSelector { vs, .. }) => {
            let transformed =
                transform_expr_py(py, Expr::VectorSelector(vs.clone()), pre_visit, post_visit)?;
            match transformed {
                Expr::VectorSelector(transformed_vs) => *vs = transformed_vs,
                _ => {
                    return Err(PyValueError::new_err(
                        "matrix selector vector_selector must transform to a VectorSelector",
                    ));
                }
            }
        }
        Expr::NumberLiteral(_) | Expr::StringLiteral(_) | Expr::VectorSelector(_) => {}
        Expr::Extension(_) => {
            return Err(PyValueError::new_err(
                "extension expressions are not supported",
            ));
        }
    }

    if let Some(replacement) = callback_replacement(post_visit, PyExpr::create(py, expr.clone())?)?
    {
        expr = replacement;
    }

    Ok(expr)
}

/// Transform an expression AST in depth-first order and return a new expression.
///
/// `pre_visit` and `post_visit` callbacks may return a replacement `Expr`, or `None` to keep
/// the current node. The input expression is not modified in place.
#[pyfunction]
#[pyo3(signature = (expr, pre_visit=None, post_visit=None))]
fn transform(
    py: Python,
    expr: Bound<'_, PyAny>,
    pre_visit: Option<Bound<'_, PyAny>>,
    post_visit: Option<Bound<'_, PyAny>>,
) -> PyResult<Py<PyAny>> {
    let expr = extract_expr(&expr)?;
    let expr = transform_expr_py(py, expr, pre_visit.as_ref(), post_visit.as_ref())?;
    PyExpr::create(py, expr)
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
    m.add_class::<expr::PyVectorMatchFillValues>()?;
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
    m.add_function(wrap_pyfunction!(walk, m)?)?;
    m.add_function(wrap_pyfunction!(transform, m)?)?;
    m.add_function(wrap_pyfunction!(parse_duration, m)?)?;
    m.add_function(wrap_pyfunction!(display_duration, m)?)?;
    Ok(())
}
