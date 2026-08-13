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
