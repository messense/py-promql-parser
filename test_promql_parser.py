from promql_parser import parse, check_ast

l = 'prometheus_http_requests_total{code="200", job="prometheus"}'
print(check_ast(parse(l)))

print(parse('min_over_time(rate(foo{bar="baz"}[2s])[5m:] @ 1603775091)[4m:3s]'))

print(parse('1'))

print(parse('1 + 1'))

print(parse('1 + 2/(3*1)'))

print(parse('+some_metric'))
