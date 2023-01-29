import promql_parser


l = 'prometheus_http_requests_total{code="200", job="prometheus"}'
print(promql_parser.parse(l))

print(promql_parser.parse('min_over_time(rate(foo{bar="baz"}[2s])[5m:] @ 1603775091)[4m:3s]'))

print(promql_parser.parse('1'))

print(promql_parser.parse('1 + 1'))

print(promql_parser.parse('1 + 2/(3*1)'))

print(promql_parser.parse('+some_metric'))
