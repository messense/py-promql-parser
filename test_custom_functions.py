from __future__ import annotations

import unittest

import promql_parser


def function(
    name: str,
    arg_types: list[promql_parser.ValueType],
    return_type: promql_parser.ValueType = promql_parser.ValueType.Vector,
) -> promql_parser.Function:
    return promql_parser.Function(name, arg_types, return_type)


class TestCustomFunctions(unittest.TestCase):
    def setUp(self) -> None:
        promql_parser.clear_extra_functions()

    def tearDown(self) -> None:
        promql_parser.clear_extra_functions()

    def test_register_and_parse_one_function(self) -> None:
        promql_parser.register_extra_functions(
            [
                function(
                    "custom_over_time",
                    [promql_parser.ValueType.Matrix],
                )
            ]
        )

        expr = promql_parser.parse("custom_over_time(requests_total[5m])")

        self.assertIsInstance(expr, promql_parser.Call)
        assert isinstance(expr, promql_parser.Call)
        self.assertEqual(expr.func.name, "custom_over_time")
        self.assertEqual(expr.func.arg_types, [promql_parser.ValueType.Matrix])
        self.assertEqual(expr.func.return_type, promql_parser.ValueType.Vector)

    def test_register_multiple_functions(self) -> None:
        promql_parser.register_extra_functions(
            [
                function("custom_rate", [promql_parser.ValueType.Matrix]),
                function(
                    "custom_scale",
                    [
                        promql_parser.ValueType.Vector,
                        promql_parser.ValueType.Scalar,
                    ],
                ),
            ]
        )

        promql_parser.parse("custom_rate(requests_total[5m])")
        promql_parser.parse("custom_scale(requests_total, 2)")

    def test_argument_and_return_types_are_validated(self) -> None:
        promql_parser.register_extra_functions(
            [
                function(
                    "custom_matrix",
                    [promql_parser.ValueType.Matrix],
                ),
                function(
                    "custom_scalar",
                    [],
                    promql_parser.ValueType.Scalar,
                ),
            ]
        )

        with self.assertRaises(ValueError):
            promql_parser.parse("custom_matrix(requests_total)")

        promql_parser.parse("vector(custom_scalar())")
        with self.assertRaises(ValueError):
            promql_parser.parse("abs(custom_scalar())")

        with self.assertRaises(TypeError):
            promql_parser.Function(
                "invalid_arg_type",
                ["matrix"],  # type: ignore[list-item]
                promql_parser.ValueType.Vector,
            )
        with self.assertRaises(TypeError):
            promql_parser.Function(
                "invalid_return_type",
                [promql_parser.ValueType.Matrix],
                "vector",  # type: ignore[arg-type]
            )

    def test_builtin_function_conflicts_are_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "conflicts with built-in function"):
            promql_parser.register_extra_functions(
                [
                    function(
                        "rate",
                        [promql_parser.ValueType.Matrix],
                    )
                ]
            )

    def test_unrelated_unknown_functions_remain_rejected(self) -> None:
        promql_parser.register_extra_functions(
            [function("known_custom", [promql_parser.ValueType.Vector])]
        )

        with self.assertRaises(ValueError):
            promql_parser.parse("unrelated_unknown(requests_total)")

    def test_clear_registrations(self) -> None:
        promql_parser.register_extra_functions(
            [function("temporary_custom", [promql_parser.ValueType.Vector])]
        )
        promql_parser.parse("temporary_custom(requests_total)")

        promql_parser.clear_extra_functions()

        with self.assertRaises(ValueError):
            promql_parser.parse("temporary_custom(requests_total)")


if __name__ == "__main__":
    unittest.main()
