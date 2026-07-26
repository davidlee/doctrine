# Memory body-write verbs and corpus-aware verify gate

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

The verify half of IMP-221 was filed on a stale premise. It asserts that
`memory verify` refuses a dirty working tree, full stop; `--allow-dirty` had
already shipped 2026-06-18/21, roughly ten days before the item was written.
But the friction is real and still recurring — the flag is undiscoverable, and
in practice agents hit the refusal and reach for `git stash` rather than for
`--help`. Worse, the common case is self-inflicted: doing doctrine work means
the authored corpus is essentially always dirty, so verifying a memory you just
edited is blocked *by your own edit*.

The fix is to make the cleanliness test measure the thing the attestation is
actually about. A memory attests a claim against the **code**; a dirty
governance corpus says nothing about whether that claim still holds. So the
dirty check should ignore doctrine's own authored trees by default, and keep
refusing on a genuinely dirty source tree — where the attestation truly cannot
be pinned to a commit.

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

### 3. Corpus-aware verify dirty-tree gate

- The clean/dirty decision for `verify` ignores modifications confined to
  doctrine's **own authored trees** — `.doctrine/**` and the repo-root `memory/`
  corpus-authoring tree that `record --global` writes into (SL-018).
- The exclusion set is computed from **doctrine-owned path constants**, not a
  hardcoded literal list. POL-002 forbids load-bearing on host layout; STD-001
  forbids the magic strings. These are owned platform paths — the engine already
  defines them — so the exclusion is grounded on an owned contract, which is
  precisely what makes it legal.
- **The exclusion is claim-aware, not blanket** (revised by RV-307 F-1/F-6). The
  memory's own item directory and its declared scopes are *never* excluded — they
  are the evidence the attestation is about. Blanket exclusion would have stamped
  a commit that did not contain the attested body, and would have ignored a
  modified file a memory explicitly scopes. So `verify` asks two questions: is the
  code dirty, and is the claim committed?
- **Consequence, stated plainly:** `record` → `verify` now **refuses** until the
  new memory is committed. The friction this slice removes is *unrelated* corpus
  dirt — another agent's uncommitted backlog file, your own unrelated spec edit —
  not the requirement that the claim you are attesting exist in a commit.
- A dirty **source** tree still refuses, and `--allow-dirty` remains the escape
  hatch for that case with its `checkout_state_id` stamping unchanged.
- The `thread_expiry` gate (SL-008 D6) is not touched.

### 4. SPEC-007 reconciliation — REV-034

The retired "clean working tree, refusing a dirty one" contract lives at **three**
sites, not two: `spec-007.toml:22`, `spec-007.md:132-133`, and **REQ-147**, an
active member of SPEC-007 whose *title is that contract verbatim* (found by
RV-307 F-5; a two-site amendment would have left the requirement asserting the
opposite of the code). The implementation already diverged when `--allow-dirty`
shipped; this slice changes it further. Corrected through **REV-034**
(`SL-230 needs REV-034`), applied at close so spec and code turn over together.

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

### 6. Verify refusal names its escape hatch

The dirty-tree refusal names `--allow-dirty` instead of prescribing a commit.

## Non-Goals

- **An MCP `memory_verify` tool.** SL-164's stated reason for excluding it
  dissolves here, but re-litigating that exclusion is its own decision with its
  own scope. Captured as a follow-up, not smuggled in.
- **`--edit-body` / `$EDITOR` interactive body editing.** Floated in IMP-221 as
  a nice-to-have; an interactive editor is unusable from a jailed or MCP agent
  context, which is the whole audience. Dropped, not deferred.
- **Loosening `thread_expiry`** or any other retrieval-side gate. Reviewed canon
  (SL-008 D6).
- **The shipped-corpus authoring pipeline** (`cargo build` re-embed →
  `memory sync` → `doctrine install`). Untouched.
- **Filling CHR-035's empty body.** This slice supplies the verb; using it is
  that item's job.
- **Any change to the memory TOML/MD schema.**

## Risks, assumptions, open questions

**Status: design locked-pending-review (`design.md`). All scoping-stage unknowns
below are resolved; live open questions now live in `design.md` § 6.**

- **A1 — RESOLVED ✓.** SL-005 § 5.2 (review #7) reads: "v1 scaffolds a template
  containing title + summary only — no editor, no stdin, no `--body`. *Richer
  body capture is a later mutation verb.*" A deferral, not a prohibition — this
  slice is that verb. No governance step needed before objectives 1-2.
- **OQ-1 — RESOLVED.** Narrowed to a single path: `MEMORY_SHIPPED_DIR` and
  `MEMORY_ITEMS_DIR` are both *under* `.doctrine`, so one exclusion root covers
  them; only repo-root `MEMORY_MASTERS_DIR` sits outside. Excluded, guarded on
  the directory's existence — POL-002-legal (an owned constant) without carrying
  dead weight into client projects that have no such tree.
- **OQ-3 — RESOLVED ✓.** Exclusion applies at verify only. `capture()` has
  exactly three callers (verified: `retrieve.rs:532`, `memory.rs:1708`,
  `memory.rs:3382`); two would be damaged by unconditional leniency. Implemented
  as `capture_with(root, excludes)` with `capture()` delegating — see design D3
  and review finding A1 (the first draft's parallel probe was rejected).
- **OQ-2 — RESOLVED.** Replace + append, via `--body-mode`. Revisit on evidence.
- **R1 — RESOLVED.** Corpus-dirty stamps the **HEAD commit**, which is *stronger*
  evidence than today's `checkout_state_id` hash, not weaker. See design D3.
- **R2 — CARRIED, sharpened.** Stored memory text is untrusted (SPEC-007
  § Concerns). Review finding A3 corrected the framing: there is no write-time
  escaping on the `.md` tier to bypass; the defence is read-time (nonce +
  data-framing), untouched here. Pinned by T16.

### Discovered during design — scope grew from 4 objectives to 6

- **Attestation survives claim change (new objective 5).** Observed live: a
  memory verified at `933b747c` kept `verification_state = "verified"` through a
  committed body edit. `apply_edit` touches no verification field. Body-write
  turns a hand-edit-only footgun into a one-command operation, so this slice
  closes it: the verb clears the axis (D4), and `validate` gains an
  own-directory staleness check to catch the hand-edit bypass (D5).
- **The dirty refusal hides its own escape hatch (new objective 6).** The message
  prescribes committing and never mentions `--allow-dirty` — which is why, in
  practice, agents reach for `git stash` instead. The refusal now names the flag.

## Verification / closure intent

- Body written via `record` and via `edit` (both modes, both surfaces) round-
  trips through `memory show` byte-for-byte.
- Verify succeeds with only `.doctrine/**` dirty; still refuses with a dirty
  source tree; `--allow-dirty` behaviour unchanged.
- Exclusion set derives from named constants — no path literals at the call
  site.
- Existing memory suites stay green unchanged (behaviour-preservation gate on
  shared machinery).
- SPEC-007 text and implementation agree.

## Summary

## Follow-Ups

- Reopen SL-164's MCP `memory_verify` exclusion now that its stated rationale
  no longer holds.
- Correct IMP-221's body: sub-item C's premise is stale (`--allow-dirty` predates
  the filing).
