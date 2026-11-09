// k6 load test for the URL shortener.
//
// Start the server first (`cargo run --release`), then:
//   k6 run k6/load-test.js
//
// Options (environment variables):
//   BASE_URL   server address          (default http://127.0.0.1:3000)
//   DURATION   how long each scenario runs (default 30s)
//
// Every run adds rows to urls.db. Delete urls.db to start from an empty table.

import http from 'k6/http';
import { check, fail } from 'k6';

const BASE_URL = __ENV.BASE_URL || 'http://127.0.0.1:3000';
const DURATION = __ENV.DURATION || '30s';
const SEED_COUNT = 100;

// Alex Xu's design assumes reads (redirects) far outnumber writes (shortens),
// so the redirect scenario runs with 10x the virtual users.
export const options = {
  scenarios: {
    shorten: {
      executor: 'constant-vus',
      exec: 'shortenUrl',
      vus: 5,
      duration: DURATION,
    },
    redirect: {
      executor: 'constant-vus',
      exec: 'followShortUrl',
      vus: 50,
      duration: DURATION,
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.01'],                               // under 1% errors
    'http_req_duration{scenario:shorten}': ['p(95)<200'],         // 95% of shortens under 200ms
    'http_req_duration{scenario:redirect}': ['p(95)<100'],        // 95% of redirects under 100ms
    checks: ['rate>0.99'],
  },
};

const JSON_HEADERS = { headers: { 'Content-Type': 'application/json' } };

function shorten(longUrl) {
  return http.post(`${BASE_URL}/shorten`, JSON.stringify({ long_url: longUrl }), JSON_HEADERS);
}

// "http://127.0.0.1:3000/2TX" -> "2TX"
function codeFrom(res) {
  return res.json('short_url').split('/').pop();
}

// Runs once before the scenarios start: creates URLs to redirect to and
// checks the basic behaviour.
export function setup() {
  const codes = [];
  for (let i = 0; i < SEED_COUNT; i++) {
    const longUrl = `https://example.com/seed/${Date.now()}/${i}`;
    const res = shorten(longUrl);
    if (!check(res, { 'seed: status 200': (r) => r.status === 200 })) {
      fail(`could not seed URLs, is the server running at ${BASE_URL}? (status ${res.status})`);
    }
    codes.push({ code: codeFrom(res), longUrl });
  }

  // Shortening the same URL twice must return the same code.
  const again = shorten(codes[0].longUrl);
  check(again, { 'setup: same URL gives same code': (r) => codeFrom(r) === codes[0].code });

  // An unknown code must give 404 (and it shouldn't count as a failed request).
  const missing = http.get(`${BASE_URL}/doesNotExist123`, {
    redirects: 0,
    responseCallback: http.expectedStatuses(404),
  });
  check(missing, { 'setup: unknown code gives 404': (r) => r.status === 404 });

  return { codes };
}

// Write path: every iteration shortens a brand-new URL.
export function shortenUrl() {
  const longUrl = `https://example.com/load/${__VU}/${__ITER}/${Date.now()}`;
  const res = shorten(longUrl);
  check(res, {
    'shorten: status 200': (r) => r.status === 200,
    'shorten: returns a short_url': (r) => typeof r.json('short_url') === 'string',
  });
}

// Read path: pick a seeded code and request it.
// redirects: 0 so k6 measures our server, not example.com.
export function followShortUrl(data) {
  const { code, longUrl } = data.codes[Math.floor(Math.random() * data.codes.length)];
  const res = http.get(`${BASE_URL}/${code}`, { redirects: 0 });
  check(res, {
    'redirect: status 307': (r) => r.status === 307,
    'redirect: location is the long URL': (r) => r.headers['Location'] === longUrl,
  });
}
