# ISS-485: No review amend or reopen: a stale response or out-of-band verify is unrecoverable

Contest/verify reasoning has no durable field (`ISS-280`), and there is no verb to
correct a finding whose durable state was moved out of band:

- **stale response** — the design moved out-of-band; `review dispose` refuses
  (`current status answered != required open`), so the ledger carries a knowingly
  stale response (obs `019faae6`).
- **out-of-band verify** — a responder set `status=verified` (the raiser's act);
  the raiser then cannot contest, and nothing reopens a verified finding
  (obs `01a0d7f7`).

Fix: an amend/reopen path for the raiser, and/or a write-time refusal of a
responder-authored `verified`. Complements `ISS-280`. See RFC-032 `research.md` F2.
