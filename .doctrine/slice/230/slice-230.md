# Memory body-write seam

## Context

Descends from IMP-221, re-scoped against three facts the backlog item predates.

A memory is two tiers: `memory.toml` (structured, edit-preserving) and
`memory.md` (prose body). Every write verb reaches only the first tier. There is
**no supported write path for memory body prose anywhere in the product** —
neither `memory record` nor `memory edit`, on neither the CLI nor the MCP
surface. The only ways to put prose in a body today are hand-editing
`memory.md` (the raw-file write the guardrails tell agents not to do) and the
internal `seed_by_key` (`src/memory.rs:1785`), which already writes a full body
verbatim from a template. The machinery exists; the public verb does not.

This bites agents twice. On record, a freshly-minted memory is a title and a
summary with an empty body — CHR-035 is an open item tracking exactly one such
carcass. On edit, correcting a stale body means bypassing the CLI entirely,
which is how IMP-221 was surfaced in the first place (a stale spec reference in
`mem.signpost.project.orientation` during RFC-012).

*The verify half of this slice was split out.* IMP-221 also asked for relief from
`verify`'s dirty-tree refusal, and this slice originally carried it as objective 3.
At RV-307 round 8 the user split that work into **SL-232** (see **DEC-027**): the
gate proved to be a substantially larger problem than the body-write seam, and the
two halves were not converging at the same rate. What remains here is the seam and
the invalidation that a writable body makes necessary.

Two prior decisions bear on this and neither is recorded in IMP-221:

- **SL-164 rejected an MCP `memory_verify` tool**, reasoning: *"clean-tree
  precondition makes it fragile as a tool anyway."* That rationale is exactly
  what this slice dissolves. The MCP verify question therefore reopens — noted
  here, deliberately not answered (see Non-Goals).
- **`mem.pattern.memory.thread-hidden-until-verified`** (trust: high): a
  `thread` memory is invisible to `find`/`retrieve` until verified. Verify
  friction is not cosmetic for that type — it is the difference between recorded
  and reachable. That memory also warns the SL-008 D6 gate is reviewed canon and
  must not be loosened, which bounds how far the relaxation may go.

## Scope & Objectives

### 1. Body-write on the CLI (`record` and `edit`)

- `memory edit` gains body-replace and body-append, sourced from stdin or a
  file.
- `memory record` gains the same body affordance, so a memory can be born with
  its content instead of requiring a follow-up edit.
- Both ride the existing `seed_by_key` / `memory_scaffold` fileset seam. No
  second body-write path — the transactional write already exists and must be
  reused, not paralleled.
- `updated` stamping and the edit-preserving `memory.toml` contract behave
  exactly as they do for metadata edits.

### 2. Body-write on the MCP surface

- `memory_edit` gains `body` + `body_mode` (`replace` | `append`). Metadata-only
  edit stays the default path — backward compatible.
- `memory_record` gains the matching `body` field.
- Both delegate to the same core as the CLI verbs; the MCP layer stays a thin
  argument adapter.

### 3–4. Corpus-aware verify gate + SPEC-007 reconciliation — MOVED to SL-232

Both objectives left this slice by **DEC-027** at RV-307 round 8.

The gate (claim-aware exclusion, the claim-surface constructor, non-contribution
reporting under DEC-020, and the historical-consumer question) is **SL-232**'s,
along with **REV-034** — the SPEC-007 + REQ-147 amendment — because the retired
"clean working tree" contract is changed by the gate and not by body-write. The
`needs` edge moved with it, so this slice is no longer gated on that revision.

Read `.doctrine/slice/232/design.md` for the inherited design; it carries two open
blockers (RV-307 F-36, F-37) and is not locked.

### 5. Attestation invalidation

Added during design — see "Discovered during design" below.

Editing a **claim field** through `edit` clears the verification axis
(`verification_state`/`reviewed`/`verified_sha`), iff the content genuinely
changed. Claim fields are `body`, `title`, `summary` and `scope.*` — what the
memory asserts and what it asserts against (design D8, widened from body-only by
RV-307 F-8). Record fields — `status`, `lifespan`, `review_by`, `trust`,
`severity` — do not clear; they are judgements *about* the record.

`validate`'s staleness check additionally counts commits touching the memory's
**own item directory** since `verified_sha`, catching the hand-edit bypass and
other agents. It does **not** catch masters: they are unanchored and `collect_all`
never scans them (RV-307 F-7; design R5). Advisory surface only; retrieve-side
ranking is deliberately untouched (design OQ-2).

### 6. Verify refusal names its escape hatch — MOVED to SL-232

