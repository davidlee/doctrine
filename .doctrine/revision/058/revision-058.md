# REV REV-058 — Reconcile PRD-004 with tool-call-keyed ambient surfacing

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-263 (*Ambient memory surfacing for pi and codex*) ships a tool-call-keyed
surfacing mechanism: a harness hook fires on a tool call (Claude / codex
`PreToolUse`; pi `tool_result`), resolves the call to a scope probe, and appends
concise memory *pointers* to that tool's context. PRD-004 §2 out-scopes
*proactive, unsolicited injection of memories into a context ahead of demand*,
and §8 leaves the proactive-vs-demand question open, blocking any pre-emptive
surfacing contract and the trust bar such surfacing would require. Reconcile owes
the reading that makes the shipped mechanism consistent with — not an exception
to — the authored scope.

### The reading

**A tool-call-keyed surface delivers concise pointers at the moment of demand,
not memory payload ahead of demand.**

- **The demand signal is the tool call.** Recall fires because the caller chose
  to act on a path or command, at the moment it acts — never on context entry,
  never on a timer, never unsolicited.
- **The payload is a pointer**, a memory's title and scope rather than its body,
  so the surface offers a reference the model may follow rather than injecting
  recalled knowledge it did not ask for.
- **No new trust bar is owed.** The surface reuses the recall pipeline's existing
  holdback (`REQ-018`'s non-bypassable trust gate; `REQ-151`/`REQ-152`), so a
  suppressed or low-trust memory is withheld exactly as on any other recall.

So §2's out-of-scope bullet still stands in kind — it forbids injection *ahead
of* demand, which this is not — and gains a clarification that a demand-keyed
pointer surface is not the thing it out-scopes. §8's first open question
resolves to the explicit-demand branch.

This is the reading the locked design (`SL-263` §3, §9) carries; approval of the
reading is the user's at reconcile.

### PRD-004 §2 — before/after

**Before** (Out of scope, third bullet):

> Proactive, unsolicited injection of memories into a context ahead of demand.

**After:**

> Proactive, unsolicited injection of memories into a context ahead of demand. A
> *demand-keyed* surface — recall fired by the caller's own act (e.g. a tool
> call) at the moment of that act, delivering pointers rather than memory bodies
> — is not ahead-of-demand injection and is in scope; it is governed by the
> recall requirements like any other recall.

### PRD-004 §8 — before/after

**Before** (first open question, removed as settled):

> Should recall surface memories proactively as a context is entered, or only on
> explicit demand? This blocks the contract for any pre-emptive surfacing and the
> trust bar such surfacing would require.

**After** (replaced by the settled reading, not an open question):

> **Settled:** recall surfaces on demand, never on context entry. The demand
> signal may be the caller's own act — a tool call keyed to the path or command
> it touches — which delivers the nudge at the moment of demand. Pre-emptive
> surfacing on context entry remains out of scope (§2); no separate trust bar is
> owed, because every surfaced pointer passes the existing recall holdback.

The second open question (retention and erasure) is untouched.

### Why not a requirement

§2 and §8 are scope prose and an open question. The mechanism adds no recall
behaviour beyond `REQ-013`–`REQ-016` (record, recall-for-context, lifecycle,
identity) and the trust requirements it reuses, so a `[[change]]` row against a
requirement would misstate a scope reading as a contract change. The row is
`modify PRD-004` (prose, applied at reconcile); no `REQ` is introduced, modified,
or retired.

### Sibling check — PRD-007

PRD-007 §2 out-scopes *per-turn injection of governance content during a session,
and any mechanism that re-pays orientation cost mid-session*. It does **not**
reach this feature: the surface injects memories — a footgun or a scope-relevant
pointer — never governance text, orientation content, or a re-run of the boot
snapshot. No PRD-007 revision is owed; this note records that the conflict the
earlier draft carried was checked and dismissed.
