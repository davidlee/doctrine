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
(`:(literal).doctrine/memory/items/mem_<uid>`, `:(glob).doctrine/adr/**`) — which
is what lets one primitive serve both of `verify`'s questions. **That latitude is
the leaf's, not the caller's**: the command tier composes the sets and is
responsible for neutralising untrusted input before it reaches here (F-18). A
primitive that accepts pathspec magic is correct; a policy that forwards memory
text into one unfiltered is not. Git's rule is that a path
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

**The base of the claim surface is the *canonicalised uid* directory** (RV-307
F-15) — `.doctrine/memory/items/mem_<uid>/`, never the path `resolve_show`
returned. `run_verify` resolves through `fsutil::safe_join`, a plain
`tree_root.join(rel)` with **no canonicalisation** (`src/fsutil.rs:20-33`), so a
reference given as a *key* yields `.doctrine/memory/items/<key>` — and every key
in `items/` is a **symlink** to the uid dir. **Git does not traverse symlinks in
pathspecs**: such a pathspec matches the symlink entry alone, so all three probe
legs report clean while the body is modified. Proven, not argued (git 2.54.0,
scratch repo, tracked memory with a modified `memory.md`):

| Probe, via **key** symlink | Result | Same probe, via **uid** dir | Result |
|---|---|---|---|
| `git diff HEAD --name-only` | *(empty)* | `git diff HEAD --name-only` | `…/memory.md` |
| `git diff-index --quiet --cached HEAD` | exit 0 — *clean* | same | exit 1 — *dirty* |
| `git ls-files` | 1 entry: the symlink | `git ls-files` | `memory.md`, `memory.toml` |

That is F-1 restored through the reference form, on the mainstream path: D4/D8
clear the axis on every claim edit, so *edit → re-verify* is the flow this slice
creates, and agents address memories **by key** (the boot snapshot and
`/retrieve-memory` both emit keys). T8's fresh-`record` case survives only by
luck — the untracked *symlink* trips the untracked leg — so T8 passing proves
nothing here. `validate`'s own-directory drift count (D5) takes the uid dir for
the identical reason.

**Scope entries are free text**, and this corpus proves they are not uniformly
repo-relative. Round 5 stated the treatment as a table of *cases*, which RV-307
F-26 falsified on both counts: two rows claimed the same entry (a tracked symlink
whose target resolves outside the repo), and the universal rule that was supposed
to govern them all — canonicalise everything (I9) — cannot be applied to a
pattern or to a path that no longer exists. So the rule is stated as an **ordered
algorithm**. The order is not presentation: it is what makes the classification
total and disjoint, and every class is decided by **probe outcome**, never by the
shape of the string.

Applied to each entry of `scope.paths` and `scope.globs`:

1. **Empty or whitespace-only → malformed, refuse** (E11). Never emitted in any
   form: prefixed, a bare `:(literal)`/`:(glob)` matches the entire index; raw,
   git rejects it outright.
2. **Classify the shape.** An entry containing `*`, `?` or `[` is a **pattern**;
   anything else is a **concrete path**. They resolve differently, because a
   pattern has no single real path to resolve to.
3. **Resolve.** A concrete path resolves whole. A pattern resolves its longest
   wildcard-free prefix and re-appends the wildcard tail. Necessary, not cosmetic:
   `:(glob)<slug-symlink>/**` matches **nothing** and reports clean against a
   modified target, exactly as the literal form did in F-20 — verified, git 2.54.0:
   0 files via the link, 1 via the resolved prefix, `diff-index` exit 0 versus
   exit 1. No glob declaration in this corpus is currently symlink-rooted, so this
   closes a latent hole at zero migration cost.
4. **Classify on the resolution outcome.**
   - **Resolves inside the repo** → emit it, repo-relative and magic-prefixed.
     Step 5 decides whether it contributes. (Normalising an absolute-inside entry
     is hygiene, not a correctness fix — git converts absolute pathspecs itself —
     but it keeps the emitted pathspec legible and stable across checkouts.)
   - **Resolves outside the repo** → **malformed, refuse.** This is F-26's
     collision, cut in favour of the probe: `git ls-files --error-unmatch` on such
     a path aborts `exit 128` (`is outside repository`), and *aborting the probe*
     is the malformed class's defining property. Where the entry points is not the
     test; what it does to the probe is. An absolute-outside literal and a tracked
     symlink whose target is outside are therefore **one** class, not two.
   - **Does not resolve** → ask git's *history*, not the filesystem:
     `git rev-list --all --max-count=1 -- <entry>`, asked with the **same
     magic-prefixed form** the surface would have emitted, never the raw entry.
     F-18 applies to this probe as much as to the claim probe: an unresolved
     entry is still untrusted text, and interpolated raw a `:(exclude)…` value
     matches everything it does not name, so history would report "tracked once"
     for an entry naming nothing.
     - **Ever tracked** → **stale, refuse** (D10). The memory names evidence that
       once existed and no longer does — a real defect, and a fixable one.
     - **Never tracked** → **unobservable**, reported and attested over (D10).
       Git has never held this path and no scope edit can change that.
5. **Contribution.** `git ls-files --error-unmatch` over the emitted set. An entry
   that resolves inside the repo but matches no tracked file is **unobservable**
   on the same terms as step 4's never-tracked branch.
6. `scope.commands` never enters: not path-shaped, exempt by kind (E5).
7. Nothing left → the uid directory alone (E6). It is unconditional, so the claim
   surface is never empty.

