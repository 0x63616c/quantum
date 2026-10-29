# Quantum feedback relay (deployment pending owner approval)

Cloudflare Workers fits the owner's existing Cloudflare infrastructure. A SQLite
Durable Object coordinates quotas across isolates; Vercel continues to host the
static site. Nothing here provisions a resource or configures a default endpoint.
The app still uses the honest browser fallback unless `QUANTUM_REPORT_ENDPOINT`
is explicitly configured at runtime or when compiling the app.

## Protocol and privacy

`POST /reports`, JSON `quantum-crash::RelayPayload` schema 1. Only bug/type:fix and
feature/type:feat pairs are accepted. Unknown fields (including scene bytes),
empty/oversized public fields and malformed contact addresses are rejected.
Only title, body and the single allowed label are sent to the fixed repository
`0x63616c/quantum`. There is no client token and no arbitrary upstream destination.

A contact address is validated then immediately discarded, never logged, stored,
forwarded or used to contact the sender (zero application retention). The app
therefore has no contact field. If the private address occurs in a public field,
the request is rejected. No scene attachments are supported. Free text can still
contain private details; redaction and full preview review remain necessary.

The relay stores no report payloads. GitHub issues are public and retained until
removed by GitHub/the repository owner; deletion of a local report does not delete
an issue. Cloudflare necessarily processes IP/network metadata under its own
[privacy policy](https://www.cloudflare.com/privacypolicy/). Application logs and
preview URLs are disabled; do not enable request/body capture, Logpush or tail
sessions containing reports. The rate store contains only counters and hourly
IP hashes, up to 30 keys, expires at the hour boundary (alarm delivery may be
late), and drops old state on the next request. Hashes are pseudonymous, not
anonymous. No report/contact data enters that store.

Limits: 96 KiB streamed request, 200-character title, 60,000-character body,
256 KiB streamed GitHub response, 8-second relay deadline covering reads and
upstream work, fixed-window 3 attempts/IP/hour and 30 globally/hour. Failed and
invalid attempts consume quota. Cloudflare must overwrite CF-Connecting-IP;
never expose the handler through a proxy that permits forged values. This is
bounded abuse mitigation, not proof of human identity; monitor aggregate billing
and pause the Worker if abuse exhausts quota. No CORS allowance is provided.

Only GitHub HTTP 201 with a bounded valid repository issue URL/number produces
201 `created`. Every other result is a non-2xx error. There is one upstream
attempt, no retry or queue. Lost responses can mean an issue was created without
confirmation; inspect GitHub before a manual retry. Local crash files are never
deleted on submission. The app uses system curl with curlrc, redirects and retries
disabled, 5-second connect/15-second total limits and a 4 KiB relay response cap.
macOS includes curl; Linux installations require it. The temporary request
file is private and removed on completion; responses are capped while reading
stdout, including on older curl versions without streamed size enforcement. A missing curl fails visibly.

## Local evidence (no credentials or external resources)

From the repository root, Node 22 or newer:

```sh
node --test packaging/feedback-relay/test/*.test.mjs
```

The end-to-end test starts two loopback HTTP servers: the relay adapter and a fake
GitHub API. It asserts exactly one issue creation attempt, matching public fields,
no private address on the GitHub side, 201 acceptance, and 502 on GitHub failure.
Other tests cover validation, quotas across object instances, cleanup races,
malformed/oversized requests and replies, redirects, missing configuration,
unconfirmed responses and slow body/upstream timeouts. CI Linux runs these tests.
A local Wrangler 4.155.0 bundle dry run also passed. Local `wrangler dev --local`
with a fake token returned 422 for three invalid reports then 429 on the fourth,
exercising the real Durable Object storage/transaction/alarm scheduling API.
These are local tests, not evidence of deployed Cloudflare routing,
Durable Object runtime behavior, real GitHub permissions or AppKit interaction.

## Owner runbook — do not execute until approved

1. Approve the Cloudflare account, existing zone, destination hostname, quota and
   zero-contact-retention policy above. Proposed production URL:
   `https://quantum-feedback.worldwidewebb.co/reports`. This is a proposal, not a
   provisioned host or live service. Approve DNS/routing and Cloudflare billing.
   Use the existing zone; if DNS must be created, the owner performs that action.
2. Install the locally verified Wrangler 4.155.0, authenticate as the owner,
   and inspect [Wrangler configuration](https://developers.cloudflare.com/workers/wrangler/configuration/).
   Run `npm install --global wrangler@4.155.0` on the owner machine only.
   Set `WRANGLER_SEND_METRICS=false` in the shell.
   No Wrangler credentials belong in the repository. Replace `account_id`, route
   pattern (`quantum-feedback.worldwidewebb.co/reports`) and zone name
   (`worldwidewebb.co`) in `wrangler.jsonc`. Keep workers.dev, preview URLs and
   observability disabled. Record the approved Wrangler version with deployment
   evidence. The v1 migration creates the SQLite Durable Object only on deploy.
3. Create a fine-grained GitHub token restricted to **0x63616c/quantum**, with
   **Issues: read and write** and implicit **Metadata: read** only (see
   [GitHub issue permissions](https://docs.github.com/en/rest/issues/issues#create-an-issue)). Set an expiry
   and owner rotation reminder. No Contents/Admin permissions. No GitHub App is
   required. An existing owner-managed App installation token with these same
   repository permissions is an alternative only if the owner supplies rotation
   before its short expiry; this service does not mint/refresh App tokens.
4. From `packaging/feedback-relay`, after approving resource creation, run:

   ```sh
   wrangler login
   wrangler deploy --dry-run
   wrangler deploy
   wrangler secret put GITHUB_TOKEN
   ```

   Enter the token at the secret prompt; never put it in a command, file, build
   variable or PR. The Worker fails closed with 503 before the secret exists.
   Cloudflare deployment authorization uses the owner's Wrangler login, or an
   owner-managed `CLOUDFLARE_API_TOKEN`. Under Cloudflare's current role model,
   initial creation requires Workers product **Admin**; subsequent deploys need
   **Editor** for this Worker plus **Workers Routes Write** for the approved zone.
   Legacy token UI names are Workers Scripts Edit and Workers Routes Edit; grant
   Account Settings Read if Wrangler requires account discovery. Keep this
   deployment credential outside the app/build. See
   [Cloudflare permissions](https://developers.cloudflare.com/workers/authorization/workers/). `GITHUB_TOKEN` is the only runtime secret.
5. Test the approved HTTPS URL with an explicitly public synthetic report:

   ```sh
   curl --fail-with-body --max-time 15 --request POST \
     --header 'Content-Type: application/json' \
     --data '{"schema_version":1,"title":"Relay acceptance check","body":"Owner-approved synthetic test. No personal data.","labels":["type:fix"],"kind":"bug"}' \
     https://quantum-feedback.worldwidewebb.co/reports
   ```

   Expect 201 `created`; inspect the resulting public issue and close it. Verify
   invalid labels fail, quota rejects the fourth attempt, revoked token produces
   non-2xx, and no diagnostics/contact appear in Cloudflare logs. Verify no
   public workers.dev route exists. Record evidence on #206. Test actual Durable
   Object rollover and bounded failures before release.
6. Only after endpoint/privacy approval, compile the release app with
   `QUANTUM_REPORT_ENDPOINT=https://quantum-feedback.worldwidewebb.co/reports`
   in its build environment. For an owner-only local Mac QA package (no publishing):

   ```sh
   QUANTUM_REPORT_ENDPOINT=https://quantum-feedback.worldwidewebb.co/reports cargo xtask macos package
   ```

   After QA and approval, configure the optional release-build repository variable:

   ```sh
   gh variable set QUANTUM_REPORT_ENDPOINT --repo 0x63616c/quantum --body https://quantum-feedback.worldwidewebb.co/reports
   ```

   The release workflow reads that variable only for its app build step. Leaving
   it unset/empty preserves the browser fallback; no client token is involved.
   Runtime override is also available for owner QA. No endpoint is committed.
   On a physical Mac,
   verify Welcome crash review, feedback, both themes, confirmation, disconnected
   failure, copy/save and retained crash file, and confirm there is no background
   retry. Publish the matching site privacy update naming the now-live endpoint;
   current notes correctly say deployment is pending. Run normal signed release
   qualification separately; this PR does not authorize publishing.
7. Rotate with `wrangler secret put GITHUB_TOKEN`, then revoke the old token.
   For incidents, disable the Worker route/revoke the GitHub token immediately;
   remove the endpoint from subsequent app builds for browser fallback:

   ```sh
   gh variable delete QUANTUM_REPORT_ENDPOINT --repo 0x63616c/quantum
   ```

   Existing
   configured apps show a failure and retain reports. Do not promise fallback
   browser submission after a failed relay attempt. Remove the Worker and its
   Durable Object namespace via the owner's dashboard if retiring the service.

## Deployment checklist

- [ ] Owner approves destination, policy, account, DNS, resources and costs
- [ ] Owner provisions restricted GitHub credential and deploys Worker/secret
- [ ] Real HTTPS/GitHub/Cloudflare acceptance and failure checks recorded
- [ ] Physical Mac submission/failure and privacy review recorded
- [ ] Owner configures endpoint for release and publishes matching privacy notes

Until these items are met, #206's deployed-endpoint and deployed-app criteria
remain unchecked. Merging the code alone does not enable network submission.
