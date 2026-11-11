// k6 load test for the rate limiter's /ping endpoint.
//
// Start the server first (`cargo run --release`), then:
//   k6 run k6/load-test.js
//
// Options (environment variables):
//   BASE_URL   server address                         (default http://127.0.0.1:3000)
//   RATE       requests per second k6 sends            (default 1000)
//   DURATION   how long the test runs                  (default 30s)
//   CLIENTS    how many different clients to pretend to be (default 10)
//
// k6 sends RATE requests per second no matter how fast the server answers
// (constant-arrival-rate). That's what you want for a rate limiter: you pick a
// rate above the limit and check that the extra requests get 429.

import http from 'k6/http';
import { check } from 'k6';
import { Counter } from 'k6/metrics';

const BASE_URL = __ENV.BASE_URL || 'http://127.0.0.1:3000';
const RATE = parseInt(__ENV.RATE || '1000');
const DURATION = __ENV.DURATION || '10s';
const CLIENTS = parseInt(__ENV.CLIENTS || '10');

// Counted separately so the summary shows how many requests were let through
// and how many were rejected by the rate limiter.
const allowed = new Counter('requests_allowed');
const limited = new Counter('requests_rate_limited');

export const options = {
  scenarios: {
    ping: {
      executor: 'constant-arrival-rate',
      rate: RATE,
      timeUnit: '1s',
      duration: DURATION,
      preAllocatedVUs: 50,
      maxVUs: 500,
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.01'],      // under 1% errors (429 is not an error, see below)
    http_req_duration: ['p(95)<50'],     // 95% of requests under 50ms
    checks: ['rate>0.99'],
    dropped_iterations: ['count==0'],    // k6 managed to send every request it planned
  },
};

// 429 Too Many Requests is the rate limiter doing its job, so don't count it as failed.
http.setResponseCallback(http.expectedStatuses(200, 429));

export function setup() {
  const res = http.get(`${BASE_URL}/ping`);
  if (res.status !== 200) {
    throw new Error(`server not answering at ${BASE_URL}/ping (status ${res.status})`);
  }
}

export default function () {
  // Pretend to be one of CLIENTS different users. The middleware can use this
  // header to give each client its own token bucket. (Every request from k6
  // comes from 127.0.0.1, so limiting by IP would treat them all as one client.)
  const clientId = `client-${Math.floor(Math.random() * CLIENTS)}`;
  const res = http.get(`${BASE_URL}/ping`, { headers: { 'X-Client-Id': clientId } });

  if (res.status === 200) {
    allowed.add(1);
  } else if (res.status === 429) {
    limited.add(1);
  }

  check(res, {
    'status is 200 or 429': (r) => r.status === 200 || r.status === 429,
    'allowed requests say Hello world': (r) =>
      r.status !== 200 || r.json('response') === 'Hello world',
  });
}
