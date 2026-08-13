# EVD-024: Claude Code OAuth refresh: rotation, fixed TTL, absolute expiry

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Why this was probed

Agents run in several homes here — host, the bwrap jail, and (prospectively) the
`microvm-spike` capsules. Field experience: a shared persistent home keeps every
agent logged in, while agents given only a copy of `.credentials.json` work for a
while, then lose auth, and re-copying a known-good file does not rescue them.

The standing folk-explanation was server-side machine fingerprinting — Anthropic
binds the OAuth grant to a machine id and busts the session when they diverge.
That hypothesis is **not supported**; the real mechanism is refresh-token
rotation against a single-writer credential file, and the logout is executed by
the *local client*, not the server.

## Method

- **Static**: read the shipped bundle,
  `claude-code-2.1.223/bin/.claude-wrapped` (bun single-file, minified).
  Function names below are that build's minified identifiers — re-grep them to
  re-verify; they will not be stable across releases.
- **Live**: two real refreshes against this session's own credential on
  2026-08-14, then a CAS-shaped write-back, then an authenticated GET to
  `/api/oauth/claude_cli/roles` (200) to confirm the lineage survived.
- Endpoint `https://platform.claude.com/v1/oauth/token`, client id
  `9d1c250a-e61b-44d9-88ed-5944d1962f5e`. (The sibling `22422756-…` constant is
  the dev config — it carries `OAUTH_FILE_SUFFIX:"-local-oauth"` and a localhost
  MCP proxy.)

## Findings

**F1 — refresh rotates the refresh token, unconditionally.** Both live calls
returned a new `refresh_token`. There is no read-only refresh, and therefore no
such thing as two independent refreshers of one lineage: whoever refreshes second
presents a dead token.

**F2 — a caller-supplied `expires_in` is ignored.** `zwe()` accepts an
`expiresIn` option and forwards it as `expires_in`, but the server declines it:
asked 300, received 28800 (8h) on both calls. **Short-lived access tokens cannot
be minted.** 8h is both floor and ceiling, so a leaked access token is good for
up to 8h and the only available mitigation is minting later, not shorter.

**F3 — refresh-token expiry is absolute, not sliding.** The response's
`refresh_token_expires_in` (416149s) landed on `2026-08-18T20:34:28` — the same
instant as the `refreshTokenExpiresAt` already on disk *before* the refresh.
Rotating does not extend the family's life; it is anchored to the original
authorization, ~4.8 days. **Interactive re-login is mandatory on a ~5-day clock
regardless of how well refresh is managed.** Any design that treats re-login as
an exceptional failure path has mis-modelled the system.

**F4 — nothing device-derived is on the wire.** The first-party refresh body is
exactly `{grant_type, refresh_token, client_id, scope}`. `userID` and `machineID`
in `~/.claude.json` are `randomBytes(32).toString("hex")` generated once and
persisted (`$ae()`, `vZr()`) — not derived from hardware — and are consumed only
as the telemetry `device_id` and as `host_name_redacted` on crash reports.
`/etc/machine-id` is read solely by the bundled OpenTelemetry host-resource
detector (`getMachineId`), dormant unless OTel telemetry is enabled. A genuine
per-device credential does exist (`trustedDeviceToken`, sent as
`X-Trusted-Device-Token`, overridable by `CLAUDE_TRUSTED_DEVICE_TOKEN`) but is
absent from a normal subscription credential.

**F5 — the client coordinates concurrent refreshers via CAS on one file.** The
save path commits only if the on-disk refresh token still equals the one the
refresh started from, or is empty; it retries three times and reports
`adopted_sibling` when it loses. On `invalid_grant`, `Nfr()` zeroes its *own*
credential file (`refreshToken:""`, `accessToken:""`, `expiresAt:0`) and adds the
token to an in-process dead set — **but only when disk still holds the dead
token** (`if (!o || o.refreshToken !== e) return n`). That inequality guard is
why a correctly-written newer token is not clobbered by a losing sibling.

This is the whole explanation of the field observation. A shared home is not
carrying identity; it is carrying *concurrency control*. Split the home and the
lineage forks into uncoordinated copies of one rotating token; the loser receives
`invalid_grant` and destroys its own credential.