**Why history and not the filesystem** decides step 4's last split (RV-307 F-25).
The distinction D10 needs is *genuine defect* versus *git can never see this*, and
existence-on-disk does not carry it: `.claude/skills/dispatch-agent/SKILL.md` is
absent from a source checkout and present in an installed one, so a filesystem
test would refuse a whole class of harness memory in one tree and admit it in
another — E9's checkout-dependence, in the one place it must not bite. History is
identical in every checkout. Measured over the 43 non-resolving declarations in
this corpus: 30 were tracked once (`src/worktree.rs`, `src/skills.rs`,
`doc/memory-spec.md`, `web/map/app.js` — every one a moved or refactored source
path), and 13 never were (`.claude/skills/**`, `.doctrine/skills/**`,
`.harness/probe/**`). The split is clean, and it falls exactly where the design
argues it should.

Step 3 and step 4 together are why **I9 is total** rather than aspirational: every
path reaching the surface has been resolved, and the two shapes that cannot be —
absent paths and outside targets — are refused at step 4 rather than emitted
uncanonicalised.

A **gitignored** entry that resolves and is tracked is *kept*: ignore rules do not
bind tracked files, so a force-added path under an ignored root is real evidence
(E8).

**Scope entries are data, not pathspec syntax** (RV-307 F-18). `scope.paths` and
`scope.globs` are free text in an *untrusted* substrate — SPEC-007 § Concerns
treats stored memory text as hostile input (R2) — and § 5.2 previously called the
pathspec set "*arbitrary* git pathspecs". Interpolated raw, a scope entry of
`:(exclude).doctrine/memory/items/mem_<uid>` **subtracts the mandatory uid
directory from the claim surface**, and the attestation goes through against a
modified body. Demonstrated, not postulated (git 2.54.0, scratch repo, body
modified):

```
git diff-index --quiet HEAD -- items/<uid>                            → exit 1  (dirty, correct)
git diff-index --quiet HEAD -- items/<uid> ':(exclude)items/<uid>'    → exit 0  (CLEAN — false attestation)
git diff-index --quiet HEAD -- items/<uid> ':(literal):(exclude)…'    → exit 1  (dirty — magic neutralised)
```

So every scope-derived entry is emitted **magic-prefixed**: `:(literal)` for
`scope.paths`, `:(glob)` for `scope.globs`. Git parses magic only at the head of a
pathspec, so the prefix renders the remainder inert — a hostile entry degrades to
a literal path that matches nothing (and is then reported by E7), never to an
operator on the surface. The two prefixes are named constants, not inline literals
(STD-001). The uid directory is emitted the same way, so **nothing a memory
declares can subtract it** (I8).

**An empty entry is worse than a hostile one** (RV-307 F-23). The prefix rule is
unconditional, so an empty or whitespace-only scope value would be emitted as a
*bare* `:(literal)` or `:(glob)` — which matches **the entire index**, not
nothing. Verified: `git ls-files -- ':(literal)'` returns every tracked file. That
inverts the failure — the claim surface becomes the whole repository and `verify`
refuses on any unrelated dirt anywhere — and it is why the empty case belongs in
D10's *malformed* class, dropped before emission and reported, never prefixed and
passed through. Raw, it is not survivable either: git rejects an empty pathspec
outright (`fatal: empty string is not a valid pathspec`). **Never emit a bare
magic prefix** (E11).

**Canonicalisation is a rule about every path in the surface, not just the uid
directory** (RV-307 F-20). F-15's remedy resolved the item directory through its
key symlink; declared scopes need the identical treatment, because git's blindness
to symlinks is a property of *pathspecs*, not of item directories. A scope naming
a tracked symlink passes the `--error-unmatch` check — the symlink is tracked —
while the probe sees only the link blob and not the target's content:

```
git diff-index --quiet HEAD -- items/link-to-target   → exit 0   (blind)
git diff-index --quiet HEAD -- real/target.txt        → exit 1   (dirty)
```

This corpus carries **2,001 tracked symlinks** — doctrine mints a slug symlink per
entity — so the *readable* form an agent naturally scopes
(`.doctrine/adr/001-module-layering`) is precisely the blind one. Resolution is
therefore step 3 of the construction algorithm above, and applies to patterns as
well as concrete paths; an entry whose target resolves outside the repository is
**malformed and refuses** (E13, F-26). One rule, stated once and scoped to the
question it holds for: **nothing reaches `verify`'s claim surface uncanonicalised**
(I9) — and nothing canonicalised reaches `validate`'s historical one (D11, F-27).

**The inside/outside split is a property of the checkout, not of the string**
(E9). `"/workspace/doctrine/src/worktree/jail.rs"` — carried by four items — is
inside the primary tree and *outside* a dispatch worktree at any other path. So a
memory's claim surface narrows when it is verified from a linked worktree, in a
repo whose whole dispatch model is linked worktrees. E7 makes it audible; the
design states it rather than letting it be discovered.

**Non-contribution splits four ways, and refusal follows actionability** (D10,
RV-307 F-6/F-21/F-25). Git does not fail a pathspec that matches nothing (absent
`--error-unmatch` — `git diff HEAD -- src/nope.rs` exits 0 silently), so a dropped
or unmatched scope entry shrinks the claim surface *silently*.

| Class | Example, from the real corpus | Response |
|---|---|---|
| **Malformed / probe-aborting** — empty, or resolving outside the repo | `""`, `"/etc/passwd"`, a tracked symlink whose target is outside | **refuse.** It can abort the probe, and that is the class's definition (E11) |
| **Stale** — does not resolve, but git tracked it once | `src/worktree.rs`, `src/skills.rs`, `doc/memory-spec.md`, `web/map/app.js` | **refuse.** The memory names evidence that existed and no longer does. Refusal is the only response that gets it fixed, and the fix is a path correction |
| **Unobservable** — never tracked, or resolves but untracked | `.claude/skills/**`, `.harness/probe/**`, `.doctrine/state/slice/` | **report + `validate` finding**, then attest. Git has never held this evidence and no scope edit can change that |
| **Observable** | anything tracked | **must be clean, or refuse** — this is the claim probe proper |

