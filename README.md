# py-promql-parser

![CI](https://github.com/messense/py-promql-parser/workflows/CI/badge.svg)
[![PyPI](https://img.shields.io/pypi/v/promql-parser.svg)](https://pypi.org/project/promql-parser)

[promql-parser](https://github.com/GreptimeTeam/promql-parser) Python binding, a PromQL parser for Python.

## Installation

```bash
pip install promql-parser
```

## Usage

```python
import promql_parser

ast = promql_parser.parse('prometheus_http_requests_total{code="200", job="prometheus"}')
print(ast)
```

## License

This work is released under the MIT license. A copy of the license is provided in the [LICENSE](./LICENSE) file.
