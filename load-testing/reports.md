# constant load reports

* 100 requests per second over 5 minutes
* k6 run
* wrk2 run



```sh
make build
make restart && sleep 10 && load-testing/k6.wrk2.sh
make restart && sleep 10 && load-testing/wrk2.sh
```

## k6

```
constant ✓ [======================================] 100/100 VUs  5m0s  100 iters/s

     data_received..................: 9.5 MB 32 kB/s
     data_sent......................: 2.4 MB 8.0 kB/s
     http_req_blocked...............: p(99)=13.1µs  p(99.9)=172.1µs p(99.99)=250.19µs p(99.999)=331.31µs max=346.8µs  count=30001
     http_req_connecting............: p(99)=0s      p(99.9)=110.6µs p(99.99)=154.39µs p(99.999)=245.33µs max=273.9µs  count=30001
   ✓ http_req_duration..............: p(99)=61.28ms p(99.9)=61.44ms p(99.99)=111.19ms p(99.999)=111.26ms max=111.28ms count=30001
       { expected_response:true }...: p(99)=61.28ms p(99.9)=61.44ms p(99.99)=111.19ms p(99.999)=111.26ms max=111.28ms count=30001
     http_req_failed................: 0.00%  ✓ 0     ✗ 30001
     http_req_receiving.............: p(99)=96.1µs  p(99.9)=127.6µs p(99.99)=213.79µs p(99.999)=525µs    max=589.3µs  count=30001
     http_req_sending...............: p(99)=50.2µs  p(99.9)=81.2µs  p(99.99)=330.39µs p(99.999)=416.73µs max=417.7µs  count=30001
     http_req_tls_handshaking.......: p(99)=0s      p(99.9)=0s      p(99.99)=0s       p(99.999)=0s       max=0s       count=30001
     http_req_waiting...............: p(99)=61.19ms p(99.9)=61.35ms p(99.99)=111.11ms p(99.999)=111.17ms max=111.18ms count=30001
     http_reqs......................: 30001  100.002135/s
     iteration_duration.............: p(99)=61.39ms p(99.9)=61.56ms p(99.99)=111.28ms p(99.999)=111.35ms max=111.37ms count=30001
     iterations.....................: 30001  100.002135/s
     vus............................: 100    min=100 max=100
     vus_max........................: 100    min=100 max=100
```

## wrk2

Truncated view

```
  Latency Distribution (HdrHistogram - Recorded Latency)
 99.000%   50.88ms
 99.900%   61.31ms
 99.990%  109.82ms
 99.999%  111.10ms
100.000%  111.10ms

[snip]

----------------------------------------------------------
  30099 requests in 5.00m, 9.13MB read
Requests/sec:    100.33
Transfer/sec:     31.16KB
```