**F6 — access-token-only credentials are accepted and not persisted.** `cei()`
short-circuits on `!refreshToken || !expiresAt` with the
`tengu_oauth_tokens_inference_only` telemetry event and writes nothing. A
consumer handed only an access token cannot rotate, cannot race, cannot be
zeroed, and cannot revoke.

## Consequence for design

There is exactly **one refresher per lineage**, it must write back under CAS
(verified working), and everything else takes 8h bearer tokens. Delivery
env vars exist for this shape — `CLAUDE_CODE_OAUTH_TOKEN`, its
`_FILE_DESCRIPTOR` variant (never lands in the guest filesystem),
`CLAUDE_CODE_HOST_CREDS_FILE`, `CLAUDE_CODE_SDK_HAS_HOST_AUTH_REFRESH`,
`CLAUDE_CODE_PROXY_AUTH_HELPER_TTL_MS` — **names read from the bundle's env
table only; their semantics are unprobed** and are the next thing to establish
before committing to a broker interface.

The token response also carries `account` and `organization` blocks, which the
client folds into `~/.claude.json`'s `oauthAccount`. A broker should forward only
what a consumer needs to function.

## Delivery mechanisms — how a credential reaches a Claude Code process

Probed 2026-08-14, same build. Static only; none of the three below was
exercised live.

**F7 — OAuth vs API key is a branch on credential *shape*, not a header
preference.**

```js
let r = "accessToken" in e
  ? { Authorization:`Bearer ${e.accessToken}`, "anthropic-beta": gL }
  : { "x-api-key": e.apiKey };
```

A credential carrying `accessToken` is presented as `Authorization: Bearer` plus
the OAuth beta header; one carrying `apiKey` is presented as `x-api-key`. The two
are mutually exclusive by construction.

This disqualifies **`apiKeyHelper`** as the pipe for a subscription token. It is
a documented, live-reloaded settings key — a shell command producing an auth
value, refreshed on `CLAUDE_CODE_API_KEY_HELPER_TTL_MS` — and mechanically it is
the pull hook a broker wants. But it yields an untyped *string* which the docs
state is sent as **both** `X-Api-Key` and `Authorization: Bearer`, bypassing the
discriminator above. It is typed as an API key at the seam whatever is put in it.
Billing attribution for an OAuth access token routed this way is **unverified**;
do not assume subscription.

**F8 — a first-class pull interface exists over the SDK control protocol.**

```js
async requestOAuthTokenRefresh(){
  return (await this.sendRequest({subtype:"oauth_token_refresh"}, …)).accessToken }
async requestHostAuthTokenRefresh(e=bbE){
  return (await this.sendRequest({subtype:"host_auth_token_refresh"}, …)).authToken }
```

Two lanes, and they are not interchangeable: `oauth_token_refresh` → `accessToken`
is first-party subscription; `host_auth_token_refresh` → `authToken` belongs to
the third-party provider cluster (`CLAUDE_CODE_PROVIDER_MANAGED_BY_HOST`, the AWS
/ GCP credential vars → Bedrock / Vertex / Foundry). The child CLI *asks its SDK
host* to refresh — exactly the broker topology, built in.

**The gate is the catch.** The OAuth lane requires both the SDK-set flag and an
allowlisted entrypoint:

```js
Mba = new Set(["claude-desktop","local-agent","claude-vscode"]);
if (tr(env.CLAUDE_CODE_SDK_HAS_OAUTH_REFRESH) && Mba.has(env.CLAUDE_CODE_ENTRYPOINT ?? ""))
  _Ni(() => _.requestOAuthTokenRefresh());
```

A plain `cli` or `sdk-ts` entrypoint does not get it. (The host-auth lane is gated
on a provider check plus `CLAUDE_CODE_HOST_AUTH_REFRESH_TIMEOUT_MS` instead, not
on entrypoint.) `CLAUDE_CODE_ENTRYPOINT` is an ordinary env var and therefore
settable, but that is leaning on an undocumented internal gate — fragile across
releases and it misattributes telemetry. Treat as a known escape hatch, not a
design foundation.

