# Example queries

* <https://prometheus.io/docs/practices/histograms/>

```c
// request rate over time
rate(http_server_requests_count{http_route="/"}[1m])
rate(nginx_http_requests_total[1m])

// SLO - set "le" to the bucket which represents the SLO target, bucket must exist
sum(rate(http_server_request_duration_seconds_bucket{http_route="/", le="0.05"}[1m])) by (instance) /
sum(rate(http_server_request_duration_seconds_count{http_route="/"}[1m])) by (instance)

// p99
histogram_quantile(0.99, sum(rate(http_server_request_duration_seconds_bucket{http_route="/"}[1m])) by (le, instance))
histogram_quantile(0.99, sum(rate(http_server_request_duration_ms_bucket{http_route="/"}[1m])) by (le, instance))

```
