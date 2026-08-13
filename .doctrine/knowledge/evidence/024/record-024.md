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

## Limits of this evidence

Single build (2.1.223), single subscription (`max`, `default_claude_max_5x`),
single probe date. F2 and F3 are server-side policy and may differ by plan or
change without a client release. Minified identifiers are build-local.
