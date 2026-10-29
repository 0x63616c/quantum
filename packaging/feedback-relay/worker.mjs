// No payload, response body, contact address or IP is logged by this service.
const REQUEST_LIMIT = 96 * 1024;
const RESPONSE_LIMIT = 256 * 1024;
const DEADLINE_MS = 8000;
const REPOSITORY = '0x63616c/quantum';
const API = `https://api.github.com/repos/${REPOSITORY}/issues`;

export function reply(status, code) {
  return Response.json({ code }, { status, headers: { 'Cache-Control': 'no-store' } });
}

export async function boundedText(stream, limit, signal) {
  if (!stream) throw new Error('missing body');
  const reader = stream.getReader();
  const cancel = () => { void reader.cancel().catch(() => {}); };
  signal.addEventListener('abort', cancel, { once: true });
  let size = 0;
  const chunks = [];
  try {
    while (true) {
      signal.throwIfAborted();
      const { value, done } = await reader.read();
      signal.throwIfAborted();
      if (done) break;
      size += value.byteLength;
      if (size > limit) throw new Error('size');
      chunks.push(value);
    }
    const bytes = new Uint8Array(size);
    let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.length; }
    return new TextDecoder('utf-8', { fatal: true }).decode(bytes);
  } finally {
    signal.removeEventListener('abort', cancel);
    void reader.cancel().catch(() => {});
    reader.releaseLock();
  }
}

export function validate(p) {
  const keys = ['schema_version', 'title', 'body', 'labels', 'contact_email', 'kind'];
  if (!p || typeof p !== 'object' || Array.isArray(p) ||
      Object.keys(p).some(key => !keys.includes(key)) || p.schema_version !== 1 ||
      !['bug', 'feature'].includes(p.kind) ||
      typeof p.title !== 'string' || !p.title.trim() || p.title.length > 200 ||
      /[\r\n\x00-\x1f\x7f]/u.test(p.title) ||
      typeof p.body !== 'string' || !p.body.trim() || p.body.length > 60000 ||
      !Array.isArray(p.labels) || p.labels.length !== 1 ||
      p.labels[0] !== (p.kind === 'bug' ? 'type:fix' : 'type:feat') ||
      (p.contact_email != null && (typeof p.contact_email !== 'string' ||
       p.contact_email.length > 254 || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/u.test(p.contact_email)))) {
    throw new Error('validation');
  }
  // Defence in depth: the public fields must not contain the private address.
  if (p.contact_email && (p.title.toLowerCase().includes(p.contact_email.toLowerCase()) ||
      p.body.toLowerCase().includes(p.contact_email.toLowerCase()))) throw new Error('contact');
  // Explicitly select public fields. Contact is discarded; zero application retention.
  return { title: p.title, body: p.body, labels: p.labels };
}

// Dependency injection is local-test-only; the production handler always uses API.
export async function handle(request, env, { fetcher = fetch, api = API, timeout = DEADLINE_MS } = {}) {
  if (new URL(request.url).pathname !== '/reports') return reply(404, 'not_found');
  if (request.method !== 'POST') return reply(405, 'method_not_allowed');
  if (!env.GITHUB_TOKEN || !env.ABUSE) return reply(503, 'unconfigured');
  if (request.headers.get('content-type')?.split(';')[0].trim() !== 'application/json') {
    return reply(415, 'json_required');
  }
  const length = request.headers.get('content-length');
  if (length && (!/^\d+$/u.test(length) || Number(length) > REQUEST_LIMIT)) return reply(413, 'too_large');
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeout);
  try {
    // Cloudflare overwrites CF-Connecting-IP. Never trust X-Forwarded-For.
    const ip = request.headers.get('CF-Connecting-IP');
    if (!ip) return reply(403, 'source_required');
    const hour = Math.floor(Date.now() / 3600000);
    const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(`${hour}:${ip}`));
    const key = Array.from(new Uint8Array(digest), b => b.toString(16).padStart(2, '0')).join('');
    const stub = env.ABUSE.get(env.ABUSE.idFromName('global'));
    const gate = await stub.fetch('https://internal/limit', {
      method: 'POST', body: JSON.stringify({ key, hour }), signal: controller.signal,
    });
    if (gate.status !== 204) return reply(gate.status === 429 ? 429 : 503, 'rate_limited');
    let publicIssue;
    try {
      publicIssue = validate(JSON.parse(await boundedText(request.body, REQUEST_LIMIT, controller.signal)));
    } catch {
      return reply(controller.signal.aborted ? 504 : 422, 'invalid_report');
    }
    const upstream = await fetcher(api, {
      method: 'POST', redirect: 'manual', signal: controller.signal,
      headers: { Authorization: `Bearer ${env.GITHUB_TOKEN}`, 'Content-Type': 'application/json',
        Accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28',
        'User-Agent': 'Quantum-feedback-relay' },
      body: JSON.stringify(publicIssue),
    });
    if (upstream.status !== 201) { void upstream.body?.cancel(); return reply(502, 'github_failed'); }
    const issue = JSON.parse(await boundedText(upstream.body, RESPONSE_LIMIT, controller.signal));
    if (!Number.isSafeInteger(issue.number) || issue.number <= 0 ||
        issue.html_url !== `https://github.com/${REPOSITORY}/issues/${issue.number}`) {
      return reply(502, 'github_unconfirmed');
    }
    return reply(201, 'created');
  } catch {
    return reply(controller.signal.aborted ? 504 : 502, 'unconfirmed');
  } finally { clearTimeout(timer); }
}

export default { fetch: (request, env) => handle(request, env) };

// One globally coordinated object; fixed hour windows, persisted across isolates.
// Keys are hour-salted IP hashes, never raw addresses. No report/contact storage.
export class AbuseGate {
  constructor(ctx) { this.ctx = ctx; }
  async fetch(request) {
    const { key, hour } = await request.json();
    if (!/^[a-f0-9]{64}$/u.test(key) || !Number.isSafeInteger(hour)) return reply(400, 'invalid');
    const now = Math.floor(Date.now() / 3600000);
    if (hour !== now) return reply(429, 'expired');
    const allowed = await this.ctx.storage.transaction(async storage => {
      const state = await storage.get('window') ?? { hour, total: 0, clients: {} };
      const current = state.hour === hour ? state : { hour, total: 0, clients: {} };
      if (current.total >= 30 || (current.clients[key] ?? 0) >= 3) return false;
      current.total += 1;
      current.clients[key] = (current.clients[key] ?? 0) + 1;
      await storage.put('window', current);
      await storage.setAlarm((hour + 1) * 3600000);
      return true;
    });
    return allowed ? new Response(null, { status: 204 }) : reply(429, 'rate_limited');
  }
  async alarm() {
    await this.ctx.storage.transaction(async storage => {
      const state = await storage.get('window');
      if (!state || state.hour < Math.floor(Date.now() / 3600000)) await storage.delete('window');
      else await storage.setAlarm((state.hour + 1) * 3600000);
    });
  }
}
