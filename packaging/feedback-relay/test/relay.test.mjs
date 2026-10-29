import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { readFileSync } from 'node:fs';
import { handle, validate, AbuseGate } from '../worker.mjs';

const payload = () => JSON.parse(readFileSync(new URL('../../../crates/quantum-crash/tests/fixtures/relay-v1.json', import.meta.url)));
const env = { GITHUB_TOKEN: 'local-fake-token', ABUSE: {
  idFromName: name => name, get: () => ({ fetch: async () => new Response(null, { status: 204 }) }),
} };
const request = (p = payload(), headers = {}) => new Request('https://relay.invalid/reports', {
  method: 'POST', headers: { 'Content-Type': 'application/json', 'CF-Connecting-IP': '192.0.2.1', ...headers },
  body: typeof p === 'string' ? p : JSON.stringify(p),
});
const confirmation = () => Response.json({ number: 7, html_url: 'https://github.com/0x63616c/quantum/issues/7' }, { status: 201 });

test('schema, exact label allowlist, private address and unknown fields', () => {
  assert.deepEqual(Object.keys(validate(payload())), ['title', 'body', 'labels']);
  for (const update of [ { schema_version: 2 }, { labels: ['priority:p0'] }, { labels: [] },
    { labels: ['type:fix', 'type:feat'] }, { kind: 'feature' }, { kind: 'Crash' },
    { title: '' }, { title: 'a\nb' }, { title: 'x'.repeat(201) }, { body: '' },
    { body: 'x'.repeat(60001) }, { contact_email: 'invalid' }, { scene: 'private scene' },
    { body: 'private@example.invalid' }, { title: 'PRIVATE@example.invalid' } ]) {
    assert.throws(() => validate({ ...payload(), ...update }));
  }
  assert.doesNotThrow(() => validate({ ...payload(), kind: 'feature', labels: ['type:feat'] }));
});

test('HTTP limits, malformed input, unavailable configuration and gate fail closed', async () => {
  let calls = 0;
  const options = { fetcher: async () => { calls++; return confirmation(); } };
  for (const [req, status] of [ [request('broken'), 422], [request('x'.repeat(98305)), 422],
    [request(payload(), { 'Content-Length': '98305' }), 413],
    [request(payload(), { 'Content-Type': 'text/plain' }), 415],
    [request(payload(), { 'CF-Connecting-IP': '' }), 403],
    [new Request('https://relay.invalid/elsewhere'), 404],
    [new Request('https://relay.invalid/reports'), 405] ]) {
    assert.equal((await handle(req, env, options)).status, status);
  }
  assert.equal((await handle(request(), {}, options)).status, 503);
  const blocked = { ...env, ABUSE: { ...env.ABUSE, get: () => ({ fetch: async () => new Response(null, { status: 429 }) }) } };
  assert.equal((await handle(request(), blocked, options)).status, 429);
  assert.equal(calls, 0);
});

test('response caps, redirects, malformed confirmations, failures and timeouts never accept or retry', { timeout: 5000 }, async () => {
  for (const response of [new Response(null, { status: 302 }), new Response(null, { status: 403 }),
    new Response(null, { status: 500 }), new Response('not JSON', { status: 201 }),
    Response.json({ number: 7, html_url: 'https://other.invalid' }, { status: 201 }),
    new Response('x'.repeat(262145), { status: 201 })]) {
    let calls = 0;
    const result = await handle(request(), env, { fetcher: async (_, init) => {
      calls++; assert.equal(init.redirect, 'manual'); return response;
    } });
    assert.equal(result.status, 502); assert.equal(calls, 1);
    assert.equal((await result.text()).includes('private@'), false);
  }
  const timeout = await handle(request(), env, { timeout: 10, fetcher: async (_, init) => {
    init.signal.throwIfAborted();
    await new Promise((_, reject) => init.signal.addEventListener('abort', () => reject(new Error('timeout')), { once: true }));
  } });
  assert.equal(timeout.status, 504);
  const stream = new ReadableStream({ start() {} });
  assert.equal((await handle(new Request('https://relay.invalid/reports', { method: 'POST',
    headers: { 'Content-Type': 'application/json', 'CF-Connecting-IP': '192.0.2.1' },
    body: stream, duplex: 'half' }), env, { timeout: 10 })).status, 504);
});

test('persistent rate state enforces per-IP and global limits, rollover and retention', async () => {
  let saved; let alarm;
  const storage = {
    get: async () => saved, put: async (_, value) => { saved = structuredClone(value); },
    setAlarm: async value => { alarm = value; }, delete: async () => { saved = undefined; },
    transaction: async fn => fn(storage),
  };
  const send = async key => new AbuseGate({ storage }).fetch(new Request('https://internal/limit', {
    method: 'POST', body: JSON.stringify({ key: key.toString(16).padStart(64, '0'), hour: Math.floor(Date.now() / 3600000) }),
  }));
  for (let n = 0; n < 3; n++) assert.equal((await send(1)).status, 204);
  assert.equal((await send(1)).status, 429);
  for (let n = 2; n <= 28; n++) assert.equal((await send(n)).status, 204);
  assert.equal((await send(29)).status, 429);
  assert.ok(alarm > Date.now() && alarm <= Date.now() + 3600000);
  saved.hour--;
  assert.equal((await send(1)).status, 204);
  await new AbuseGate({ storage }).alarm(); assert.ok(saved); // delayed alarm cannot erase current quota
  saved.hour--;
  await new AbuseGate({ storage }).alarm(); assert.equal(saved, undefined);
});

test('local end-to-end HTTP fake GitHub: accept only confirmed creation; contact never crosses boundary', async t => {
  const received = [];
  let mode = 201;
  const github = createServer(async (req, res) => {
    let body = ''; for await (const chunk of req) body += chunk;
    received.push(JSON.parse(body));
    assert.equal(req.headers.authorization, 'Bearer local-fake-token');
    res.writeHead(mode, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ number: 7, html_url: 'https://github.com/0x63616c/quantum/issues/7' }));
  }).listen(0, '127.0.0.1');
  await once(github, 'listening'); t.after(() => github.close());
  const relay = createServer(async (req, res) => {
    let body = ''; for await (const chunk of req) body += chunk;
    const response = await handle(request(body), env, {
      api: `http://127.0.0.1:${github.address().port}/repos/0x63616c/quantum/issues`,
    });
    res.writeHead(response.status, Object.fromEntries(response.headers)); res.end(await response.text());
  }).listen(0, '127.0.0.1');
  await once(relay, 'listening'); t.after(() => relay.close());
  const submit = () => fetch(`http://127.0.0.1:${relay.address().port}/reports`, { method: 'POST', body: JSON.stringify(payload()) });
  assert.equal((await submit()).status, 201);
  assert.deepEqual(received[0], { title: 'Bug report', body: 'Redacted preview', labels: ['type:fix'] });
  mode = 503; assert.equal((await submit()).status, 502);
  assert.equal(received.length, 2); // exactly one upstream attempt per explicit submission
  assert.equal(JSON.stringify(received).includes('private@example.invalid'), false);
});
