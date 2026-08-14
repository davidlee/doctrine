Claude Code's OAuth refresh **rotates the refresh token on every use**. Exactly
one process-group may refresh a given credential lineage. Proof and method:
`EVD-024` (probed against build 2.1.223, 2026-08-14).

## Why a shared home keeps every agent logged in

It is not carrying identity — it is carrying **concurrency control**. The client
coordinates concurrent refreshers by compare-and-swap on one
`~/.claude/.credentials.json`: it commits a refreshed token only if the on-disk
refresh token still equals the one it started from, retries three times, and
reports `adopted_sibling` when it loses. Every agent sharing that file is
serialised through it.

## Why copying the credential file into a second home breaks auth

You fork the lineage into uncoordinated copies of one rotating token. It works
until the access token expires (up to 8h — nothing fails before the first
refresh), then the loser gets `invalid_grant` and **the client zeroes its own
credential file** (`refreshToken:""`, `accessToken:""`, `expiresAt:0`).

It presents as a server-side lockout. It is local self-destruction. No hardware
fingerprinting is involved: the refresh body is exactly
`{grant_type, refresh_token, client_id, scope}`, and the `userID` / `machineID`
in `~/.claude.json` are random-and-persisted telemetry ids, not derived from the
machine.

Re-copying a known-good file does not rescue it — the copy is invalidated by the
source's next refresh, and a still-running client holds the dead token in memory.

## Fixed quantities — do not design around changing them

- **Access token TTL is 8h, non-negotiable.** A caller-supplied `expires_in` is
  accepted by the client and ignored by the server. Short-lived tokens cannot be
  minted; shorten *exposure* by minting later, not shorter.
- **Refresh-token expiry is absolute, not sliding** (~4.8 days, anchored to the
  original login). Rotating does not extend it. **Interactive re-login is
  mandatory on that clock** — treat it as routine, never as an exceptional
  failure path.

## Fanning out to isolated environments

One broker holds the lineage and writes back under the same CAS discipline;
every consumer gets an **access-token-only** credential. The client accepts one
(`!refreshToken || !expiresAt` → `tengu_oauth_tokens_inference_only`) and does
not persist it, so a consumer cannot rotate, cannot race, cannot be zeroed, and
cannot revoke the grant for everyone via `/logout`.

Blast radius to respect: any home sharing the file shares the failure. One
misbehaving isolated agent can log out your interactive sessions.

## Which pipe delivers the token

Use **`CLAUDE_CODE_OAUTH_TOKEN`** (or its `_FILE_DESCRIPTOR` variant, which keeps
the token off the guest filesystem). It is the unambiguous OAuth lane.

Do **not** use `apiKeyHelper` for a subscription token. The client branches on
credential *shape* — `"accessToken" in e` yields `Authorization: Bearer` plus the
OAuth beta header, otherwise `x-api-key` — and `apiKeyHelper` produces an untyped
string that is stamped into *both* headers, bypassing that discriminator. It is
typed as an API key at the seam whatever you feed it; billing attribution is
unverified.

A first-class pull interface does exist — the SDK control protocol's
`oauth_token_refresh` request, which returns an `accessToken` from the SDK host
(distinct from `host_auth_token_refresh`, which is the Bedrock/Vertex/Foundry
lane). But it is gated on `CLAUDE_CODE_SDK_HAS_OAUTH_REFRESH` **and** an
allowlisted `CLAUDE_CODE_ENTRYPOINT` (`claude-desktop`, `local-agent`,
`claude-vscode`) — a plain CLI entrypoint does not get it.

Detail and the re-greppable identifiers: `EVD-024` §§ F7–F9.

## The rotation problem is avoidable — `claude setup-token`

`claude setup-token` mints a **one-year** OAuth token, prints it, and **saves it
nowhere**. Set it as `CLAUDE_CODE_OAUTH_TOKEN` and the process authenticates on a
grant entirely separate from `~/.claude/.credentials.json` — so it never joins
the rotation lineage. No fork, no CAS, no `invalid_grant` self-destruct, and no
~5-day re-login clock for that process.

Everything above about split homes applies to a `/login` grant. It does **not**
apply to a `setup-token` grant. Before designing a broker to fan credentials out
to isolated agents, check whether per-agent `setup-token` grants suffice — they
usually will. The tradeoff is a year-long bearer credential living inside the
agent's environment, which matters when the agent is confined precisely because
it is not trusted.

`CLAUDE_CODE_OAUTH_REFRESH_TOKEN` (with `CLAUDE_CODE_OAUTH_SCOPES`) is the other
documented provisioning path — `claude auth login` exchanges it without a
browser. But it hands over a *refresh* token, so the split-lineage rules apply in
full: only safe when the consumer owns its own grant.

Documented surface, all of it: `CLAUDE_CODE_OAUTH_TOKEN`,
`CLAUDE_CODE_OAUTH_REFRESH_TOKEN`, `CLAUDE_CODE_OAUTH_SCOPES`, `claude
setup-token`. The SDK control-protocol `oauth_token_refresh` is **not**
documented — internal and entrypoint-gated; don't build on it. `EVD-024` §§F10–F13.