So `verify` refuses on the malformed and stale classes, and on any *observable*
entry being dirty; otherwise it attests, while naming on stderr every entry that
did not contribute and why. `validate` raises the unobservable entries as a
corpus-health finding so the overclaim is aggregated rather than emitted as
per-attestation noise. `scope.commands` is *exempt by kind*, not a failed entry:
it is structurally non-path (E5), a property of the schema rather than of this
tree.

***What `verified_sha` asserts, stated rather than implied*** (RV-307 F-25). The
round-3 draft answered non-contribution with a stderr advisory and § 5.2 rejected
it — *a warning does not make an unobservable claim committed*. F-25's charge is
that the round-5 design then adopted that same advisory for two of its four
classes, condemning the mechanism in one paragraph and relying on it in the next.
Sustained. The refusal is now extended to every class where refusal is
**actionable**, and for the one remaining class the design states its reading
outright instead of arguing around it:

> A `verified_sha` asserts that **everything git could observe about this claim
> was committed and unchanged at that commit** — not that every declared entry was
> observed. A memory scoping a path git has never tracked carries an attestation
> over a proper subset of what it declares.

That is the weak reading, taken deliberately. The strong reading — every declared
entry observed, or no stamp — is the round-4 blanket refusal, and measurement
falsified it: it treats a memory about harness behaviour as defective for scoping
the harness, and the remedy it prescribes (edit the scope) is itself a claim edit,
so it clears the axis and guarantees the memory is unverified in exchange for
making it verifiable. Under the algorithm above the blanket rule costs 36 active
items; the actionable cut costs **4 currently-stamped active items**, each a
moved-source-path correction.

The weak reading's residual is real and is **not** closed here: a consumer of the
stamp cannot distinguish a full attestation from a partial one, because the
shortfall lives on stderr and in `validate`, not on the record. Closing it means
persisting the covered surface — a new field, i.e. a schema change and its own
slice, exactly OQ-3's shape. Routed as **IMP-318** and carried as **R8**, on the
R5 precedent: state the gap, do not paper it.

**The seam is the refusal path in `run_verify`, not a side-channel.** Surface
construction returns the pathspec set *and* a classified list of non-contributing
entries; the command tier refuses on any **malformed or stale** member before it
probes anything, and passes the unobservable remainder to the reporter. Detection
is `git ls-files --error-unmatch` over the constructed set plus one
`git rev-list --all --max-count=1` per non-resolving entry — the same plumbing,
no new dependency. E7 is the wording of that report; E11 and E12 the refusals.

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
supplied*, because supplied is not changed: re-setting a field to its existing
value must not clear a valid attestation (the T12b discipline, generalised).
Step 5 stays gated on `body_changed` alone, since `apply_edit` already stamps
`updated` for any metadata change.

*Where the comparison actually earns its keep* (RV-307 F-17). `apply_edit`'s bool
is a **changed** flag for the scalars and a **supplied** flag for the scope
arrays: `title`, `summary`, `lifespan` and `review_by` each read the existing
value and assign only on difference (`src/memory.rs:3826-3843`), while all three
scope arms rebuild the array and set `changed = true` unconditionally
(`:3944-3978`). So an idempotent `--title` is already a no-op without
`claim_snapshot`, and **scope is the only field where the comparison and
`apply_edit` diverge** — which is why T20b is retargeted there. One consequent
behaviour, stated rather than left to be discovered: an idempotent `--path-scope`
still stamps `updated` (the terminal block at `:3980-3983` fires on `changed`)
while the attestation correctly survives. It is the one place the two move apart;
T29 asserts it.

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
escape hatch uses today's function, unchanged. It is **not an extra capture**: the
`allow_dirty` branch is taken *before* the gate probes, so exactly one capture
runs on that path — the same single `capture(root)` `run_verify` performs today
(`src/memory.rs:3382`), and the index lock it may take is one today's
`--allow-dirty` takes anyway. Same root cause as F-1: the default path was
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
scoped-paths count — **the canonicalised uid directory**, for the symlink reason
in § 5.2 (F-15): a `rev-list … -- <key-symlink>` counts commits touching the
symlink, which is to say zero, forever.

***The claim-surface constructor serves `verify` alone*** (D11, RV-307
F-19/F-27/F-28/F-29). Round 4 widened it to `validate` as well, on the reasoning
that two constructions of "the claim's evidence surface" in one slice is the
parallel implementation A1 already rejected. Round 6 falsified that reasoning:
**the two verbs are not asking the same question, so a shared surface is not
shared correctness.**

`verify` asks *is this evidence dirty right now*, and canonicalisation is
mandatory — a pathspec naming a symlink reads clean while its target is modified
(F-15, F-20). `validate` asks *how many commits touched this evidence since
`verified_sha`*, and canonicalisation is **actively wrong**: it resolves against
today's checkout and then queries yesterday-to-today history. Demonstrated (git
2.54.0, scratch repo, a committed retarget of `link` from `real/a` to `real/b`):

```
git rev-list --count BASE..HEAD -- link         → 1   (the retarget is drift, correctly counted)
git rev-list --count BASE..HEAD -- real/b       → 0   (canonicalised: the drift disappears)
git diff-index --quiet HEAD -- link             → 0   (blind — why verify MUST canonicalise)
git diff-index --quiet HEAD -- real/b           → 1   (dirty — correct for verify)
```