The dirty-tree refusal message is part of the gate; it goes with it (DEC-027).

## Non-Goals

- **An MCP `memory_verify` tool.** SL-164's stated reason for excluding it is
  dissolved by SL-232's gate, not by this slice; re-litigating the exclusion is
  its own decision. Captured as a follow-up, not smuggled in.
- **`--edit-body` / `$EDITOR` interactive body editing.** Floated in IMP-221 as
  a nice-to-have; an interactive editor is unusable from a jailed or MCP agent
  context, which is the whole audience. Dropped, not deferred.
- **Loosening `thread_expiry`** or any other retrieval-side gate. Reviewed canon
  (SL-008 D6).
- **The shipped-corpus authoring pipeline** (`cargo build` re-embed →
  `memory sync` → `doctrine install`). Untouched.
- **Filling CHR-035's empty body.** This slice supplies the verb; using it is
  that item's job.
- **Any change to the memory TOML/MD schema.** (SL-232 *does* expect one — the
  declared boundary and attested-coverage field. That is a reason it is a separate
  slice, not a reason to pre-empt it here.)
- **The corpus-aware `verify` gate** and everything downstream of it — claim
  surfaces, exclusion sets, pathspec construction, non-contribution reporting.
  **SL-232**'s by DEC-027. Where this slice needs a git fact the gate also needs,
  it states it locally rather than depending on the gate's rules.

## Risks, assumptions, open questions

**Status: design NOT locked.** It was locked-pending-review until RV-307 round 8;
DEC-027 then narrowed the slice, so `design.md` has been rewritten to the new
boundary and needs a confirming pass over the retained half. Live open questions
are in `design.md` § 6 (OQ-4, OQ-7).

- **A1 — RESOLVED ✓.** SL-005 § 5.2 (review #7) reads: "v1 scaffolds a template
  containing title + summary only — no editor, no stdin, no `--body`. *Richer
  body capture is a later mutation verb.*" A deferral, not a prohibition — this
  slice is that verb. No governance step needed before objectives 1-2.
- **OQ-2 — RESOLVED.** Replace + append, via `--body-mode`. Revisit on evidence.
- **OQ-1, OQ-3, R1 — MOVED to SL-232** (DEC-027). All three were scoping-stage
  questions about the exclusion set and `capture_with`; they were resolved, and
  their resolutions travel with the gate rather than being restated here.
- **R2 — CARRIED, sharpened.** Stored memory text is untrusted (SPEC-007
  § Concerns). Review finding A3 corrected the framing: there is no write-time
  escaping on the `.md` tier to bypass; the defence is read-time (nonce +
  data-framing), untouched here. Pinned by T16.

### Discovered during design — then narrowed by the split

- **Attestation survives claim change (new objective 5).** Observed live: a
  memory verified at `933b747c` kept `verification_state = "verified"` through a
  committed body edit. `apply_edit` touches no verification field. Body-write
  turns a hand-edit-only footgun into a one-command operation, so this slice
  closes it: the verb clears the axis (D4), and `validate` gains an
  own-directory staleness check to catch the hand-edit bypass (D5).
- **The dirty refusal hides its own escape hatch (was objective 6).** The message
  prescribes committing and never mentions `--allow-dirty` — which is why, in
  practice, agents reach for `git stash` instead. Still true, now **SL-232**'s to
  fix (DEC-027).
- **Scope grew from 4 objectives to 6 during design, then split.** The growth was
  real and the review found it load-bearing; the split is the correction, not a
  retreat. DEC-027 records why.

## Verification / closure intent

- Body written via `record` and via `edit` (both modes, both surfaces) round-
  trips through `memory show` byte-for-byte.
- Editing a claim field through `edit` clears the verification axis; editing a
  record field does not; re-setting a field to its existing value clears nothing.
- `validate` flags staleness after a **hand-edit** of `memory.md` (the bypass
  path — the verb path clears the stamp, so it never reaches the check).
- Existing memory and entity suites stay green unchanged (behaviour-preservation
  gate on shared machinery).
- A rejected `edit` argument leaves **both** tiers untouched.

*Gate-side closure intent — dirty-tree behaviour, exclusion constants, and
SPEC-007 agreement — moved to SL-232 (DEC-027).*

## Summary

## Follow-Ups

- Reopen SL-164's MCP `memory_verify` exclusion once SL-232 lands — its stated
  rationale ("clean-tree precondition makes it fragile as a tool anyway") is
  dissolved by the gate, not by this slice, so the follow-up belongs after SL-232.
- Correct IMP-221's body: sub-item C's premise is stale (`--allow-dirty` predates
  the filing).
