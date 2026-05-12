# py-promql-parser

![CI](https://github.com/messense/py-promql-parser/workflows/CI/badge.svg)
[![PyPI](https://img.shields.io/pypi/v/promql-parser.svg)](https://pypi.org/project/promql-parser)

Python bindings for [promql-parser](https://github.com/GreptimeTeam/promql-parser), a PromQL parser written in Rust.

`promql-parser` parses Prometheus query expressions into a Python-accessible AST that can be inspected, walked, transformed, and formatted back to PromQL.

## Installation

```bash
pip install promql-parser
```

## Quick start

```python
import promql_parser

ast = promql_parser.parse('prometheus_http_requests_total{code="200", job="prometheus"}')
print(ast)
print(ast.prettify())
```

## Inspecting the AST

Parsed queries are returned as subclasses of `promql_parser.Expr` such as `VectorSelector`, `MatrixSelector`, `Call`, `AggregateExpr`, and `BinaryExpr`.

```python
import promql_parser

expr = promql_parser.parse('rate(http_requests_total{job="api"}[5m])')

assert isinstance(expr, promql_parser.Call)
assert expr.func.name == "rate"

matrix = expr.args[0]
assert isinstance(matrix, promql_parser.MatrixSelector)
assert matrix.range.total_seconds() == 300

vector = matrix.vector_selector
assert vector.name == "http_requests_total"
assert str(vector.matchers) == '{job="api"}'
```

## Walking expressions

Use `walk()` to visit every expression node in depth-first order. `pre_visit` is called before children are visited, and `post_visit` after children are visited. Return `False` from either callback to stop traversal early; return `True` or `None` to continue.

```python
import promql_parser

expr = promql_parser.parse('sum(rate(http_requests_total{job="api"}[5m]))')

names = []

def collect_vector_names(node):
    if isinstance(node, promql_parser.VectorSelector) and node.name is not None:
        names.append(node.name)

promql_parser.walk(expr, pre_visit=collect_vector_names)

assert names == ["http_requests_total"]
```

## Transforming expressions

Use `transform()` to return a new expression tree. A callback may return a replacement `Expr`, or `None` to keep the current node unchanged. The original expression is not modified in place.

```python
import promql_parser

expr = promql_parser.parse('rate(http_requests_total{job="api"}[5m]) + errors_total')

def rename_metric(node):
    if isinstance(node, promql_parser.VectorSelector) and node.name == "errors_total":
        return promql_parser.parse('http_errors_total')

new_expr = promql_parser.transform(expr, pre_visit=rename_metric)

assert str(expr) == 'rate(http_requests_total{job="api"}[5m]) + errors_total{}'
assert str(new_expr) == 'rate(http_requests_total{job="api"}[5m]) + http_errors_total{}'
```

## Duration helpers

```python
from datetime import timedelta
import promql_parser

assert promql_parser.parse_duration("1h30m") == timedelta(seconds=5400)
assert promql_parser.display_duration(timedelta(days=7, hours=2)) == "7d2h"
```

## License

This work is released under the MIT license. A copy of the license is provided in the [LICENSE](./LICENSE) file.
