import http from 'k6/http';

export let options = {
  // https://k6.io/docs/using-k6/scenarios/executors/constant-arrival-rate
  scenarios: {
    constant: {
      executor: 'constant-arrival-rate',
      rate: 100, // request per timeUnit (default: 1s)
      duration: '20m',
      preAllocatedVUs: 100,
      maxVUs: 100,
    },
  },
  thresholds: {
    "http_req_duration": ["p(99.9)<300"],
  },

  // noCookiesReset: true,
  // noConnectionReuse: true,
  // noVUConnectionReuse: true,
  discardResponseBodies: true,

  // summaryTimeUnit: 'ms',
  summaryTrendStats: ['p(99)', 'p(99.9)', 'p(99.99)', 'p(99.999)', 'max', 'count'],

  noUsageReport: true,
};

export default function () {
  http.get('http://localhost:8080/');
}
