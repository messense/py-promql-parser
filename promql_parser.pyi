"""PromQL Lexer and Parser

The goal of this project is to build a PromQL lexer and parser capable of
parsing PromQL that conforms with [Prometheus Query](https://prometheus.io/docs/prometheus/latest/querying/basics/).

## Example

The parser entry point is `promql_parser.parse`, which takes a string slice of PromQL
and returns the parse result as an AST (`promql_parser.Expr`).

```python
import promql_parser

promql = 'http_requests_total{environment=~"staging|testing|development",method!="GET"} offset 5m'
expr = promql_parser.parse(promql)
print(f"Prettify:\n{expr.prettify()}")
print(f"AST:\n{expr}")
```

This outputs:

```text
Prettify:
http_requests_total{environment=~"staging|testing|development",method!="GET"} offset 5m

AST:
VectorSelector(VectorSelector { name: Some("http_requests_total"), matchers: Matchers { matchers: [Matcher { op: Re(staging|testing|development), name: "environment", value: "staging|testing|development" }, Matcher { op: NotEqual, name: "method", value: "GET" }] }, offset: Some(Pos(300s)), at: None })
```

## PromQL compliance

This library declares compatible with [prometheus v2.45.0](https://github.com/prometheus/prometheus/tree/v2.45.0),
which isreleased at 2023-06-23. Any revision on PromQL after this commit is not guaranteed.
"""

from datetime import datetime, timedelta
from enum import Enum
from typing import Any, List, Optional, final

def parse(input: str) -> Expr:
    """Parse the given query literal to an AST."""
    ...

def parse_duration(duration: str) -> timedelta:
    """Parse a string into a duration.

    Assumes that a year always has 365d, a week always has 7d,
    and a day always has 24h.

    Basic usage:

    ```python
    from datetime import timedelta
    import promql_parser

    assert promql_parser.parse_duration("1h") == timedelta(seconds=3600);
    assert promql_parser.parse_duration("4d") == timedelta(seconds=3600 * 24 * 4)
    assert promql_parser.parse_duration("4d1h") == timedelta(seconds=3600 * 97)
    ```
    """
    ...

def display_duration(delta: timedelta) -> str:
    """Display Duration in Prometheus format"""
    ...

class Expr:
    @staticmethod
    def parse(input: str) -> Any: ...
    def prettify(self) -> str: ...

@final
class AggregateExpr(Expr):
    """An aggregation operation on a Vector.

    Attributes:
      op: The used aggregation operation.
      expr: The Vector expression over which is aggregated.
      param: Parameter used by some aggregators.
      modifier: An optional modifier for some operations like sum.
    """

    op: TokenType
    expr: Expr
    param: Optional[Any]
    modifier: Optional[AggModifier]

@final
class TokenType:
    pass

@final
class AggModifier:
    """Aggregation modifier (by/without clause)."""
    type: AggModifierType
    labels: List[str]

    def __init__(self, type: AggModifierType, labels: List[str]) -> None:
        """Create a new AggModifier.

        Args:
            type: The modifier type (By or Without).
            labels: List of label names.
        """
        ...

@final
class AggModifierType(Enum):
    By: Any
    Without: Any

@final
class UnaryExpr(Expr):
    """An unary operation on another expression."""

    expr: Expr

@final
class BinaryExpr(Expr):
    """A binary expression between two child expressions.

    Attributes:
      op: The operation of the expression.
      lhs: The operands on the left side of the operator.
      rhs: The operands on the right side of the operator.
      modifier: An optional modifier.
    """

    op: TokenType
    lhs: Expr
    rhs: Expr
    modifier: Optional[BinModifier]

@final
class BinModifier:
    """Binary expression modifier

    Attributes:
      card:
        The matching behavior for the operation if both operands are Vectors.
        If they are not this field is None.
      matching: on/ignoring on labels. Like a + b, no match modified is needed.
      return_bool: If a comparison operator, return 0/1 rather than filtering.
    """

    card: VectorMatchCardinality
    matching: Optional[LabelModifier]
    return_bool: bool

    def __init__(self, card: VectorMatchCardinality, return_bool: bool, matching: Optional[LabelModifier] = None) -> None:
        """Create a new BinModifier.

        Args:
            card: The vector matching cardinality.
            return_bool: Whether to return bool values.
            matching: Optional label modifier for matching.
        """
        ...

