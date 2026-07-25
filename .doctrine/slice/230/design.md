# Design SL-230: Memory body-write verbs and corpus-aware verify gate

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

A memory is two tiers — `memory.toml` (structured, edit-preserving) and
`memory.md` (prose body). **Every write verb in the product reaches only the
first tier.** There is no supported path to author or amend memory prose on the
CLI or over MCP; the only options are hand-editing the `.md` (the raw-file write
the guardrails forbid) or the internal `seed_by_key`. A memory is therefore born
as a title and a summary with an empty body, and correcting a stale body means
leaving the tooling entirely.

Compounding it, `verify` refuses to attest against a dirty working tree. Doing
doctrine work means the authored corpus is almost always dirty, so the common
case is self-inflicted: you cannot attest a memory because of the corpus edit you
just made — or, as observed live during this design round, because of an
*unrelated* backlog file another agent left uncommitted.

This slice closes both, and in doing so exposes a third defect it must also
close: nothing invalidates an attestation when the claim it attests to changes.

## 2. Current State

| Surface | Behaviour | Site |
|---|---|---|
| `memory record` | scaffolds `memory.md` from a template — title + summary only | `render_memory_md` `src/memory.rs:1577` |
| `memory edit` | parses `memory.toml` into `toml_edit::DocumentMut`, writes via `write_atomic`; **never opens the `.md`** | `run_edit` `src/memory.rs:3991` |
| MCP `memory_record` / `memory_edit` | metadata fields only; no `body` | `src/mcp_server/tools.rs:310`, `:902-1010` |
| `memory verify` | refuses on any dirty tree unless `--allow-dirty` | `src/memory.rs:3382-3390` |
| Verification axis | `[review].verification_state`, `[review].reviewed`, `[git].verified_sha` — written **only** by `stamp_verification` | `src/memory.rs:3350-3362` |
| `memory validate` | staleness = commits touching **scoped paths** since `verified_sha` | `src/memory.rs:3424` |

Two facts establish that no seam is being duplicated:

- **No verb of any entity kind writes a prose tier from user input.** Twelve
  checked (`spec edit` — descent scalars only, `src/spec.rs:348`; `backlog edit`
  — status/resolution, `src/backlog.rs:1873`; `adr` has no edit verb; the rest
  scaffold from templates). `seed_by_key` (`:1785`) is the sole body-write seam
  and is internal to install seeding.
- **`capture()` has exactly three callers** — `src/retrieve.rs:532` (read path),
  `src/memory.rs:1708` (`record`), `src/memory.rs:3382` (`verify`).

### 2.1 Observed defect — attestation survives claim change

Demonstrated during this design round, not inferred. A memory was verified
(`verified_sha = 933b747c`, `verification_state = "verified"`,
`reviewed = 2026-07-25`), then its body was edited and the edit **committed**.
The verification axis was untouched: it still reads `verified`.

