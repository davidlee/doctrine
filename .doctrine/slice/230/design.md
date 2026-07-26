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
| **SPEC-007** | Asserts verify attests "against a clean working tree, refusing a dirty one" — **three** sites, not two: `spec-007.toml:22`, `spec-007.md:132-133`, and **`REQ-147`**, whose *title is the retired contract verbatim* and which is an active member of SPEC-007 (`doctrine inspect REQ-147` → `members: SPEC-007`). Already false since `--allow-dirty`; this slice changes it further. Amended by **REV-034**. |
| **ADR-013** ✓ | Governance→work dependency routes through a Revision. **REV-034** is minted and `SL-230 needs REV-034` is authored — the dependency is instantiated, not promised. |
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
                                       │             composes the pathspec sets (policy)
engine tier    entity.rs  write_body(dir, file, text, mode) -> Result<bool>
                                       │
leaf tier      fsutil.rs  write_atomic
               corpus_guard.rs  DOCTRINE_PATHSPEC  (existing constant, STD-001)
               git.rs     dirty_under(root, pathspecs) -> Result<bool>     ← the primitive
                          capture_with(root, excludes) -> Result<Frame>    ← delegates to it
                          capture(root) = capture_with(root, &[])          ← unchanged behaviour
```

Three changes, each at its correct altitude: a kind-agnostic body writer at
engine, one parameterised dirtiness primitive at leaf, and policy composition at
command. **There is exactly one dirtiness measurement in the design** —
`dirty_under` — used twice by `verify` with different pathspec sets. No path
predicate exists at leaf and no second probe exists anywhere; both were specified
by earlier drafts and are deleted (RV-307 F-2).

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

**`body_mode` without `body` is rejected on both surfaces**, with the same
message — one rule (*`body_mode` is meaningless without `body`*) instanced twice,
alongside the `--body-mode`-on-`record` rejection. A silent default would let a
caller believe an append happened when nothing was written, and would let the two
adapters diverge while both still passed the test matrix (RV-307 F-10).
`has_any()` is unaffected: `body_mode` alone never constitutes an edit.

**Dirtiness primitive (leaf, `src/git.rs`)** — one implementation, parameterised.

```rust
/// Is anything under `pathspecs` dirty? Runs the three dirtiness probes with
/// the given pathspec set. Returns a bool: it never computes
/// `checkout_state_id`, so it never calls `write-tree` and never takes
/// `.git/index.lock` (I2). An empty set means "the whole tree".
pub(crate) fn dirty_under(root: &Path, pathspecs: &[&str]) -> Result<bool, CaptureError>

/// Capture the git frame, ignoring paths under `excludes` when deciding
/// dirtiness. Delegates the dirty decision to `dirty_under`.
pub(crate) fn capture_with(root: &Path, excludes: &[&str]) -> Result<Frame, CaptureError>