**F9 — `CLAUDE_CODE_OAUTH_TOKEN` is the unambiguous pipe.** It sits in the auth
env cluster beside `ANTHROPIC_API_KEY` / `ANTHROPIC_AUTH_TOKEN`, is unset on
logout (`te.unset("CLAUDE_CODE_OAUTH_TOKEN")`), and is cached through the
OAuth-token setter. Its `_FILE_DESCRIPTOR` variant is resolved by a shared helper
(`bcu({envVar, wellKnownPath, label:"OAuth token", getCached, setCached,
skipInReviewOrigin})`) — the token arrives on a file descriptor and never lands
in the guest filesystem. Combined with F6 (access-token-only credentials are
accepted and not persisted) this is a push, not a pull: mint per process start,
accept the 8h ceiling, restart to renew.

## Documented surface — and a correction to F9

Third round, 2026-08-14. Source here is **official documentation**, fetched live
(`code.claude.com/docs/en/{env-vars,authentication}.md`), not scraping. The local
`docs/claude/` cache did not carry either page and was five weeks stale; that is
fixed separately in `docs/claude/fetch.sh`.

**F10 — `claude setup-token` mints a one-year OAuth token.** From the
authentication doc: *"generate a one-year OAuth token with `claude setup-token`
… It does not save the token anywhere; copy it and set it as the
`CLAUDE_CODE_OAUTH_TOKEN` environment variable."* The bundle corroborates —
`elt=31536000` is one year in seconds.

**This corrects F9.** F9 concluded that `CLAUDE_CODE_OAUTH_TOKEN` implied an 8h
ceiling with restart-to-renew. The pipe was right; the lifetime was wrong. The 8h
of F2 governs an access token derived from a `/login` grant. A `setup-token`
grant is **separate, year-long, and never written to `.credentials.json`** — so
it does not join the rotation lineage at all. For a fan-out design this dissolves
F1/F5 entirely: no fork, no CAS, no `invalid_grant` self-destruct, and no ~5-day
re-login clock (F3) for a process authenticated this way.

**F11 — the credential precedence is documented**, and `apiKeyHelper` outranks
the OAuth token:

1. gateway/provider sessions · 2. `ANTHROPIC_AUTH_TOKEN` (`Authorization: Bearer`)
· 3. `ANTHROPIC_API_KEY` (`X-Api-Key`) · 4. `apiKeyHelper` · 5.
`CLAUDE_CODE_OAUTH_TOKEN` · 6. Anthropic profile / federation · 7. subscription
OAuth from `/login`.

Consistent with F7: the doc describes `apiKeyHelper` as being for *"dynamic or
rotating credentials, such as short-lived tokens fetched from a vault"* — the
broker pattern exactly — but it remains the API-key-typed seam.

**F12 — `CLAUDE_CODE_OAUTH_REFRESH_TOKEN` is a documented provisioning path.**
With `CLAUDE_CODE_OAUTH_SCOPES` set, `claude auth login` exchanges the refresh
token directly instead of opening a browser — *"useful for provisioning
authentication in automated environments."* Note this hands the consumer a
**refresh token**, so F1/F5 apply in full: only safe when the consumer owns its
own grant, never when copies of one grant are distributed.

**F13 — the SDK control-protocol path (F8) is undocumented.** The live Agent SDK
TypeScript reference (4820 lines, vs the 3550-line five-week-old cached copy)
documents no OAuth-refresh callback — only `ApiKeySource` (`"user" | "project" |
"org" | "temporary" | "oauth"`) as a *reported* value on the init message. So
`oauth_token_refresh` is internal, entrypoint-gated, and unsupported. **Do not
build on it**; the documented env-var surface covers the same need.

### Where that leaves a fan-out design

Per-consumer `setup-token` grants need no broker, no shared lineage, and no
re-login clock, at the cost of a year-long bearer credential living inside the
consumer. A broker minting 8h tokens holds guest exposure to 8h — and is now
cheaper than when first costed, because a `setup-token` can authenticate the
*broker*, removing its own re-login problem. The trade is containment against
moving parts; nothing here decides it.

## Limits of this evidence

Single build (2.1.223), single subscription (`max`, `default_claude_max_5x`),
single probe date. F2 and F3 are server-side policy and may differ by plan or
change without a client release. Minified identifiers are build-local.
