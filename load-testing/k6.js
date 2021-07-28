import http from 'k6/http';

export let options = {
  stages: [
    { duration: '20s', target: 5 },
    { duration: '1m', target: 5 },
    { duration: '20s', target: 10 },
    { duration: '1m', target: 10 },
    { duration: '10s', target: 3 },
    { duration: '1m', target: 3 },
    { duration: '15s', target: 5 },
    { duration: '2m', target: 5 },
    { duration: '20s', target: 15 },
    { duration: '2m', target: 20 },
    { duration: '2m', target: 10 },
    { duration: '10s', target: 2 },
    { duration: '1m', target: 2 },
    { duration: '15s', target: 6 },
    { duration: '2m', target: 6 },
    { duration: '20s', target: 10 },
    { duration: '1m', target: 10 },
    { duration: '30s', target: 2 },
    { duration: '15s', target: 1 },
    { duration: '5s', target: 0 },
  ],

  noCookiesReset: true,
  noConnectionReuse: true,
  noVUConnectionReuse: true,
  discardResponseBodies: true,

  // summaryTimeUnit: 'ms',
  summaryTrendStats: ['p(99)', 'p(99.9)', 'p(99.99)', 'p(99.999)', 'max', 'count'],

  noUsageReport: true,
};

export default function () {
  http.get('http://localhost:8080/');
}