/// Unchanged public behaviour — the existing three callers keep this signature.
pub(crate) fn capture(root: &Path) -> Result<Frame, CaptureError> {
    capture_with(root, &[])
}
```

`pathspecs` are *arbitrary* git pathspecs — negative (`:(exclude)…`) or positive
(`.doctrine/memory/items/<key>`, `:(glob).doctrine/adr/**`) — which is what lets
one primitive serve both of `verify`'s questions. Git's rule is that a path
matches iff it matches at least one positive pathspec (or there are none) *and* no
negative one, so the two questions cannot be folded into a single call: **two
calls, one implementation.**

*Revised twice.* The first draft proposed a separate `source_clean` probe, which
would have duplicated repo-identity derivation, the multi-root guard, submodule
rejection and ref resolution (§ 10, A1). The second parameterised `capture()`
itself, but that left `verify` with only a whole-frame answer when it needed a
narrow boolean — and computing a `Frame` for the claim question would have taken
the index lock on exactly the path I2 protects (§ 10, X1/F-11). Extracting
`dirty_under` is what A1 actually wanted: the shared measurement lifted once, with
`capture_with` as its first consumer. I1 still holds by construction —
`capture(root)` delegates with an empty slice.

Parameterisation is required because `capture()` **blanks the commit oid whenever
the tree is dirty** (`git.rs`: `commit: String::new(), // empty iff dirty`) — it
yields only a `checkout_state_id` hash, and you cannot subtract corpus paths from
a sha256. The three probes take pathspecs natively:

| Probe | Command | Pathspec |
|---|---|---|
| worktree | `git diff HEAD --binary …` | passed through |
| untracked | `untracked_fingerprint` → `git ls-files --others --exclude-standard -z` | passed through |
| index | `git diff-index --quiet --cached HEAD` | passed through |

✓ **Verified empirically, not assumed** (RV-307 brief line 3, Git 2.54.0, scratch
repos + this repo). Exclusion-only pathspec sets behave as required: a binary
worktree change confined to `.doctrine` yielded 0 diff bytes, adding a source
change yielded 3,824,818; the untracked leg printed nothing until a source file
was added; the index leg exited 0 then 1. `untracked_fingerprint` gains a
`pathspecs` parameter — a signature change to a private leaf fn whose only caller
is `dirty_under`.

**The two questions `verify` asks.** Policy stays entirely at command tier:

```rust
// 1. Is the CODE dirty?  (the anchor — corpus dirt is not evidence about it)
let source = capture_with(root, &corpus_excludes)?;

// 2. Is the CLAIM's own evidence committed?
let claim_dirty = dirty_under(root, &claim_pathspecs)?;
```

| Set | Contents | Why |
|---|---|---|
| `corpus_excludes` | `:(exclude)` + `DOCTRINE_PATHSPEC`; plus `:(exclude)memory` (`MEMORY_MASTERS_DIR`) **only when that directory exists** | dirt in doctrine's own authored trees says nothing about whether a claim about the code still holds |
| `claim_pathspecs` | the memory's **own item directory**, plus its declared `scope.paths` and `scope.globs`, per the construction rule below | this *is* the claim's evidence surface — the prose being attested, and the code it is attested against |

**`claim_pathspecs` construction is total** (RV-307 F-6, second round). Scope
arrays are free text and this corpus proves they are not uniformly repo-relative,
so the rule is stated rather than left to the implementation:

| Scope entry | Contributes | Why |
|---|---|---|
| repo-relative path or glob | as-is (globs via `:(glob)…`) | the normal case |
| absolute path **inside** the repo | normalised to repo-relative | 4 items carry these, e.g. `"/workspace/doctrine/src/worktree/jail.rs"` — git resolves a leading slash against the repo root, so unnormalised they match **nothing** |
| absolute path **outside** the repo | **dropped** | git cannot observe it |
| gitignored path | **dropped** | carries no tracked evidence; `.doctrine/state/` is scoped by several items and is ignored at `.gitignore:39` |
| `scope.commands` | **dropped** | not path-shaped (E5) |
| nothing left | item directory alone (E6) | the body must still be committed |

**Every drop is announced** (E7). Git does not fail a pathspec that matches
nothing (absent `--error-unmatch`), so an unnormalised or ignored scope entry
would otherwise shrink the claim surface *silently* — reaching a false attestation
by a quieter road than the one F-1 found. `verify` reports on stderr which scope
entries did not contribute and why, so a memory whose evidence surface is narrower
than its declared scope says so at the moment of attestation.

This is the correction for RV-307 F-1 and F-6. Excluding `.doctrine/**` wholesale
excluded the memory *being verified* — items live at
`.doctrine/memory/items/<key>/` — so `verify` would have stamped a HEAD that
provably lacked the attested body, and would have ignored a modified
`.doctrine/adr/001/layering.toml` that a memory explicitly scopes. **81 items in
this corpus carry `.doctrine/**` scopes.** Doctrine's ownership of the path
constant makes the exclusion legal under POL-002; it does not make the excluded
evidence irrelevant.

### 5.3 Data, State & Ownership

No schema change. No new persisted field. The verification axis keeps its three
existing fields; `write_body` owns no state; `dirty_under` returns a value.

`MEMORY_SHIPPED_DIR` (`.doctrine/memory/shipped`) and `MEMORY_ITEMS_DIR`
(`.doctrine/memory/items`) are both *under* `.doctrine`, so one exclusion root
covers them. Only `MEMORY_MASTERS_DIR` (`memory`, repo-root) sits outside — and
it is contributed only when the directory exists (E4). The item under attestation
is then re-admitted as a *positive* pathspec in `claim_pathspecs`; the two sets are
independent, so no re-inclusion magic is needed (git offers none).

### 5.4 Lifecycle, Operations & Dynamics

**Body on `record`** rides the existing scaffold seam, not `write_body`: `Draft`
and `RecordArgs` gain `body: Option<&str>`; `memory_scaffold` substitutes it for
`render_memory_md`'s output — exactly `seed_by_key`'s existing move
(`fileset.get_mut(1)` → `body.clone_into(b)`, `:1808-1810`). The transactional
`materialise_named` write is preserved unchanged.

**Body on `edit`.** `EditFields` gains `body` + `body_mode`; `has_any()` counts
body, so `--body` alone is a valid edit.

**Write ordering: validate everything, then write.** `run_edit` now writes two
files and is no longer atomic across them. **Every fallible step precedes every
disk write**, and the body still lands before the TOML:

```
let before = claim_snapshot(&doc);                     // 0. pure read, pre-edit
toml_changed = apply_edit(&mut doc, fields, today)?    // 1. VALIDATES; mutates in memory only
body_changed = write_body(dir, "memory.md", text, mode)?  // 2. first disk write
claim_changed = body_changed || claim_snapshot(&doc) != before;   // 3.
if claim_changed { clear_verification(&mut doc) }      // 4. D8
if body_changed  { doc["updated"] = today }            // 5. re-stamp — see below
if body_changed || toml_changed { write_atomic(toml_path) }  // 6. TOML last
```

**The claim-change signal** (RV-307 F-8, second round). `claim_snapshot` is a
pure read of the four claim-bearing fields — `title`, `summary`,
`scope.paths`/`globs`/`commands` — compared before and after `apply_edit`:

```rust
fn claim_snapshot(doc: &toml_edit::DocumentMut) -> ClaimSnapshot
```

Computed at the caller by comparison rather than by widening `apply_edit`'s
return type, because `apply_edit` returns one bool that cannot distinguish claim
fields from record fields, and changing its signature would break its existing
suite — which R3 forbids. Compared rather than inferred from *which flags were
supplied*, because supplied is not changed: setting `--title` to its existing
value must not clear a valid attestation (the T12b discipline, generalised).
Step 5 stays gated on `body_changed` alone, since `apply_edit` already stamps
`updated` for any metadata change.

The first round of this design named the claim fields in D8 and in a table, and
wired only `body_changed` into the operation — a decision recorded in two places
and implemented in none.

*Revised by RV-307 F-3.* The previous order put `write_body` first, on the reading
that the TOML content depends on `body_changed`. But `apply_edit` is fallible at
four sites *after entry* (`src/memory.rs:3817-3827` key/title, `:3844-3860`
status/lifespan, `:3893-3921` trust/severity, `:3935-3939` key normalisation),
while mutating only an in-memory `DocumentMut`. So `memory edit --body - --trust
bogus` would have rewritten `memory.md` and *then* errored — mutation on an
argument-validation failure, a new failure mode rather than the acknowledged crash
window. The fix is free: what must follow the body write is the TOML **write**, not
the TOML **computation**. Step 3 applies the body-dependent mutation to the same
in-memory document, after validation has already passed.

*Why the explicit re-stamp (RV-307 F-12).* There is no separate `stamp_updated`
helper to compose — `apply_edit` stamps `updated` itself, in a terminal block
gated on its own internal `changed` flag, which counts metadata fields only and
cannot see the body. Left as drafted, a body-only edit would never stamp
`updated`, falsifying T5. Step 3 re-stamps explicitly; the write is idempotent when
`apply_edit` already stamped (identical date), and `apply_edit`'s signature is
untouched — which matters under R3, since its existing suite must stay green
unchanged.

So `updated` is stamped, and the verification axis cleared, **iff the body
genuinely changed** — a `--body` that replaces content with itself is a full
no-op on both tiers (content and mtime hold), consistent with the existing
`apply_edit` no-op guard. A crash between steps 2 and 4 leaves a changed body with
a stale `updated`, never the reverse (R1).

Full two-tier atomicity would mean routing `edit` through the fileset/rollback
machinery — a large refactor of a shared write path, disproportionate to a crash
window this narrow. Stated, not buried.

**Verify.** The gate is **two questions, both of which must pass**:

```
if allow_dirty {
    let full = capture(root)?;      // UNEXCLUDED — the real state of the tree
    stamp(full);                    // Commit if genuinely clean, else CheckoutState
} else {
    let source = capture_with(root, corpus_excludes)?;   // 1. is the code dirty?
    let claim_dirty = dirty_under(root, claim_pathspecs)?;  // 2. is the claim committed?
    match (source.anchor_kind, claim_dirty) {
        (Commit, false) => attest against source.commit,    // the only success
        _               => refuse, naming which question failed,
    }
}
```

**Why `--allow-dirty` re-captures unexcluded** (RV-307 F-13). Both `Commit`
branches of `capture` set `checkout_state_id: String::new()` (`src/git.rs:2036`,
`:2048`) — only the dirty branch computes one. So a claim-only-dirty tree yields a
`Commit`-anchored `source` frame carrying no `checkout_state_id`, and the claim
leg is deliberately a bool (I2 — it must not compute one, or it takes the index
lock). The escape hatch would have had nothing to stamp, and would have recorded a
clean-looking attestation for a tree the operator explicitly flagged as dirty.
Taking the anchor from an unmodified `capture(root)` makes I4 literally true: the
escape hatch uses today's function, unchanged. The extra capture is confined to
that path, where the index lock is already acceptable — today's `--allow-dirty` on
a dirty tree takes it anyway. Same root cause as F-1: the default path was
reasoned about and the escape hatch left to inherit machinery built for a
different question.

Dirt in doctrine's authored corpus that the memory does not claim against no
longer blocks, and the anchor is a real addressable commit rather than a
`checkout_state_id`. But the memory's own body — and every path it declares a
scope over — must be committed, because that is what the stamp asserts.

**This costs the `record` → `verify` convenience, deliberately** (RV-307 F-1). A
freshly recorded memory's directory is untracked, so `verify` now refuses until it
is committed. The alternative was a `verified_sha` naming a commit that provably
did not contain the attested prose — demonstrated in a scratch repo, where
`git cat-file -e "$verified_sha:.../memory.md"` exited 128 while the drift count
that was supposed to catch it printed 0, then and forever. A worthless stamp is
worse than an extra `git commit`. The refusal says which of the two questions
failed, and what to do about it.

**Verification invalidation — claim fields, not record fields.** Editing what the
memory *asserts*, or *what it asserts against*, clears the axis:
`verification_state` → `unverified`, `reviewed` and `verified_sha` → empty (not a
new state; the scaffold default).

| Cleared by | Not cleared by |
|---|---|
| `body` (replace **and** append — appended prose is unverified claim) | `status` |
| `title` | `lifespan` |
| `summary` | `review_by` |
| `scope.paths` / `scope.globs` / `scope.commands` | `trust` |
| | `severity` |

*Widened by RV-307 F-8, closing OQ-1 as D8.* The left column constitutes the
claim; the right column is judgement *about* the record. Deferring title and
summary would have left a residual false-stamp path beneath § 1's broader closure
promise, and the stated reason for deferring (behaviour-preservation on shared
machinery) was void — D4 already changes `edit`'s behaviour through the same
`clear_verification` call, so the regression surface does not widen with the field
set. Scopes are included on the same reasoning: changing what a memory is attested
*against* invalidates the attestation as surely as changing the assertion.

**Staleness (the hand-edit path).** Clearing on the verb alone would invert the
guardrail: the sanctioned path would be stricter than hand-editing, rewarding
bypass. So `validate`'s staleness check additionally counts commits touching **the
memory's own item directory** since `verified_sha`, alongside the existing
scoped-paths count.

Coverage stated honestly (RV-307 F-7 — the draft claimed more): this catches the
verb path, hand-edits to **items**, and other agents. It does **not** catch
masters. Masters are minted unanchored (`anchor_kind = None`, `src/memory.rs:1705`)
so there is no `verified_sha` to diff from, and `collect_all` scans only items and
shipped — `MEMORY_MASTERS_DIR` appears in production code at `:1753` (record
placement) alone, never in `validate`. Masters are out of scope by D6; the gap is
carried as R5, not papered over.

Verified against live data: `git rev-list --count <verified_sha>..HEAD -- <dir>`
returned 3 for the memory whose stamp survived a committed body edit — the same
plumbing the scoped-paths check already uses.

**Refusal legibility.** The current message — "working tree is dirty: refusing to
verify … Commit first, then verify." — never mentions `--allow-dirty`. At the one
moment an agent is looking for the escape hatch, the tool hides it and prescribes
stashing. The refusal names its own flag.

### 5.5 Invariants, Assumptions & Edge Cases

- **I1** — the three existing `capture()` call sites see byte-for-byte identical
  frames. Guaranteed by construction (`capture` delegates with `&[]`), pinned by
  T11. `record`'s born anchor and the retrieve read path must not move.
- **I2** — the clean-after-exclusion path never calls `write-tree`, so it takes
  no index lock — preserving the lock-contention property `capture()` documents
  for concurrent doctrine processes (`src/git.rs:1996-2000`). **Strengthened**:
  `dirty_under` returns a bool and never computes `checkout_state_id`, so the
  *claim* probe never reaches `write_tree_with_retry` (`src/git.rs:1924`) even when
  the claim surface is dirty. Pinned by T23.
- **I3** — a genuinely dirty *source* tree still refuses without `--allow-dirty`.
- **I4** — `--allow-dirty` semantics unchanged: it bypasses **both** gate
  questions and stamps the frame from an **unexcluded** `capture(root)` — today's
  function, called as today (RV-307 F-13). Stated explicitly rather than inferred,
  because inferring it from the exclusion-aware frame gave an empty
  `checkout_state_id`.
- **I5** — `thread_expiry` untouched.
- **I6** — a successful attestation's `verified_sha` **contains the attested
  body**. The point of the claim probe; pinned by T24 (`git cat-file -e
  "$verified_sha:<dir>/memory.md"` must succeed).
- **E1** — thread memories vanish from `find`/`retrieve` after a body edit until
  re-verified (SL-008 D6 feeding on honest input). Correct but surprising —
  the verb says so on stderr.
- **E2** — repo-empty masters carry `anchor_kind = None`, never `CheckoutState`;
  the dirty gate never fires for them and nothing changes.
- **E3** — body content `-` is unreachable inline; use stdin.
- **E4** — `memory/` absent (every client project) → that exclusion root is
  simply not contributed.
- **E5** — `scope.commands` is not path-shaped and contributes no pathspec to
  `claim_pathspecs`; a memory scoped only by command has just its item directory
  in the claim surface.
- **E6** — a memory with an empty scope has a claim surface of exactly its own
  item directory. Still meaningful: the body must be committed.
- **E7** — a scope entry that contributes no pathspec (absolute-outside-repo,
  gitignored, `commands`) is **reported on stderr at verify time**, never dropped
  silently. Silent narrowing of the claim surface is a false attestation reached
  quietly; the operator is told when the evidence surface is smaller than the
  declared scope.

## 6. Open Questions & Unknowns

- ~~**OQ-1**~~ — **closed by RV-307 F-8**; answered as **D8**. Title, summary and
  scopes clear verification alongside body; status / lifespan / review_by / trust /
  severity do not.
- **OQ-2** — should own-directory drift feed *retrieve-side* `staleness`, not
  just `validate`? Deferred deliberately: it would reclassify a large fraction of
  the corpus at once and shift retrieval ordering broadly (D5).
- **OQ-3** — a body digest stamped at verify time would make invalidation
  git-independent and path-independent, covering uncommitted edits and masters
  (which have no `verified_sha`). Needs a new persisted field → schema change →
  its own slice. **No longer load-bearing** (it was, before the claim probe closed
  F-1): it would now buy master coverage and uncommitted-edit detection, not
  attestation truth.
- **OQ-4** — when do other kinds adopt `write_body`? This slice wires memory only.
- **OQ-5** — should the *source* leg narrow to the memory's declared scopes too,
  so a dirty `src/` file no memory claims against stops blocking? Raised by the
  F-6 disposition and deliberately not taken: it changes what the anchor means for
  every memory at once, where the claim probe only adds a check. I3 is preserved
  as-is.

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
- **D3 — extract one dirtiness primitive (`dirty_under(root, pathspecs)`);
  `capture_with` delegates to it; `capture()` delegates with `&[]`.** *Revised
  twice — by § 10 A1, then by RV-307 F-1/F-11.* The original decision was a
  separate `source_clean` probe, on the reasoning that `capture()` must stay
  untouched; that confused *behaviour* with *code* (the invariant worth protecting
  is I1, which delegation gives by construction). The second version parameterised
  `capture()` alone, which was still short: `verify` needs a *narrow boolean* for
  the claim question, and building a whole `Frame` to answer it would take the
  index lock on precisely the path I2 protects. Extracting the measurement is what
  A1 was actually asking for. *Alternative:* bake the exclusion into `capture()`
  unconditionally. *Rejected:* two of its three callers would be damaged —
  `record` would stamp a false born anchor and the retrieve read path would
  shift. *Alternative:* soften the refusal and keep stamping `checkout_state_id`.
  *Rejected:* weaker evidence for the common case, and it makes the default and
  `--allow-dirty` near-identical.
- **D9 — the gate asks two questions: is the code dirty, and is the claim
  committed?** The exclusion set answers the first; a positive pathspec set over
  the memory's item directory and declared scopes answers the second. *Forced by
  RV-307 F-1/F-6* — a single exclusion set cannot express "ignore corpus dirt
  except the part this memory is about", because git offers no re-inclusion after
  an exclude. *Alternative:* OQ-3's body digest. *Rejected here:* new persisted
  field, schema change, own slice — and unnecessary, since a positive pathspec
  answers the same question with machinery this slice already builds.
  *Alternative:* keep the blanket exclusion and accept the false stamp.
  *Rejected:* it is the exact hazard SPEC-007 § Concerns names, made cheap.
- **D4 — body edit clears the verification axis.** Affordable only because the
  gate relaxation makes re-verifying cheap; the halves pay for each other.
  *Alternative:* leave it (status quo) — rejected, the stamp would lie by
  one command. *Alternative:* a third "stale" state — rejected, new vocabulary
  for no gain over `unverified`.
- **D8 — invalidation covers claim fields, not record fields.** Body, title,
  summary and scopes clear; status, lifespan, review_by, trust and severity do
  not. *Closes OQ-1, forced by RV-307 F-8.* The line is what the memory asserts
  and what it asserts against, versus judgement about the record. *Alternative:*
  body only (the draft) — rejected: § 1 claims to close "nothing invalidates an
  attestation when the claim changes", and a summary rewrite is a claim change.
  *Alternative:* every field — rejected: a `trust` downgrade is a statement about
  the memory, not by it, and clearing on it would make `verify` and `trust`
  fight.
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
  bounded, tool-authored prose" — rich bodies make "bounded" false.
  **Corrected by review (§ 10, A3):** there is no *write-time* escaping on the
  `.md` tier to bypass — SL-024's escaping work was TOML free-text, and markdown
  prose is stored verbatim by design. The entire defence is **read-time**: the
  per-render nonce and `data, never instruction` framing at
  `src/memory.rs:2021-2045`, which this slice does not touch. So the risk is not
  "the new path bypasses escaping" but "bodies get large enough to matter". No
  size cap is imposed: anyone who can run the verb can already write the file, so
  a cap is theatre rather than a boundary. T16 pins the read-time framing against
  a hostile body written through the *new* path.
- **R3 — behaviour preservation on shared machinery.** `entity.rs` and
  `validate` are shared. Existing suites must stay green unchanged.
- **R4 — mass re-verification.** D4/D8 mean every claim-field edit costs a verify.
  Mitigated by the gate relaxation; if it still bites, that is evidence for OQ-3.
  D8 widens the trigger set, so this risk grows — accepted, because the
  alternative is a stamp that lies.
- **R5 — masters remain uncovered by every invalidation path** (RV-307 F-7).
  They are unanchored (no `verified_sha`) and `collect_all` does not scan them, so
  neither D5's own-directory drift nor D8's field-clearing reaches a master edit.
  D6 puts them out of scope; the only standing mitigation is the
  `mem.system.memory.global-master-authoring` guard memory (glob `memory/**`,
  severity high). Stated as a known gap, not a solved problem; OQ-3 would close it.
- **R6 — `verify` is now harder to satisfy, not easier, for the freshly-recorded
  memory.** `record` → `verify` refuses until the memory is committed (D9). The
  slice's headline benefit is narrower than the scope document implied: unrelated
  corpus dirt stops blocking, your own uncommitted claim still does. Accepted as
  the honest reading; `slice-230.md` is reconciled to say so.

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
| T7 | verify, unrelated `.doctrine/**` dirty, memory committed | succeeds, stamps **HEAD commit** (not `checkout_state_id`) |
| T8 | verify, memory dir untracked (`record` → `verify`) | **refuses** (D9); message names both the cause and `git commit` |
| T9 | verify, source tree dirty | still refuses; message names `--allow-dirty` |
| T10 | `--allow-dirty`, source tree dirty | unchanged, stamps `checkout_state_id` |
| T10b | `--allow-dirty`, **only the claim** dirty (source clean after exclusion) | stamps a real `checkout_state_id` from the unexcluded capture — **not** empty, not a bare commit (I4, F-13) |
| T11 | `capture(root)` == `capture_with(root, &[])` | I1 — identical frames on clean, dirty, unborn, non-repo |
| T12 | body edit via the verb | clears `verification_state`/`reviewed`/`verified_sha` |
| T12b | `--body` replacing content with itself | full no-op: `updated` **not** stamped, verification **not** cleared |
| T13 | **hand-edit** `memory.md` directly, commit, `validate` | flags stale via own-directory drift — must exercise the *bypass* path, since the verb path clears the stamp and would never reach the staleness check |
| T14 | `memory/` absent | exclusion root not contributed; no error |
| T15 | existing memory + entity suites | green unchanged (R3) |
| T16 | hostile body written via `--body`, then `show` | read-time nonce + `data, never instruction` framing intact (R2) |
| T17 | verify, **staged-only** corpus change | excluded; succeeds (index probe leg) |
| T18 | verify, **unstaged/binary** corpus change | excluded; succeeds (worktree diff leg) |
| T19 | verify, **untracked** corpus file outside the memory | excluded; succeeds (untracked leg) |
| T20 | edit `title` / `summary` / `scope.*`, each alone | clears the verification axis (D8) |
| T20b | set `--title` to its **existing** value | does **not** clear — `claim_snapshot` compares, it does not count flags (F-8) |
| T21 | edit `status` / `lifespan` / `review_by` / `trust` / `severity`, each alone | does **not** clear (D8's other half) |
| T22 | `body_mode` without `body`, CLI **and** MCP | rejected on both, same message (F-10) |
| T23 | verify on the clean-after-exclusion path while `.git/index.lock` is held | completes — I2 canary; fails if `write-tree` creeps back in |
| T24 | after a successful verify | `git show "$verified_sha:<dir>/memory.md"` equals the on-disk body **byte-for-byte** — I6. Existence (`cat-file -e`) would pass against any stale ancestor blob (F-14) |
| T24b | body **tracked but modified** (not untracked), verify | **refuses** — the case where existence and equality disagree, and the untracked leg does not fire |
| T25 | verify, memory scopes `.doctrine/adr/**`, an ADR under it modified | **refuses** — scoped corpus dirt is claim-relevant (F-6) |
| T26 | claim pathspec construction: absolute-inside-repo, absolute-outside-repo, gitignored, unmatched | normalised / dropped / dropped / no-op, per the § 5.2 rule (F-6 round 2) |
| T27 | verify with a non-contributing scope entry | stderr names the entry and the reason (E7) |

Closure: **every test in § 9 green** (stated as a set, not a numeric range, so a
test added by a later review cannot fall outside the gate by omission — RV-307
F-9); `doctrine check gate` clean; **REV-034 applied** so SPEC-007, REQ-147 and the
implementation agree.

## 10. Review Notes

### Internal adversarial pass (pre-external)

Four findings against the first draft. All integrated above; recorded here so the
reasoning is not lost.

- **A1 — parallel implementation in `source_clean` (material; design changed).**
  The draft added a second git probe alongside `capture()`, justified as "keeping
  `capture()` untouched". But `capture()` does far more than the dirty check —
  repo-identity derivation, multi-root guard, submodule rejection, ref resolution
  — so a parallel probe either duplicates all of it or silently returns a
  differently-constituted `Frame`. Both are worse than the problem being solved,
  and duplication is explicitly forbidden. **Root cause: I protected the wrong
  invariant.** What matters is that existing callers see identical frames, not
  that the function's bytes are unchanged. D3 revised to `capture_with(root,
  excludes)` with `capture()` delegating — I1 now holds by construction.
- **A2 — the changed-flag was underspecified (material; design changed).** The
  draft said a body-only edit "stamps `updated`", ignoring the case where
  `--body` replaces content with itself. Left as written, a no-op body write
  would stamp `updated` *and* clear a valid attestation — actively destructive.
  § 5.4 now specifies the composed flag and the exact step order, and T12b pins
  it. This also revealed *why* body-first ordering is mandatory: the TOML content
  depends on `body_changed`, so the body write must precede it. The draft had the
  right order for the wrong reason (crash-safety alone).
- **A3 — R2 was confused about where the defence lives (correction).** The draft
  warned the new path "must not bypass the scaffold's escaping". There is no
  write-time escaping on the `.md` tier — SL-024's work was TOML free-text, and
  markdown bodies are stored verbatim by design. The defence is entirely
  read-time (per-render nonce + data-framing), and this slice doesn't touch it.
  R2 rewritten; T16 added to pin read-time framing against a body written through
  the new path.
- **A4 — T13 would not have tested what it claimed (test gap).** It asserted
  `validate` flags staleness after a body edit — but under D4 a *verb* edit
  clears the stamp, so there would be no `verified_sha` left to compare and the
  staleness check would never run. The test only has meaning on the **hand-edit
  bypass** path, which is the whole reason D5 exists. T13 rewritten to hand-edit
  the file directly.

**Feasibility claim checked, not assumed:** all three dirtiness probes accept
pathspecs natively — `untracked_fingerprint` shells `git ls-files --others
--exclude-standard -z` (`src/git.rs:2201`), so exclusion is a parameter, not a
re-implementation. ✓

**Not found wanting:** the layering (ADR-001) holds — `entity` is engine and
imports only `fsutil`; `corpus_guard` and `git` are leaf; the pure predicate
takes roots as data rather than reaching up to command tier for
`MEMORY_MASTERS_DIR`. POL-002 is satisfied by guarding the `memory/` root on
existence rather than assuming it. STD-001 is satisfied by reusing
`DOCTRINE_PATHSPEC` instead of minting a literal.

### External review — RV-307 (codex/GPT, inquisitor posture)

Twelve findings: five blockers, two majors, five minors. **All disposed
`fix-now`; none deferred, none tolerated.** Full charges, evidence and responses
are on the ledger (`doctrine review show RV-307`) — summarised here only where the
design moved.

**Blockers.**

- **F-1 — false attestation (design changed; D9, I6, T8, T24).** The relaxation
  excluded `.doctrine/**` wholesale, and memory items live *inside* it. So
  `verify` would have stamped `verified_sha = HEAD` for a memory whose body HEAD
  did not contain, and D5's own-directory drift count — which counts *commits* —
  would have returned 0 forever, defeating the invalidation precisely in the case
  the relaxation created. Proven in a scratch repo, not argued. Fixed by the claim
  probe rather than by OQ-3's schema change. **Root cause: the exclusion set was
  written from the perspective of "what is noise?" and never asked "where does the
  thing being attested actually live?"**
- **F-3 — mutation on argument-validation failure (design changed; § 5.4).**
  Body-first ordering meant `--body - --trust bogus` rewrote `memory.md` and then
  errored. Fixed free, because `apply_edit` mutates only in memory: validate
  first, write second. The draft had confused "the TOML *write* must follow the
  body" (true) with "the TOML *computation* must follow it" (false).
- **F-4 / F-5 — governance debt (discharged).** The Revision was promised twice
  and never minted; worse, the amendment inventory named two sites and missed
  **REQ-147**, an active SPEC-007 member whose *title is the retired contract
  verbatim*. REV-034 now carries REQ-147 as its primary row plus SPEC-007, and
  `SL-230 needs REV-034` is authored.
- **F-6 — blanket exclusion hid claim-relevant evidence (design changed; D9).**
  81 items in this corpus scope into `.doctrine/**`. Doctrine's ownership of the
  path constant makes the exclusion legal under POL-002; it does not make the
  excluded evidence irrelevant. Declared scopes joined the claim surface.

**Majors.** F-8 closed OQ-1 as **D8** (claim fields clear, record fields do not) —
the deferral's stated reason was void, since D4 already changes the same
behaviour. **F-12 was raised by the responder during disposition**, and had
escaped both the internal and external passes: there is no `stamp_updated` helper
to compose — `apply_edit` stamps `updated` itself, gated on a metadata-only flag,
so a body-only edit would never have stamped it and T5 was unsatisfiable.

**Minors.** F-2 purged the dead `is_excluded` / stale `source_clean` that A1's
integration had left in the normative sections. F-7 struck a false capability
claim (own-directory drift does *not* catch masters — they are unanchored and
`collect_all` never scans them); the gap is now R5. F-9 replaced the closure
range with a set. F-10 made the `body_mode`-without-`body` contract total on both
surfaces. F-11 added the probe-partition tests (T17-T19) and the I2 lock canary
(T23).

**Acquitted on the evidence.** The git pathspec claims — the one place the design
had marked ✓ without proof at the time of writing. The tribunal built scratch
repositories under Git 2.54.0 and put all three probes to the question:
exclusion-only pathspec sets behave exactly as § 5.2 asserts. Recorded because a
review that only ever confirms suspicion is not measuring anything.

**Pattern across the twelve.** Four of the findings (F-2, F-7, F-9, F-12) are
*incomplete integration of already-settled corrections* — A1's decision landed in
D3 but not in § 5.1/§ 5.3; A3's T16 reached the table but not the gate; A2
composed a flag over a step that does not exist. The lesson is recorded in the
closure criterion (a set, not a range) and is worth carrying beyond this slice:
**integrating a finding means sweeping every section it touches, not the one where
the decision is recorded.**

#### Round 2 — confirmatory pass (same raiser)

Ten of the twelve verified. **Two contested, and both contests were correct** —
including, pointedly, one instance of the very pattern the paragraph above had
just diagnosed.

- **F-8 contested and sustained (design changed).** D8 was written as a decision
  *and* a table naming four claim fields, while § 5.4 step 3 still read
  `if body_changed`. A decision recorded twice and implemented nowhere. Now wired
  through `claim_snapshot`, compared before/after `apply_edit` — comparison rather
  than flag-counting, so an idempotent `--title` does not clear a valid stamp
  (T20b).
- **F-6 contested and sustained (design changed).** The claim surface was
  under-specified in a way this corpus falsifies: four items carry **absolute**
  scope paths, which git resolves against the repo root and which therefore match
  nothing; several scope `.doctrine/state/`, which is gitignored. Neither errors —
  git does not fail an unmatched pathspec — so both shrink the claim surface
  *silently*. § 5.2 gains a total construction rule, and E7 makes every drop
  audible on stderr. **Same defect class as the original F-1, reached by a quieter
  road.**

Two new findings, both sustained:

- **F-13 (blocker).** `--allow-dirty` had nothing to stamp. Both `Commit` branches
  of `capture` leave `checkout_state_id` empty (`src/git.rs:2036`, `:2048`), and
  the claim leg is a bool by design (I2). A claim-only-dirty tree would have
  recorded a clean-looking attestation for a tree the operator flagged as dirty.
  Fixed by taking the escape hatch's anchor from an unexcluded `capture(root)`.
  **Same root cause as F-1** — the default path reasoned about, the escape hatch
  left to inherit machinery built for a different question.
- **F-14 (minor).** T24's `cat-file -e` proved a blob existed at the path, which
  any stale ancestor body would satisfy. Now byte equality, plus T24b for the
  tracked-but-modified case where existence and equality disagree.

**What the two rounds together say about this design.** The recurring failure was
never the mechanism — it was reasoning about the principal path and letting
adjacent paths inherit assumptions that no longer held: the escape hatch (F-13),
the bypass path (A4), the hand-edit path (D5), the no-op write (A2), the scope
entry that isn't repo-relative (F-6 round 2). Five instances of one habit. The
design is now specified at those edges rather than at the centre alone, which is
why the test matrix roughly doubled.
