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
and §8 asks whether recall may happen before an explicit memory request. The
tool action is deliberate, but it is **not** an explicit request for memory.
The spec needs to name this context-triggered pointer case without treating it
as either a context-entry push or an explicit memory query.

### The reading

**A tool-call-keyed surface delivers concise pointers when the agent acts on a
path or command, without requiring an explicit memory query.**

- **The trigger is a work action.** The hook asks the memory engine about the
  path or command the agent chose to use. This is automatic surfacing at that
  action, not an agent request to recall memory. It does not fire merely because
  a context was entered or time passed.
- **The payload is a pointer**, carrying a memory's title, uid and a short
  triage label rather than its body, so the surface offers a reference the model
  may follow.
- **The existing suppression gate still applies.** The surface obtains rows
  through the memory engine's retrieval path: quarantined and retracted
  memories are suppressed (`REQ-017` / `REQ-151`), and low-trust,
  high-severity memories pass through the non-bypassable holdback (`REQ-152`).
  `REQ-018` governs the rendering of recalled knowledge as data; it is not
  itself a trust gate. A pointer's title is recalled, untrusted knowledge, so
  the pointer must also meet `REQ-018`'s quoted, attributed presentation rule.
  The shipped formatter's discrepancy is tracked by `ISS-480`.

So §2's out-of-scope bullet retains the context-entry boundary and gains an
explicit in-scope case for action-triggered pointers. §8's first open question
settles with three distinct cases: explicit memory queries, optional
action-triggered pointers, and out-of-scope context-entry injection.

This is the reading the locked design (`SL-263` §3, §9) carries; approval of the
reading is the user's at reconcile.

### PRD-004 §2 — before/after

**Before** (Out of scope, third bullet):

> Proactive, unsolicited injection of memories into a context ahead of demand.

**After:**

> Proactive injection of memory bodies or pointers merely because a context is
> entered or a timer fires, before any working action or explicit recall request.
> An optional surface triggered by the agent's deliberate action on a path or
> command may provide concise memory pointers at that action. This is in scope
> even when the agent did not explicitly request memory.

### PRD-004 §6 and §8 — before/after

**Before** (first open question, removed as settled):

> Should recall surface memories proactively as a context is entered, or only on
> explicit demand? This blocks the contract for any pre-emptive surfacing and the
> trust bar such surfacing would require.

**After** (remove the settled question from §8; add to §6 Behaviour):

> Optional pointer flow — when an enabled harness observes a deliberate tool
> action on a path or command, it may ask for relevant memory pointers at that
> action. The system applies the agent-facing recall exclusions before returning
> concise pointers rendered as quoted, attributed data bearing identity, trust
> standing, and context; the agent may follow a pointer to request the full
> memory. This flow does not run merely on context entry. The pointer title is
> untrusted data and never becomes an instruction.

The second open question (retention and erasure) is untouched.

### Why not a requirement

`REQ-014` already lets a caller request memories relevant to a working context;
the enabled harness is a caller using that contract at a work action. This
revision clarifies when that optional caller may act. It does not require every
harness to install a pointer surface or change the full-memory recall contract.
`REQ-017` / `REQ-151` / `REQ-152` still govern admission, and `REQ-018` still
requires recalled knowledge to be presented as data. The row is `modify PRD-004`
(prose, applied at reconcile); no `REQ` is introduced, modified, or retired.

### Sibling check — PRD-007

PRD-007 §2 out-scopes *per-turn injection of governance content during a session,
and any mechanism that re-pays orientation cost mid-session*. It does **not**
reach this feature: the surface injects memories — a footgun or a scope-relevant
pointer — never governance text, orientation content, or a re-run of the boot
snapshot. No PRD-007 revision is owed; this note records that the conflict the
earlier draft carried was checked and dismissed.

## Reconcile narrative (SL-263)

- [RV-381 F-5]: PRD-004 did not describe the shipped tool-call-keyed pointer surface. Approved by the user at reconcile ("agreed", 2026-09-24). Landed by hand: §2 out-of-scope bullet narrowed to context-entry/timer injection with the action-triggered pointer case in scope; §6 gains the optional pointer flow; §8's settled first open question removed.
- [RV-381 F-6]: this revision states the REQ-018 pointer-rendering contract; it does not certify the shipped formatter. ISS-480 stays open.