One transformation, opposite correctness on the two consumers. D11 was an
over-generalisation of I9 — the same defect class this review keeps finding, one
level up: the remedy was written against the finding (F-20's symlink blindness)
and promoted to a universal invariant without checking the sibling it would reach.

Two further facts made the shared form unbuildable in any case:
`memory_health_findings(root, &[Memory], today)` (`src/memory.rs:3400`) receives
no item directory; `run_validate` discards `_dir` at `:3480-3483`; and
`collect_all` (`:2826-2834`) unions `items/` and `shipped/` into one `Vec<Memory>`,
so the row's origin is not merely absent but unrecoverable from `uid` — which is
why `read_body` (`:2788-2797`) probes both roots. And the design specified a
refusal for `verify`'s malformed class while saying nothing about what a
corpus-wide `validate` does with one (F-29).

**So `validate` keeps its existing raw seam and gains only D5's own-directory
count.** The raw seam's defects are real, pre-existing, and named rather than
silently inherited:

| Defect in `validate`'s raw seam (`src/memory.rs:3413-3421`) | Consequence |
|---|---|
| gated on `!memory.scope.paths.is_empty()` (`:3414`) | a memory scoped **only by glob** is never staleness-checked |
| `scope.globs` never passed | glob scopes are invisible to drift |
| absolute entries passed as-is | absolute-scope items match nothing, or abort the call |
| no magic neutralisation | the F-18 injection reaches `commits_touching` too |

They are **not** fixed here, because fixing them correctly means building
`validate` a *history-stable* surface — a second constructor with different rules,
not a reuse of this one — and that is a change to corpus-wide staleness ranking,
which is OQ-2's deferred decision. Carried as **R7** and routed as **IMP-317**.

D5's own-directory count is unaffected by F-27: the uid directory is reached by
resolving the *key* symlink, and a uid never changes, so there is no retarget for
history to lose. Stated rather than left to be inferred.

**`retrieve::git_facts` is the third consumer and is likewise not converted**
(RV-307 F-24). It gates on `m.scope.paths.is_empty()` and passes `scope.paths` raw
(`src/retrieve.rs:556-557`) — the same defects, plus the same missing directory
(`:628` has `root`, `Memory` and `Snapshot`, no `dir`).

**Adoption is not a call-site swap, and round 5 was wrong to claim it was**
(RV-307 F-28). The constructor takes `(root, memory, dir)`; neither `validate` nor
`retrieve` has a `dir` to give it, and `collect_all` has already discarded the
provenance that would supply one. Adoption requires threading item-directory
origin through `collect_all` and `memory_health_findings` — a dataflow change, not
two arguments. IMP-317 carries that corrected scope. Bounding the slice is
legitimate; misdescribing the cost of un-bounding it later is not.

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
  body**. The point of the claim probe; pinned by T24 as **byte equality** —
  `git show "$verified_sha:<uid-dir>/memory.md"` equals the on-disk body.
  Existence (`cat-file -e`) is *not* the assertion and never was sufficient: any
  stale ancestor blob at that path satisfies it (F-14).
- **I8** — nothing a memory *declares* can subtract from what it is *measured
  against*. Scope entries are emitted magic-prefixed (§ 5.2, F-18), so the uid
  directory is present in the claim surface unconditionally. Pinned by T30.
- **I9** — nothing reaches **`verify`'s** claim surface uncanonicalised: the uid
  directory and every declared scope alike is symlink-resolved before it becomes a
  pathspec — a concrete path whole, a pattern by its longest wildcard-free prefix
  (§ 5.2, F-15/F-20/F-26). Total by construction, because the two shapes that
  cannot be resolved are refused rather than emitted: an absent path is stale, an
  outside target is malformed. **Scoped to `verify` deliberately** (F-27):
  canonicalisation is required for a dirtiness question and wrong for a historical
  one, so it must not reach `validate`'s drift count (D11). Pinned by T28, T33,
  T36, T37.
- **E1** — thread memories vanish from `find`/`retrieve` after a body edit until
  re-verified (SL-008 D6 feeding on honest input). Correct but surprising —
  the verb says so on stderr.
- **I7** — the claim surface names **real tracked files**, never a symlink
  standing in for them: it is rooted at the canonicalised uid directory (§ 5.2,
  F-15). Pinned by T28, which verifies *by key* against a modified tracked body
  and requires a refusal.
- **E2** — masters and shipped never reach the gate at all: `run_verify` resolves
  through `items_root` alone (`src/memory.rs:3378-3380`), so `verify` is
  items-only by construction and no master or shipped memory is addressable by it.
  (Repo-empty masters additionally carry `anchor_kind = None`, never
  `CheckoutState`.) Checked because the claim probe made the question live —
  `.doctrine/memory/shipped/` is gitignored (`.gitignore:44`, 0 tracked files), so
  had `verify` reached it, its whole claim surface would have been untracked and
  the probe would have passed vacuously. It cannot. Nothing to do; recorded so the
  next pass need not re-derive it.
- **E3** — body content `-` is unreachable inline; use stdin.
- **E4** — `memory/` absent (every client project) → that exclusion root is
  simply not contributed.
- **E5** — `scope.commands` is not path-shaped and contributes no pathspec to
  `claim_pathspecs`; a memory scoped only by command has just its item directory
  in the claim surface.
- **E6** — a memory with an empty scope has a claim surface of exactly its own
  item directory. Still meaningful: the body must be committed.
- **E7** — an *unobservable* scope entry is **reported on stderr at verify time
  and raised by `validate` as a corpus-health finding** (D10). Silent narrowing of
  the claim surface is a false attestation reached quietly; the operator is told
  when the evidence surface is smaller than the declared scope, and the
  corpus-wide verb makes the backlog visible rather than per-attestation noise.
  `scope.commands` is exempt by kind and is not reported as a defect (E5).
  *Narrowed by F-25:* the stale class no longer reaches E7 — it refuses (E12).
- **E8** — a **gitignored** scope entry is *kept*, not dropped (§ 5.2, F-16).
  Ignore rules do not bind tracked files, so dropping would discard real evidence
  from a force-added path; keeping is inert when the path is genuinely untracked.
- **E11** — an empty or whitespace-only scope entry is **dropped before emission
  and refused**, never prefixed: a bare `:(literal)`/`:(glob)` matches the entire
  index (F-23), and raw it aborts the probe. No bare magic prefix is ever emitted.
- **E12** — a **stale** entry refuses (RV-307 F-25): it does not resolve, but
  `git rev-list --all --max-count=1 -- <entry>` is non-empty, so git tracked it
  once and the memory now names evidence that no longer exists. The refusal names
  the entry and says the path moved. Distinguished from *unobservable* by history
  rather than by the filesystem, because existence is checkout-dependent and
  history is not (§ 5.2, E9). Measured cost: 4 currently-stamped active items.
- **E13** — a scope entry that resolves **outside** the repository is malformed,
  whether it is an absolute literal or a tracked symlink pointing out (F-26). One
  class, because the test is what the entry does to the probe — `ls-files
  --error-unmatch` aborts `exit 128` — not where it points.
- **E9** — the absolute-inside / absolute-outside classification is a property of
  the **checkout location**, not of the scope string. The four items scoped
  `"/workspace/doctrine/…"` resolve inside the primary tree and outside a linked
  worktree, so the same memory has a narrower claim surface when verified from a
  dispatch worktree — announced by E7 rather than silent.

## 6. Open Questions & Unknowns

- ~~**OQ-1**~~ — **closed by RV-307 F-8**; answered as **D8**. Title, summary and
  scopes clear verification alongside body; status / lifespan / review_by / trust /
  severity do not.
- **OQ-2** — should own-directory drift feed *retrieve-side* `staleness`, not
  just `validate`? Deferred deliberately: it would reclassify a large fraction of
  the corpus at once and shift retrieval ordering broadly (D5). **Load-bearing on
  a second question** (RV-307 F-24, widened by F-27): it gates whether a
  *history-stable* claim surface is built at all, and therefore whether either
  `validate` or `retrieve::git_facts` leaves the raw seam. Both are the same
  corpus-wide reclassification by different routes. Routed as **IMP-317**, which
  closes as `wont-do` if OQ-2 answers "no" — in which case R7 is restated as
  intended rather than provisional. Tracked as QUE-175.
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
- **D10 — non-contribution is classified, and refusal follows actionability.**
  *Revised three times: forced by RV-307 F-6 round 3, bounded by F-21's census,
  re-cut by F-25.* Malformed (empty, or resolving outside the repo) and **stale**
  (does not resolve, but git tracked it once) refuse; **unobservable** (never
  tracked) is reported and attested over; observable must be clean. The line is
  whether refusal is *actionable*: a moved source path is fixed by correcting it,
  while no edit makes `.claude/skills/**` visible to git.
  *Alternative:* warn and proceed on everything (round-3 draft) — rejected: a
  warning does not make an unobservable claim committed. *Alternative:* refuse on
  everything (round-4 draft) — rejected on **measurement**: 36 active items stop
  verifying, including an entire class of harness memory git can never observe.
  *Alternative:* split by filesystem existence (round-5 draft) — rejected by
  measurement too, and this is the one that matters: `.claude/skills/**` is absent
  from a source checkout and present in an installed one, so that rule refuses a
  protected class in one tree and admits it in another. History is checkout-stable;
  the filesystem is not. *Alternative:* record the covered surface in the
  attestation — the honest closure of the weak reading's residual, deferred only
  because it needs a persisted field (OQ-3's shape). Routed as IMP-318, carried as
  R8.
- **D11 — the claim-surface constructor serves `verify` alone.** *Forced by
  RV-307 F-19, bounded by F-24, then narrowed by F-27/F-28/F-29.* Round 4 widened
  it to `validate` to avoid a parallel implementation; round 6 showed the two
  verbs ask different questions — `verify` asks *dirty now*, where canonicalisation
  is mandatory, and `validate` asks *commits since*, where canonicalising against
  today's checkout erases a committed symlink retarget (1 → 0, measured). Sharing
  the surface would have shipped a false negative in the drift count, and could
  not have been built anyway: `collect_all` unions items and shipped, discarding
  the item-directory provenance the constructor requires. `validate` therefore
  keeps its raw seam and gains only D5's own-directory count; its defects and
  `retrieve::git_facts`'s are named, carried as R7 and routed as IMP-317.
  *Alternative:* convert all three now — rejected as scope expansion under cover
  of a bug fix, and now known to need a dataflow change rather than a call-site
  swap. *Alternative:* build `validate` a second, history-stable constructor here
  — rejected: it changes corpus-wide staleness ranking, which is OQ-2's deferred
  decision, not a body-write slice's.
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
- **R7 — two scope consumers keep the raw seam** (RV-307 F-24, widened by F-27).
  `validate`'s staleness check (`src/memory.rs:3413-3421`) and
  `retrieve::git_facts` (`src/retrieve.rs:556-557`) both gate on
  `scope.paths.is_empty()` and pass the array raw, so glob-only memories get no
  drift signal and the canonicalisation/neutralisation fixes do not reach either.
  Bounded deliberately: a correct shared surface for a *historical* question is a
  second constructor, not a reuse of `verify`'s (D11), and building it changes
  corpus-wide staleness ranking, which OQ-2 defers. Routed as IMP-317. **Cost
  corrected** (F-28): adoption needs item-directory provenance threaded through
  `collect_all` and `memory_health_findings` — a dataflow change, not a call-site
  swap. Standing risk: two notions of "scoped drift" coexist and the weaker one
  drives both staleness and ranking.
- **R8 — an attestation does not record what it covered** (RV-307 F-25). Under
  D10's weak reading a `verified_sha` may attest over a proper subset of the
  declared scope, and the shortfall lives on stderr and in `validate` rather than
  on the record — so a downstream consumer cannot tell a full attestation from a
  partial one. 32 active items currently carry at least one unobservable entry.
  Closing it needs a persisted coverage field (OQ-3's shape); routed as IMP-318.
  Mitigated meanwhile by E7's report and the `validate` finding, which make the
  shortfall visible to an operator even though it is invisible to a consumer.

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
| T20b | set `--path-scope` to its **existing** value | does **not** clear — `claim_snapshot` compares, it does not count flags (F-8). Scope, not title: `apply_edit` already guards title, so a title-based test passes with `claim_snapshot` absent and proves nothing (F-17) |
| T21 | edit `status` / `lifespan` / `review_by` / `trust` / `severity`, each alone | does **not** clear (D8's other half) |
| T22 | `body_mode` without `body`, CLI **and** MCP | rejected on both, same message (F-10) |
| T23 | verify on the clean-after-exclusion path while `.git/index.lock` is held | completes — I2 canary; fails if `write-tree` creeps back in |
| T24 | after a successful verify | `git show "$verified_sha:<dir>/memory.md"` equals the on-disk body **byte-for-byte** — I6. Existence (`cat-file -e`) would pass against any stale ancestor blob (F-14) |
| T24b | body **tracked but modified** (not untracked), verify | **refuses** — the case where existence and equality disagree, and the untracked leg does not fire |
| T25 | verify, memory scopes `.doctrine/adr/**`, an ADR under it modified | **refuses** — scoped corpus dirt is claim-relevant (F-6) |
| T26 | claim pathspec construction: absolute-inside-repo, absolute-outside-repo, gitignored-but-tracked, resolves-but-unmatched | normalised / **refuses** / **kept** / reported, per the § 5.2 algorithm (F-6, F-16). The absolute-outside case must assert `verify` **does not abort** — an unfiltered entry makes git `fatal` |
| T27 | verify with a **malformed** scope entry (absolute-outside from this checkout; empty string; tracked symlink whose target is outside) | **refuses**, naming the entry and the reason (D10 malformed, E11, E13). The symlink case is the F-26 collision: it must take the *malformed* branch, not the unobservable one |
| T27b | verify with `scope.commands` and no path scopes | **succeeds** — commands are exempt by kind, not a failed entry (E5) |
| T27c | verify with a **stale** (`src/worktree.rs` — absent, once tracked) and an **unobservable** (`.claude/skills/dispatch-agent/SKILL.md` — absent, never tracked) scope entry | stale **refuses**; unobservable **succeeds** with stderr naming it and `validate` raising it (D10, E7, E12). Both entries are absent from disk, so a test that only checks existence passes with the discriminator absent and proves nothing — the fixture must pin the *history* branch (F-25) |
| T28 | verify **by key** (not uid), tracked memory, `memory.md` modified | **refuses** — I7. Fails if the claim pathspec is built from the key symlink, where all three legs read clean (F-15). Must use the key form; the uid form passes either way |
| T29 | idempotent `--path-scope` | `updated` **is** stamped (`apply_edit` counts it changed) while the verification axis is **not** cleared — the one place the two diverge (F-17) |
| T30 | memory whose `scope.paths` carries `:(exclude)<its own uid dir>`, body modified | **refuses** — I8. The hostile entry is inert under `:(literal)` and cannot subtract the uid directory (F-18). Fails loudly if any entry is interpolated raw |
| T31 | `validate` staleness on a memory scoped **only by globs** | **still not flagged** — the raw seam's `!scope.paths.is_empty()` gate is retained deliberately (D11 narrowed). Pins the *known* gap so R7 cannot be silently closed or silently widened; retargeted by F-27 from an assertion that `validate` was repaired |
| T32 | `validate`'s drift count over a memory scoped by a **tracked symlink that was retargeted in a commit** | counts the retarget — asserting `validate` does **not** canonicalise. Fails if `verify`'s surface leaks into the staleness seam, which would return 0 where the raw path returns 1 (F-27). Equality between the two verbs' surfaces is explicitly **not** asserted; round 5's T32 asserted it and was wrong |
| T33 | memory scoping a **tracked symlink** whose target content changed | `verify` **refuses** — I9. Must probe the *claim* leg specifically, not the source leg, or a dirty tree passes it for the wrong reason. The symlink alone reads clean (F-20) |
| T34 | memory with an **empty-string** scope entry | refuses per T27, and — the discriminating half — the constructed surface is asserted **not** to contain a bare `:(literal)`/`:(glob)`, which would match the whole index (F-23, E11) |
| T35 | `validate` over a corpus containing unobservable scopes | each is raised as a health finding, once per entry, `scope.commands` excluded (E7). Stale entries do not appear — they refuse at `verify` (E12) |
| T36 | memory whose `scope.globs` pattern is rooted at a **slug symlink** (`.doctrine/adr/001-module-layering/**`), target content modified | **refuses** — I9 step 3. Fails if only concrete paths are resolved: `:(glob)<link>/**` matches 0 files and reads clean, `:(glob)<resolved>/**` matches and reads dirty (F-26) |
| T37 | claim-surface construction over an entry that **does not resolve**, in both history states | once-tracked → refuses (E12); never-tracked → attests with a report (E7). One fixture, two git histories, same filesystem state — the discriminating pair for F-25's cut |

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

Twenty-four findings over five rounds. **All disposed `fix-now`; none deferred,
none tolerated.** Full charges, evidence and responses are on the ledger
(`doctrine review show RV-307` / `review-307.toml`).

**This section points; it does not restate** (RV-307 F-22). Every mechanism below
has a normative home, and that home is authoritative. History that re-describes a
rule is how three of these findings survived their own corrections — the narrative
kept asserting what the normative section had already stopped saying. So: what
fired, where it landed, and what it taught. Nothing else.

| # | What fired | Where it landed |
|---|---|---|
| F-1 | false attestation — items live inside the excluded set | D9, I6, § 5.2 |
| F-2 | dead `is_excluded` / stale `source_clean` left by A1 | § 5.1, § 5.3 |
| F-3 | mutation on argument-validation failure | § 5.4 step order |
| F-4 / F-5 | ADR-013 debt; inventory missed REQ-147 | REV-034 |
| F-6 | blanket exclusion hid claim-relevant evidence; then construction rule; then advisory-vs-refusal | D9, D10, § 5.2 |
| F-7 | false capability claim about masters | R5 |
| F-8 | claim-field invalidation decided but unwired | D8, § 5.4 `claim_snapshot` |
| F-9 | closure stated as a range | § 9 closure |
| F-10 | `body_mode`-without-`body` not total | § 5.2 |
| F-11 | probe partition + lock canary unpinned | T17–T19, T23 |
| F-12 | no `stamp_updated` helper exists (responder-raised) | § 5.4 re-stamp |
| F-13 | `--allow-dirty` had nothing to stamp | I4, § 5.4 |
| F-14 | `cat-file -e` proves existence, not identity | I6, T24 |
| F-15 | key symlinks defeat the claim probe | I9, § 5.2, T28 |
| F-16 | two git mechanics stated backwards | § 5.2 table |
| F-17 | T20b could not fail | T20b, T29 |
| F-18 | pathspec injection via `scope.paths` | I8, § 5.2, T30 |
| F-19 | `validate` kept the raw seam | D11 (narrowed by F-27), R7, T31 |
| F-20 | tracked-symlink scopes are blind | I9, § 5.2, T33 |
| F-21 | blanket refusal costs 51 active memories | D10 |
| F-22 | this section restated instead of pointing | this table |
| F-23 | empty entries expand the surface to the index | E11, T34 |
| F-24 | `retrieve` is a third raw consumer | R7, D11 |
| F-25 | D10 attested surfaces missing declared evidence | D10, E12, § 5.2 weak reading, R8, T27c, T37 |
| F-26 | I9 unapplicable to patterns / absent paths; class collision | § 5.2 algorithm, I9, E13, T27, T36 |
| F-27 | canonicalising a historical query erases retarget drift | D11 narrowed, I9 scope, T32 |
| F-28 | `validate`/`retrieve` lack the constructor's `dir` | D11, R7 cost correction |
| F-29 | no `validate` policy for the malformed class | dissolved by D11 narrowing |
| F-30 | R7's insertion overwrote R6's heading | § 8 R6 restored |

**Acquitted on the evidence.** The git pathspec claims the design had marked ✓
without proof. Scratch repositories under Git 2.54.0, all three probes:
exclusion-only pathspec sets behave as § 5.2 asserts. Recorded because a review
that only ever confirms suspicion is not measuring anything.

#### Round 2 — confirmatory pass (same raiser)

Ten of the twelve verified. Two contested (F-6, F-8) and both contests were
correct — one of them an instance of the very pattern the round-1 review had just
diagnosed: F-8 was recorded as a decision *and* a table, and implemented in
neither. Two new findings, both sustained: **F-13** (blocker — the escape hatch
had nothing to stamp) and **F-14**. F-13's root cause is F-1's: the default path
reasoned about, the adjacent path left to inherit machinery built for a different
question.

#### Round 3 — responder self-audit under `/rigour` (pre-handback)

Before returning the round-2 dispositions to the raiser, the four remedies were
put to the question themselves: every git-mechanical claim they rest on was
executed rather than reasoned about (git 2.54.0, this repo + scratch repos). Three
findings, raised by the responder on the same ledger.

**F-15** (blocker — the reference form defeats the claim probe), **F-16** (the
round-2 construction table had two git mechanics backwards) and **F-17** (a test
that could not fail). Two hypotheses were raised and **retired on the evidence**,
recorded so a later pass need not re-derive them: `verify` cannot reach masters or
shipped memories (E2), and `apply_edit` does compare before assigning for the
scalar fields, so F-8's premise holds.

#### Round 4 — external confirmatory pass on rounds 2 and 3

The raiser re-ran every git mechanic independently (git 2.54.0) and confirmed all
of them: symlink blindness, absolute-inside resolution, absolute-outside exit 128,
unmatched exit 0, tracked-file visibility beneath ignored roots. **F-8, F-13, F-15
and F-17 verified.** Three contests and two new charges, all sustained.

Contested **F-16** and **F-14** — in both cases the correction had landed in one
section while another still asserted what it replaced; F-16's stale text was in
§ 10 itself, committed in the very edit that named the pattern for the third time.
Contested **F-6**: the advisory was not a remedy. New: **F-18** (blocker —
pathspec injection; the F-15 fix made the surface *contain* the uid directory
without making that containment *non-negotiable*) and **F-19** (the repair stopped
at `verify`).

#### Round 5 — external pass on the round-4 remediation

Verified F-14 and F-16. Contested F-6, F-18, F-19; raised **F-20** (blocker),
F-21, F-22, F-23, F-24. Every contest and every charge was sustained on
verification.

The round's shape is the same as its predecessors, one layer in. **F-23** is the
sharpest: the F-18 magic-prefix rule was applied unconditionally, which is correct
for hostile input and catastrophic for *empty* input — a bare `:(literal)` matches
the entire index, inverting a narrowed surface into a total one. The responder had
probed the prefix rule and read `exit=1` as "neutralised" when it meant "the
surface is now the whole repo and something in it is dirty": **a non-discriminating
probe, which is F-17's defect committed by the party that raised F-17.** **F-20**
is F-15 one axis over — canonicalisation was applied to the item directory and not
to declared scopes, though git's symlink blindness belongs to pathspecs, not to
item directories. **F-24** found the third scope consumer that D11 had claimed did
not exist.

**F-21 is the one that changed a decision rather than a mechanism.** A census of
the real corpus — not a sample — established the round-4 blanket refusal's cost,
and D10 was re-cut on that measurement. Round 6 re-cut it again on a corrected
one (F-25); the current rule and its cost are § 5.2's, not this paragraph's. What
survives as history is the practice: a decision of this shape is taken on a census
of the whole corpus, and the measurement is what makes it a decision rather than
an accident.

**What the first five rounds said about this design.** The recurring failure was
never the mechanism — it was reasoning about the principal path and letting
adjacent paths inherit assumptions that no longer held: the escape hatch (F-13),
the bypass path (A4), the hand-edit path (D5), the no-op write (A2), the scope
entry that isn't repo-relative (F-6 round 2), **the reference form** (F-15), **the
test that pins the guarded field instead of the unguarded one** (F-17), **the
sibling verb** (F-19), **the input treated as syntax** (F-18), **the empty input**
(F-23), and **the declared scope that is also a symlink** (F-20). The design is
now specified at those edges rather than at the centre alone, which is why the
test matrix roughly tripled.

Three lessons are worth carrying past this slice.

**A remedy can be right in its conclusion and wrong in every reason it gives**
(F-16). That survives review, because reviewers check conclusions. Only executing
the commands caught it.

**A probe that cannot distinguish the two outcomes proves nothing** (F-17, F-23).
The design raised F-17 against its own test matrix, then committed the identical
error one round later in a shell probe — reading a pass as evidence of the
mechanism it was meant to isolate. Registering the *falsifier* before running the
check, not after, is the only thing that catches this.

**Sweeping is not a step you perform once** (F-14, F-16, F-22). § 10 diagnosed
incomplete integration after round 1, and every round since found another
instance — round 4's inside the paragraph that had just restated the diagnosis,
round 5's in the section itself. Naming a failure mode does not immunise you
against it, and a design doc that narrates its own history accumulates exactly the
stale text that hides the next finding. The remedy is structural, not diligence:
**normative sections are the single source; history points at them and states
nothing a reader could act on.** § 10 is now a table of what fired and where it
landed, for that reason.

#### Round 6 — external pass on the round-5 remediation

Verified F-18, F-21, F-23. Contested F-6, F-19, F-20, F-22, F-24; raised **F-25**
and **F-27** (blockers), F-26, F-28, F-29 (majors) and F-30. Every charge was
sustained on independent verification.

The round moved the failure up a level. Rounds 1–5 found remedies written against
a finding rather than against the invariant it instantiates; round 6 found the
opposite error in the fix for that — **an invariant generalised past the domain it
holds in**. I9 ("nothing reaches the claim surface uncanonicalised") was the
right rule for `verify` and was promoted to a universal, and D11 then handed that
universal to `validate`, whose question is historical. Canonicalising resolves
against today's checkout; a committed symlink retarget then counts 1 through the
declared path and 0 through the resolved one (F-27). The remedy for
over-specificity was over-generality, and it produced a false negative in the one
verb that was supposed to catch what `verify` misses.

**F-25 changed a decision for the second time, and the measurement changed with
it.** The round-5 cut split non-contribution by whether the path exists on disk.
Re-measuring under the actual construction algorithm showed that boundary falls in
the wrong place: `.claude/skills/**` is absent from a source checkout and present
in an installed one, so an existence test would refuse a protected class of
harness memory in one tree and admit it in another. History is checkout-stable —
30 of the 43 non-resolving declarations were tracked once (moved source paths, a
real defect), 13 never were. D10 now refuses where refusal is *actionable*, at a
measured cost of 4 currently-stamped active items rather than 36.

F-26 and F-29 were dissolved structurally rather than patched: the scope rule is
now an ordered algorithm whose classes are decided by probe outcome, and
`validate` no longer builds a surface at all, so it needs no policy for one.

**Where this leaves the design.** Six rounds, thirty findings, seven blockers. The
rate has not decayed, and round 6's findings were not polish — two of them
refuted a decision (D11) and a class boundary (D10) rather than correcting an
edge. That is recorded here rather than smoothed over. What has changed is the
*kind* of defect: the remaining surface is no longer "which adjacent path inherits
a dead assumption" but "does each rule hold across the domain it is stated over",
which is a smaller and more checkable question. The gate for `/plan` is the
ledger's, not the author's confidence.
