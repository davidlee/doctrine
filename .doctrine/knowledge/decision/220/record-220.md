# DEC-220: Capsule auth: year-long setup-tokens now, broker later

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Confined agents (`microvm-spike` capsules, and any comparable isolated home) each
get their own year-long OAuth grant from `claude setup-token`, injected as
`CLAUDE_CODE_OAUTH_TOKEN`. A credential broker is deferred to a prototype, not
built now.

## Why

`EVD-024` established that the split-home logout everyone had attributed to
server-side machine fingerprinting is actually refresh-token rotation against a
single-writer credential file, with the *local* client destroying its own
credential on `invalid_grant`. That framing implied a broker: one refresher owning
the lineage, everything else on short-lived tokens.

`EVD-024` § F10 then dissolved the premise. A `setup-token` grant is year-long,
saved nowhere, and never joins the `.credentials.json` lineage — so per-agent
grants avoid rotation, the compare-and-swap, and the ~5-day absolute re-login
clock without any broker at all. What remained was not a mechanism question but a
containment one: a year-long bearer credential inside an agent that is confined
*precisely because it is not trusted*.

Three things decided it:

1. **It is blocking work now.** A broker is an afternoon's prototype competing
   against a capability that is wanted today.
2. **The blast radius is an acceptable trade at this scale.** Single user, few
   capsules, and a credential whose worst case is subscription abuse rather than
   billing exposure or lateral access to anything canonical.
3. **The multi-user case is out of scope by construction.** On a shared
   "agent ranch" serving several users' agents, one person's subscription token
   fanned across other users' workloads is account-sharing on its own terms, and
   the ToS pressure toward API keys resolves the question before the technical
   trade is reached. That is a different deployment, and this decision does not
   extend to it.

## What this accepts

An eight-hour ceiling on guest-side credential exposure was available, via a
broker minting access tokens from a lineage it alone holds. We are taking a
one-year ceiling instead, in exchange for no moving parts.

**Known-unknown: revocation is undocumented.** The published authentication and
env-var docs describe no way to revoke an individual `setup-token`; the only
revocation seen anywhere is the refresh-token revoke on logout. So per-capsule
independent revocation is **unverified and may not exist** — containment after a
compromise may mean re-authenticating the account rather than killing one grant.
At the present scale that is a cheap operation and does not change the decision,
but it is the assumption most worth testing before the capsule count grows, and
the one that would most weaken the reasoning above if the fleet expanded.

## What would reopen this

- Capsule count growing past what account-wide re-auth comfortably covers.
- Any move toward multiple users' agents sharing a host — see reason 3; that
  routes to API keys, not to a broker.
- A documented per-token revocation path appearing, which would strengthen this
  decision rather than reverse it.
- The broker prototype landing and proving cheap enough to adopt on its merits.