`apply_edit` manages title, summary, status, lifespan, review_by, trust,
severity, key and scopes — and no verification field. So the stamp outlives the
claim it attests to. SPEC-007 § Concerns names this exact hazard ("over-trust of
stale or poisoned memory"), and the retrieval sort ranks verification *above*
lexical score, so a false stamp reaches agent context with extra weight.

Body-write does not create this defect, but it turns a hand-edit-only footgun
into a first-class, one-command operation. It must be closed here.

## 3. Forces & Constraints

| Authority | Constraint |
|---|---|
| **SL-005 § 5.2 (review #7)** ✓ | "v1 scaffolds a template containing title + summary only — no editor, no stdin, no `--body`. **Richer body capture is a later mutation verb.**" A deferral, not a prohibition — this slice is that verb. No governance step needed. |
| **SPEC-007** | Asserts verify attests "against a clean working tree, refusing a dirty one" (`spec-007.toml:22`, `spec-007.md:132`). Already false since `--allow-dirty`; this slice widens it. Amend via REV. |
| **ADR-013** | Governance→work dependency routes through a Revision — the SPEC-007 fix is a `REV-NNN`, not a raw edit. |
| **ADR-001** ✓ | `corpus_guard` = leaf, `git` = leaf, `entity` = engine ("the hub — kind-agnostic directory-entity scaffold", imports `fsutil` only), `memory` = command. Downward edges only. |
| **POL-002** | The exclusion set must rest on doctrine-owned contracts, never host layout. `.doctrine` and `MEMORY_MASTERS_DIR` are platform-owned constants, so exclusion is legal — but `memory/` exists only in this repo, so guard it on existence rather than assume it. |
| **STD-001** | Named constants, not path literals. Satisfied by reuse: `DOCTRINE_PATHSPEC` already exists (`src/corpus_guard.rs:43`). |
| **SL-008 D6** | `thread_expiry` is reviewed canon — not loosened. This slice only feeds it honest input. |

## 4. Guiding Principles

- **The frame tells the truth.** `capture()` reports the literal state of the
  tree. Leniency is a *policy* applied by one consumer, never baked into the
  measurement.
- **Attestation is about the claim.** A memory attests that its body is true of
  the code. Dirt in the governance corpus says nothing about that; a change to
  the body says everything.
- **Strictness must be affordable.** Mandatory re-verification is only tolerable
  because the gate relaxation makes verifying cheap. The two halves pay for each
  other.
- **Build the seam once.** This is the product's first prose-write path; its
  shape is inherited by every kind that follows.

## 5. Proposed Design

### 5.1 System Model

```
command tier   memory.rs ──────────────┬──────────── run_record / run_edit / run_verify
                                       │
engine tier    entity.rs  write_body(dir, file, text, mode) -> Result<bool>
                                       │
leaf tier      fsutil.rs  write_atomic          corpus_guard.rs  is_excluded(path, roots)
               git.rs     capture()  +  source_clean(root, excludes) -> Option<Frame>
```

Four changes, each at its correct altitude: a kind-agnostic body writer at
engine, a pure exclusion predicate and a new git probe at leaf, and policy
composition at command.

### 5.2 Interfaces & Contracts

**Body seam (engine, `src/entity.rs`)** — the reusable unit.

```rust
pub(crate) enum BodyMode { Replace, Append }

/// Write an entity's prose tier. `dir` is the entity's item directory,
/// `file` its body filename (`memory.md`, `spec-007.md`, …).
/// Append inserts a single blank-line separator iff the existing body is
/// non-empty and does not already end in one.
/// Returns false when the write was a no-op (Replace with identical content).
pub(crate) fn write_body(
    dir: &Path, file: &str, text: &str, mode: BodyMode,
) -> Result<bool>
```

Takes `dir` + `file` rather than a path so kind layout stays with the caller.
Returns a changed-flag mirroring `apply_edit`, preserving the no-op guard
(content + mtime hold) from `mem.pattern.entity.edit-preserving-status-transition`.
Owns the append separator so no kind re-derives it.

**CLI (`memory record` / `memory edit`)**

```
--body <TEXT|->        prose body; `-` reads stdin
--body-mode <MODE>     replace | append  (default: replace; `edit` only)
```

One input flag with `-` as the stdin sentinel — a separate `--body-file` is
redundant once stdin works, and heredoc-into-stdin is the only quoting-safe way
for an agent to pass multi-paragraph markdown. The pair maps 1:1 onto MCP's
`body` / `body_mode`, so the model is identical across arms. A literal body of
`-` is documented as unreachable (use stdin).

`--body-mode` on `record` is **rejected**, not ignored — a new memory has no
body to append to.

**MCP** — `memory_record` gains `body`; `memory_edit` gains `body` +
`body_mode`. Metadata-only edit stays the default path. Tool count is unchanged
(no new tools), so the `25` assertions at `tools.rs:1488` / `:1870` do not move.

**Source-cleanliness probe (leaf, `src/git.rs`)** — new, additive.

```rust
/// Is the tree clean once `excludes` are ignored? Returns a commit-anchored
/// Frame when so, `None` when genuinely dirty. Never mutates; `capture()` and
/// its three callers are untouched.
pub(crate) fn source_clean(root: &Path, excludes: &[&str]) -> Result<Option<Frame>>
```

Required because `capture()` **blanks the commit oid whenever the tree is dirty**
(`git.rs`: `commit: String::new(), // empty iff dirty`) — it yields only a
`checkout_state_id` hash, and you cannot subtract corpus paths from a sha256. The
probe re-runs the same three measurements with exclusion pathspecs:

| Probe | `capture()` | `source_clean()` |
|---|---|---|
| worktree | `git diff HEAD --binary …` | same `+ :(exclude)<root>` per exclude |
| untracked | `untracked_fingerprint` | same, exclusions applied |
| index | `git diff-index --quiet --cached HEAD` | same `+ :(exclude)…` |

Untracked coverage is not optional: a freshly-recorded memory's directory is
untracked, so `record` → `verify` is otherwise still blocked by the memory just
recorded.

**Exclusion predicate (leaf, `src/corpus_guard.rs`)** — pure, roots passed in as
data (a leaf module cannot import `MEMORY_MASTERS_DIR` from command tier; this
also honours the pure/imperative split). Roots supplied by the command-tier
caller: `DOCTRINE_PATHSPEC` always, `MEMORY_MASTERS_DIR` only when that directory
exists.

### 5.3 Data, State & Ownership

No schema change. No new persisted field. The verification axis keeps its three
existing fields; `write_body` owns no state; `source_clean` returns a value.

`MEMORY_SHIPPED_DIR` (`.doctrine/memory/shipped`) and `MEMORY_ITEMS_DIR`
(`.doctrine/memory/items`) are both *under* `.doctrine`, so one exclusion root
covers them. Only `MEMORY_MASTERS_DIR` (`memory`, repo-root) sits outside.

### 5.4 Lifecycle, Operations & Dynamics

**Body on `record`** rides the existing scaffold seam, not `write_body`: `Draft`
and `RecordArgs` gain `body: Option<&str>`; `memory_scaffold` substitutes it for
`render_memory_md`'s output — exactly `seed_by_key`'s existing move
(`fileset.get_mut(1)` → `body.clone_into(b)`, `:1808-1810`). The transactional
`materialise_named` write is preserved unchanged.

**Body on `edit`** calls `write_body` after the TOML write. `EditFields` gains
`body` + `body_mode`; `has_any()` counts body, so `--body` alone is a valid edit,
and a body-only edit stamps `updated` (the memory did change).

**Write ordering.** `run_edit` now writes two files and is no longer atomic.
Order is **body first, then TOML**, so `updated` lands last and never claims a
change that did not. Full two-tier atomicity would mean routing `edit` through
the fileset/rollback machinery — a large refactor of a shared write path,
disproportionate to a crash window this narrow. Stated, not buried (R1).

**Verify.** Gate becomes: probe `source_clean`; if it yields a Frame, attest
against that HEAD commit; else refuse unless `--allow-dirty`. Corpus-dirty now
produces a *stronger* anchor than today's `checkout_state_id` — a real,
addressable commit.

**Verification invalidation.** A body write through `edit` clears the axis:
`verification_state` → `unverified`, `reviewed` and `verified_sha` → empty. Not a
new state; the scaffold default. Append clears too — appended prose is unverified
claim.

**Staleness (the hand-edit path).** Clearing on the verb alone would invert the
guardrail: the sanctioned path would be stricter than hand-editing, rewarding
bypass — and masters are hand-edit-only by design. So `validate`'s staleness
check additionally counts commits touching **the memory's own item directory**
since `verified_sha`, alongside the existing scoped-paths count. Edit-path
agnostic: catches the verb, hand-edits, masters, and other agents.

Verified against live data: `git rev-list --count <verified_sha>..HEAD -- <dir>`
returned 3 for the memory whose stamp survived a committed body edit — the same
plumbing the scoped-paths check already uses.

**Refusal legibility.** The current message — "working tree is dirty: refusing to
verify … Commit first, then verify." — never mentions `--allow-dirty`. At the one
moment an agent is looking for the escape hatch, the tool hides it and prescribes
stashing. The refusal names its own flag.

### 5.5 Invariants, Assumptions & Edge Cases

- **I1** — `capture()`'s behaviour and its three call sites are byte-for-byte
  unchanged. `record`'s born anchor and the retrieve read path must not move.
- **I2** — `source_clean` never mutates; no `write-tree`, no index lock on the
  clean path.
- **I3** — a genuinely dirty *source* tree still refuses without `--allow-dirty`.
- **I4** — `--allow-dirty` semantics unchanged (attest anyway, stamp
  `checkout_state_id`).
- **I5** — `thread_expiry` untouched.
- **E1** — thread memories vanish from `find`/`retrieve` after a body edit until
  re-verified (SL-008 D6 feeding on honest input). Correct but surprising —
  the verb says so on stderr.
- **E2** — repo-empty masters carry `anchor_kind = None`, never `CheckoutState`;
  the dirty gate never fires for them and nothing changes.
- **E3** — body content `-` is unreachable inline; use stdin.
- **E4** — `memory/` absent (every client project) → that exclusion root is
  simply not contributed.

## 6. Open Questions & Unknowns

- **OQ-1** — should `--summary` / `--title` also clear verification? They are
  claim-bearing and today do not. Deferred: changing them alters existing verb
  behaviour and puts the behaviour-preservation gate on shared machinery in play
  for no gain to the motivating case. Lean: yes, in a follow-up owning the
  regression surface.
- **OQ-2** — should own-directory drift feed *retrieve-side* `staleness`, not
  just `validate`? Deferred deliberately: it would reclassify a large fraction of
  the corpus at once and shift retrieval ordering broadly (D5).
- **OQ-3** — a body digest stamped at verify time would make invalidation
  git-independent and path-independent, covering uncommitted edits and masters
  (which have no `verified_sha`). Needs a new persisted field → schema change →
  its own slice.
- **OQ-4** — when do other kinds adopt `write_body`? This slice wires memory only.

## 7. Decisions, Rationale & Alternatives

- **D1 — reusable body seam at engine tier.** `entity.rs`, kind-agnostic.
  *Alternative:* memory-local helper, lift later. *Rejected:* this is the
  product's first prose-write path; whatever it looks like is inherited. Building
  it reusable now costs little because `entity.rs` is already the kind-agnostic
  hub and imports only `fsutil`.
- **D2 — `--body` with `-` sentinel + `--body-mode`.** *Alternatives:* separate
  `--replace-body`/`--append-body` (IMP-221's original — reads better but
  diverges from MCP's single `body_mode` and scales badly to a third mode);
  `--body` + explicit `--body-file` (a flag stdin already covers).
- **D3 — new `source_clean` probe; `capture()` untouched.** *Alternative:* apply
  the exclusion inside `capture()`. *Rejected:* two of its three callers would be
  damaged — `record` would stamp a false born anchor and the retrieve read path
  would shift. *Alternative:* soften the refusal and keep stamping
  `checkout_state_id`. *Rejected:* weaker evidence for the common case, and it
  makes the default and `--allow-dirty` near-identical.
- **D4 — body edit clears the verification axis.** Affordable only because the
  gate relaxation makes re-verifying cheap; the halves pay for each other.
  *Alternative:* leave it (status quo) — rejected, the stamp would lie by
  one command. *Alternative:* a third "stale" state — rejected, new vocabulary
  for no gain over `unverified`.
- **D5 — own-directory staleness in `validate` only.** Closes the hand-edit
  inversion without re-ranking the corpus as a side effect of a body-write slice.
- **D6 — items-only; masters out of scope.** The motivating memory
  (`mem.signpost.project.orientation`) is an item ✓, so the case is covered.
  Extending `resolve_memory_toml_path` would change *every* memory write verb.
  Mitigated by `mem.system.memory.global-master-authoring` — now glob-scoped
  `memory/**` at severity high, so it fires on any masters edit.
- **D7 — `--edit-body` / `$EDITOR` dropped, not deferred.** Unusable from a
  jailed or MCP agent, which is the audience.

## 8. Risks & Mitigations

- **R1 — `edit` is no longer two-tier atomic.** Body-then-TOML ordering means a
  crash leaves a changed body with a stale `updated`, never the reverse.
  Accepted; full atomicity is a shared-write-path refactor.
- **R2 — hostile-input substrate.** SPEC-007 treats stored memory text as
  untrusted, and SL-005 justified thin bodies with "`show` therefore renders
  bounded, tool-authored prose" — rich bodies make "bounded" false. The
  body-guard nonce and data-framing (`src/memory.rs:2021-2045`) are unchanged and
  still apply; the new path must not bypass the scaffold's escaping. No size cap
  is imposed: anyone who can run the verb can already write the file, so a cap
  would be theatre rather than a boundary.
- **R3 — behaviour preservation on shared machinery.** `entity.rs` and
  `validate` are shared. Existing suites must stay green unchanged.
- **R4 — mass re-verification.** D4 means every body edit costs a verify.
  Mitigated by the gate relaxation; if it still bites, that is evidence for OQ-3.

## 9. Quality Engineering & Validation

Model test: `memory_verify_allow_dirty_stamps_checkout_state_id` (`:9123`);
fixture: `GitScratch` (`:5617`); MCP e2e: `tests/e2e_mcp_server.rs:963-1110`.

| # | Test | Asserts |
|---|---|---|
| T1 | body on `record`, CLI + MCP | round-trips through `show` byte-for-byte |
| T2 | `--body` replace / append, CLI + MCP | append inserts exactly one blank-line separator; replace is exact |
| T3 | `--body -` reads stdin | multi-paragraph markdown survives unaltered |
| T4 | `--body-mode` on `record` | rejected, not ignored |
| T5 | body-only edit | `has_any()` true; `updated` stamped |
| T6 | replace with identical content | no-op: content + mtime hold |
| T7 | verify, only `.doctrine/**` dirty | succeeds, stamps **HEAD commit** (not `checkout_state_id`) |
| T8 | verify, untracked memory dir only | succeeds — the `record` → `verify` case |
| T9 | verify, source tree dirty | still refuses; message names `--allow-dirty` |
| T10 | `--allow-dirty` | unchanged, stamps `checkout_state_id` |
| T11 | `capture()` behaviour | unchanged — I1 regression guard |
| T12 | body edit | clears `verification_state`/`reviewed`/`verified_sha` |
| T13 | `validate` after committed body edit | flags stale via own-directory drift |
| T14 | `memory/` absent | exclusion root not contributed; no error |
| T15 | existing memory + entity suites | green unchanged (R3) |

Closure: T1-T15 green; `doctrine check gate` clean; SPEC-007 REV applied so text
and implementation agree.

## 10. Review Notes