@final
class LabelModifier:
    """LabelModifier acts as

    # Aggregation Modifier

    - Exclude means `ignoring`
    - Include means `on`

    # Vector Match Modifier

    - Exclude means `without` removes the listed labels from the result vector,
    while all other labels are preserved in the output.
    - Include means `by` does the opposite and drops labels that are not listed in the by clause,
    even if their label values are identical between all elements of the vector.

    If empty listed labels, meaning no grouping
    """

    type: LabelModifierType
    labels: List[str]

    def __init__(self, type: LabelModifierType, labels: List[str]) -> None:
        """Create a new LabelModifier.

        Args:
            type: The modifier type (Include or Exclude).
            labels: List of label names.
        """
        ...

@final
class LabelModifierType(Enum):
    Include: Any
    Exclude: Any

@final
class VectorMatchCardinality(Enum):
    """The label list provided with the group_left or group_right modifier contains
    additional labels from the "one"-side to be included in the result metrics."""

    OneToOne: Any
    ManyToOne: Any
    OneToMany: Any
    ManyToMany: Any

@final
class ParenExpr(Expr):
    """Wraps an expression so it cannot be disassembled as a consequence of operator precedence."""

    expr: Expr

@final
class SubqueryExpr(Expr):
    """A subquery."""

    expr: Expr
    offset: Optional[timedelta]
    at: Optional[AtModifier]
    range: Optional[timedelta]
    step: Optional[timedelta]

@final
class AtModifier:
    type: AtModifierType
    at: Optional[datetime]

@final
class AtModifierType(Enum):
    Start: Any
    End: Any
    At: Any

@final
class NumberLiteral(Expr):
    """A number literal."""

    val: float

    def __str__(self) -> str:
        """Return the PromQL string representation of this number."""
        ...

@final
class StringLiteral(Expr):
    """A string literal."""

    val: str

    def __str__(self) -> str:
        """Return the PromQL string representation of this string."""
        ...

@final
class MatchOp(Enum):
    """Match operator for label matchers."""
    Equal: Any
    NotEqual: Any
    Re: Any
    NotRe: Any

@final
class Matcher:
    """A label matcher.

    Example:
        Create a new matcher:
        ```python
        import promql_parser

        m = promql_parser.Matcher(promql_parser.MatchOp.Equal, "job", "prometheus")
        m.value = "alertmanager"  # Modify the value
        print(str(m))  # Output: job="alertmanager"
        ```
    """
    op: MatchOp
    name: str
    value: str

    def __init__(self, op: MatchOp, name: str, value: str) -> None:
        """Create a new Matcher.

        Args:
            op: The match operator (Equal, NotEqual, Re, NotRe).
            name: The label name.
            value: The label value.
        """
        ...

    def __str__(self) -> str:
        """Return the PromQL string representation of this matcher."""
        ...

@final
class Matchers:
    """A collection of label matchers.

    Example:
        Create matchers from scratch:
        ```python
        import promql_parser

        m1 = promql_parser.Matcher(promql_parser.MatchOp.Equal, "job", "prometheus")
        m2 = promql_parser.Matcher(promql_parser.MatchOp.Re, "instance", ".*:9090")
        matchers = promql_parser.Matchers([m1, m2])
        print(str(matchers))  # Output: {job="prometheus",instance=~".*:9090"}
        ```
    """
    matchers: List[Matcher]
    or_matchers: List[List[Matcher]]

    def __init__(self, matchers: List[Matcher]) -> None:
        """Create a new Matchers collection.

        Args:
            matchers: List of Matcher objects.
        """
        ...

    def with_or_matchers(self, or_matchers: List[List[Matcher]]) -> "Matchers":
        """Return a new Matchers with the specified or_matchers.

        Args:
            or_matchers: List of alternative matcher groups.

        Returns:
            A new Matchers instance with the same matchers and the specified or_matchers.
        """
        ...

    def __str__(self) -> str:
        """Return the PromQL string representation of these matchers."""
        ...

@final
class VectorSelector(Expr):
    """A Vector selection.

    Use str() to convert to PromQL after modifications.
    """

    name: Optional[str]
    matchers: Matchers
    offset: Optional[timedelta]
    at: Optional[AtModifier]

    def __str__(self) -> str:
        """Return the PromQL string representation of this vector selector."""
        ...

@final
class MatrixSelector(Expr):
    """A Matrix selection.

    Use str() to convert to PromQL after modifications.
    """

    vector_selector: VectorSelector
    range: timedelta

    def __str__(self) -> str:
        """Return the PromQL string representation of this matrix selector."""
        ...

@final
class Call(Expr):
    """A call to a Prometheus function."""

    func: Function
    args: List[Any]

@final
class ValueType(Enum):
    Vector: Any
    Scalar: Any
    Matrix: Any
    String: Any

@final
class Function:
    name: str
    arg_types: List[ValueType]
    variadic: int
    return_type: ValueType
