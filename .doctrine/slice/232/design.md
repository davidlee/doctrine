# Design SL-232: Corpus-aware memory verify gate

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-232, REQ-147, ADR-004); doc-local refs bare — OQ-2 (§6), D9 (§7),
     R7 (§8), T25 (§9). -->

## 0. Provenance and status — READ FIRST

> ## ⚠ STALE AS OF 2026-07-27 — THIS DOCUMENT CONTRADICTS `slice-232.md`
>
> A design round has taken decisions that this text does not carry. **Where this
> document and `slice-232.md` / `notes.md` § Harvest disagree, they are right and
> this is wrong.** It is retained un-rewritten so the replacement is authored
> against the reasoning that failed, not from scratch — the same discipline § 5.5
> already applies to I9 and E7.
>
> **Do not implement from this document. Do not review it as canon.**
>
> | Section | Status |
> |---|---|
> | § 5.2 the ordered algorithm | **replaced wholesale** by DEC-053 (index-first). Steps 1–7, the shape rule, the whole-component prefix rule, and the `realpath` oracle are all retired. |
> | § 5.4 D11 | **falsified** — "`validate` keeps its existing raw seam" cannot survive objective 7 (DEC-054). Its four-defect enumeration is incomplete: the `None`-swallow is a fifth and the only non-conformant one. |
> | § 5.5 I9 | falsified as written *and* superseded: it must be re-expressed as an **outcome** property, not a pre-emission one. |
> | § 5.5 E13 | basis dissolved — aborts are now prevented lexically, so "git aborts, so there is no verdict" no longer justifies a refusal. Re-justify or fold into E7. |
> | § 6 OQ-2, OQ-A | **answered** (`yes` / `no`). See QUE-175 and `slice-232.md`. |
> | § 8 R-A | discharged in method; narrowed to new risks R-E / R-F. R-G added. |
> | § 9 the T-matrix | **pins an algorithm that no longer exists.** Needs rebuilding, not editing. |
>
> Authoritative now: **`slice-232.md`** (scope, seven objectives), **`notes.md`
> § Harvest** (produced / learned / open), **DEC-053**, **DEC-054**, **QUE-175**,
> and **`probes/`** — the executable evidence, re-runnable, with falsifiers
> registered in-header.
>
> Still true and still load-bearing below: § 1–§ 4 (the problem, current state,
> forces, principles), the F-18 magic-prefix rule and I8, F-13's `--allow-dirty`
> re-capture, F-15's uid-directory base, and DEC-020's ruling that
> non-contribution is reported and never classified.

**This design is inherited, not authored here, and it is NOT locked.**

Split out of SL-230 by **DEC-027** at RV-307 round 8. The text below is the
gate half of SL-230's design, carried over verbatim so that eight rounds of
adversarial review — the measured censuses, the executed git evidence, the
refuted alternatives — are not thrown away and re-derived. Every `RV-307 F-NN`
citation refers to that ledger, which stays attached to SL-230 (append-only; it
reviewed *that* document).

**Two blockers are open and inherited. They are design problems, not text
defects, and they must be solved before this design can lock:**

- **RV-307 F-36** — DEC-020 requires `validate` to raise every non-contributing
  scope entry, but D11 leaves `validate` with no contribution probe at all: it
  keeps a historical, `scope.paths`-gated seam that cannot implement T35.
  Supplying one is an undesigned second per-entry git path, plus a corpus-wide
  continuation policy for a memory whose surface is malformed (F-29's shape).
  **§ 5.4's D11 text and § 5.5's E7 both currently assert a consequence with no
  mechanism.**
- **RV-307 F-37** — the premise that a non-resolving entry contributes nothing is
  **false**. Reproduced on git 2.54.0 by three routes: `missing/../link` (an
  unresolved `..` alias), a sparse checkout where the tracked link is absent, and
  a `scope.paths` literal whose filename contains `*` (the shape rule at § 5.2
  step 2 reads the star as a wildcard and skips whole-path resolution). Each
  contributes while bypassing canonicalisation, and each reads clean against a
  dirty target. **§ 5.2 step 4, step 5 and I9 are wrong as written** — this is
  the second failed totality claim over path resolution (F-26 was the first), so
  the replacement rule needs its reachable shapes enumerated and probed, not
  reasoned about.

Two majors are also open: **F-38** (NUL/newline scope entries escape E11/E13 —
NUL cannot cross the argv boundary at all, so it is neither exit 0, 1, nor 128)
and **F-39**'s gate limb (code-only wording survives at D9; IMP-317's *title*
still names the rejected shared constructor).

Route: `/design` on this slice, starting from those four. Do not treat any
totality or stability claim below as settled merely because it survived to here —
this slice's dominant cost driver was exactly that error, eight times.

## 1. Design Problem

`memory verify` refuses to attest against a dirty working tree. Doing doctrine
work means the authored corpus is almost always dirty, so the common case is
self-inflicted: you cannot attest a memory because of the corpus edit you just
made — or, as observed live during the SL-230 design round, because of an
*unrelated* backlog file another agent left uncommitted. In practice agents hit
the refusal and reach for `git stash` rather than `--allow-dirty`, which is
undiscoverable and postdates the filing of IMP-221.

The naive remedy — ignore doctrine's own authored trees — is **wrong**, and
RV-307 F-1/F-6 proved it on this corpus. Memory items live at
`.doctrine/memory/items/<key>/`, so a blanket exclusion removes *the memory being
verified* and stamps a commit that provably lacks the attested body. And **81
items declare `.doctrine/**` scopes**, so an ADR a memory explicitly names is
claim evidence exactly as `src/` is.

So the gate must be **claim-aware**: exclude unclaimed dirt, never claimed
evidence. Constructing that per-memory surface is the whole difficulty of this
slice, and it is where all 29 of the inherited findings live.

## 2. Current State

| Surface | Behaviour | Site |
|---|---|---|
| `memory verify` | refuses on any dirty tree unless `--allow-dirty` | `src/memory.rs:3382-3390` |
| `capture()` | blanks the commit oid whenever the tree is dirty; yields a `checkout_state_id` hash instead | `src/git.rs` |
| Verification axis | `[review].verification_state`, `[review].reviewed`, `[git].verified_sha` — written **only** by `stamp_verification` | `src/memory.rs:3350-3362` |
| `memory validate` | staleness = commits touching **scoped paths** since `verified_sha`; gated on `!scope.paths.is_empty()`, array passed raw | `src/memory.rs:3413-3424` |
| `retrieve::git_facts` | same raw scope seam, feeding ranking | `src/retrieve.rs:556-557` |
| `collect_all` | unions `items/` and `shipped/` into one `Vec<Memory>`, erasing which root supplied each row | `src/memory.rs:2826-2834` |
| `fsutil::safe_join` | plain `tree_root.join(rel)` — **no canonicalisation** | `src/fsutil.rs:20-33` |

**`capture()` has exactly three callers** — `src/retrieve.rs:532` (read path),
`src/memory.rs:1708` (`record`), `src/memory.rs:3382` (`verify`). Two of the three
would be damaged by unconditional leniency, which is why the exclusion is a
parameter and not a change to `capture()`.

## 3. Forces & Constraints

| Authority | Constraint |
|---|---|
| **SPEC-007** | Asserts verify attests "against a clean working tree, refusing a dirty one" — **three** sites: `spec-007.toml:22`, `spec-007.md:132-133`, and **`REQ-147`**, whose *title is the retired contract verbatim* and which is an active member of SPEC-007. Already false since `--allow-dirty`; this slice changes it further. Amended by **REV-034** (moved here from SL-230 by DEC-027). |
| **ADR-013** ✓ | Governance→work dependency routes through a Revision. `SL-232 needs REV-034` is authored. |
| **ADR-001** ✓ | `corpus_guard` = leaf, `git` = leaf, `memory` = command. Downward edges only. |
| **POL-002** | The exclusion set must rest on doctrine-owned contracts, never host layout. `.doctrine` and `MEMORY_MASTERS_DIR` are platform-owned constants, so exclusion is legal — but `memory/` exists only in this repo, so guard it on existence rather than assume it. |
| **STD-001** | Named constants, not path literals. Satisfied by reuse: `DOCTRINE_PATHSPEC` already exists (`src/corpus_guard.rs:43`). |
| **SL-008 D6** | `thread_expiry` is reviewed canon — not loosened. |
| **DEC-020** | Non-contribution is reported and attested over, never classified. Three derived instruments were refuted; a fourth is not a finding. The stable answer is a *declared* boundary — this slice's objective 3. |
| **SL-230** | Owns the body-write seam and attestation invalidation (D1/D2/D4/D5/D7/D8). This design must not re-open them. Note the live coupling: SL-230 ships invalidation *without* this relaxation, so its R4 runs unmitigated until this lands (DEC-027). |

## 4. Guiding Principles

- **The frame tells the truth.** `capture()` reports the literal state of the
  tree. Leniency is a *policy* applied by one consumer, never baked into the
  measurement.
- **Attestation is about the claim — the whole claim, and only the claim.** A
  memory attests that its body is true of what it declares. Dirt the memory does
  not declare says nothing about that; a change to a path it *does* declare says
  everything. Governance dirt is not exempt by being governance (RV-307
  F-6/F-33, T25). The exclusion is claim-aware, never blanket.
- **A tool property is a claim needing a falsifier, not a premise.** "Stable",
  "total", "deterministic" — each must be probed by varying the local state the
  instrument reads. Measuring that a discriminator *works* is not evidence that
  it is *stable*. This is the named dominant cost driver of the eight rounds
  behind this text, and F-37 is its most recent instance.
- **Scope entries are untrusted data, never syntax.** SPEC-007 § Concerns treats
  stored memory text as hostile input.

## 5. Proposed Design

### 5.1 System Model

```
command tier   memory.rs ──────────────┬──────────── run_verify
                                       │             composes the pathspec sets (policy)
leaf tier      corpus_guard.rs  DOCTRINE_PATHSPEC  (existing constant, STD-001)
               git.rs     dirty_under(root, pathspecs) -> Result<bool>     ← the primitive
                          capture_with(root, excludes) -> Result<Frame>    ← delegates to it
                          capture(root) = capture_with(root, &[])          ← unchanged behaviour
```

Two changes at their correct altitudes: one parameterised dirtiness primitive at
leaf, and policy composition at command. **There is exactly one dirtiness
measurement in the design** — `dirty_under` — used twice by `verify` with
different pathspec sets. No path predicate exists at leaf and no second probe
exists anywhere; both were specified by earlier drafts and are deleted (RV-307
F-2).

*Open (F-36):* `validate` needs a contribution probe and this model does not
give it one. Whatever supplies it is a third element of this diagram.

### 5.2 Interfaces & Contracts

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

That is F-1 restored through the reference form, on the mainstream path: SL-230's
D4/D8 clear the axis on every claim edit, so *edit → re-verify* is the flow this slice
creates, and agents address memories **by key** (the boot snapshot and
`/retrieve-memory` both emit keys). T8's fresh-`record` case survives only by
luck — the untracked *symlink* trips the untracked leg — so T8 passing proves
nothing here. `validate`'s own-directory drift count (SL-230 D5) takes the uid dir
for the identical reason.

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
   **whole-component** prefix — the text up to the last `/` *before* the first
   wildcard character — and re-appends the remainder. The prefix must end at a
   separator, not at the wildcard (RV-307 F-32): splitting at the wildcard
   *character* yields `foo` for `foo*/bar`, which resolves to nothing, while the
   entry itself matches tracked `foobar/bar` (verified, git 2.54.0:
   `ls-files --error-unmatch -- ':(glob)foo*/bar'` → exit 0, `foobar/bar`). **A
   textual wildcard-free prefix is not a path prefix**, and treating it as one
   sends a contributing glob down the non-resolving branch. Under the corrected
   rule `foo*/bar` has an *empty* whole-component prefix and is emitted unchanged,
   where it matches. **A pattern whose whole-component prefix is empty resolves to
   nothing and is emitted unchanged** —
   `**/.gitignore` (one live corpus entry) is rooted at no directory, so there is
   no symlink for git to be blind to. This is *not* a bare magic prefix and does
   not engage E11: the emitted form is `:(glob)**/.gitignore`, which matches
   exactly the files it names (verified, git 2.54.0: 2 of 3 tracked files, where a
   bare `:(glob)` returns all 3). Stated because step 3 is otherwise silent on it
   and the case is reachable today. Resolution is necessary, not cosmetic:
   `:(glob)<slug-symlink>/**` matches **nothing** and reports clean against a
   modified target, exactly as the literal form did in F-20 — verified, git 2.54.0:
   0 files via the link, 1 via the resolved prefix, `diff-index` exit 0 versus
   exit 1. No glob declaration in this corpus is currently symlink-rooted, so this
   closes a latent hole at zero migration cost.
4. **Emit — unless the emitted form would abort the probe.**
   - **Resolves inside the repo** → emit it, repo-relative and magic-prefixed.
     Step 5 decides whether it contributes. (Normalising an absolute-inside entry
     is hygiene, not a correctness fix — git converts absolute pathspecs itself —
     but it keeps the emitted pathspec legible and stable across checkouts.)
   - **Does not resolve** → emit it as written, magic-prefixed. There is nothing
     to canonicalise, and step 5 finds that it matches nothing. No history is
     consulted and no classification is attempted (DEC-020).
   - **The emitted form is outside the repository** → **malformed, refuse**
     (E13). Git does not return a verdict on such a pathspec, it *aborts* —
     `exit 128` — and would take `verify` down with it. This is F-26's collision
     cut in favour of the probe: where the entry points is not the test, what it
     does to the probe is. It is **one** class covering both a resolution that
     lands outside and a non-resolving outside-shaped string, which is F-32's
     second limb — under the round-7 algorithm the latter was unclassified and
     reached the history probe, which aborted. Verified, git 2.54.0:

     | Emitted form | `ls-files --error-unmatch` |
     |---|---|
     | `:(literal)nonexistent/inside.txt` | exit 1 — unmatched: **a verdict** |
     | `:(literal)/tmp/no-such-absolute` | **exit 128** — `is outside repository` |
     | `:(literal)../outside-no-such` | **exit 128** — `is outside repository` |
     | `:(glob)/tmp/no-such-*/**` | **exit 128** — `Invalid path` |

     A non-resolving entry *inside* the repo is survivable and a non-resolving
     entry *outside* it is not. That, and nothing about the entry's provenance,
     is what separates malformed from merely non-contributing.
5. **Contribution.** `git ls-files --error-unmatch`, **per entry** so the report
   can name the entry rather than the set. Exit 0 → **observable**: real claim
   evidence, and it must be clean. Exit 1 → **non-contributing**: reported on
   stderr, raised by `validate`, and attested over. No further discrimination is
   attempted — that is DEC-020, and the reason is below. The observable set is
   what `dirty_under` then probes.
6. `scope.commands` never enters: not path-shaped, exempt by kind (E5).
7. Nothing left → the uid directory alone (E6). It is unconditional, so the claim
   surface is never empty.

**Why the algorithm stops at step 5 and does not sort the non-contributing**
(DEC-020, RV-307 F-25/F-31). The distinction the design kept reaching for — a
*genuine defect* the author should fix, versus evidence *git can never see* — was
drawn three times and refuted three times, and always for the same reason: every
instrument proposed to decide it reads **local repository state**.

| Instrument | Refuted by | Because |
|---|---|---|
| blanket refusal (no boundary) | F-21 | 36 active items stop verifying |
| filesystem existence | F-25 | checkout-dependent — `.claude/skills/**` is absent from a source checkout, present in an installed one |
| `git rev-list --all` history | F-31 | ref-set-dependent — deleting a branch flips a once-tracked path from refuse to attest while its commit object survives |

The third is the one that settled it. `--all` means *reachable from this clone's
current refs*, not *ever tracked* (verified, git 2.54.0: with the branch present
`rev-list --all --max-count=1 -- <path>` returns the commit; after `git branch -D`
it returns empty, while `cat-file -e` on that commit still exits 0). A shallow
clone, a pruned repo, a fresh clone and a dispatch worktree legitimately hold
different answers, because **git's view is inherently local** — so a fourth
derived instrument would fail exactly as the first three did. The property being
assumed of each instrument was *stability*, and stability is a claim needing a
falsifier, not a premise; measuring that a discriminator **works** is not evidence
that it is **stable**.

So `verify` does not classify. It attests over every non-contributing entry,
names each on stderr, and `validate` raises them. The question *should
non-contribution refuse, and on which entries* leaves this slice for its own
(DEC-020). The stable answer is known and deliberately not built here: a boundary
that survives cloning must be **declared** on the record, not derived from it — a
schema change of the same shape and cost as IMP-318 and QUE-173/OQ-3, which is
why all three are scoped together rather than one being smuggled in.

**I9 is total** because every entry that *contributes* has been resolved. The two
shapes that cannot be resolved are handled without an uncanonicalised pathspec
ever bearing evidence: an outside target is refused at step 4, and an absent path
is emitted inert — it matches nothing, so it contributes nothing to canonicalise.

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

**Non-contribution is reported, never classified** (D10, DEC-020, RV-307
F-6/F-21/F-25/F-31). Git does not fail a pathspec that matches nothing (absent
`--error-unmatch` — `git diff HEAD -- src/nope.rs` exits 0 silently), so a dropped
or unmatched scope entry shrinks the claim surface *silently*. Making it audible
is the whole remedy; sorting the silence into kinds is what this slice does not do.

| Outcome | Example, from the real corpus | Response |
|---|---|---|
| **Probe-aborting** — empty, or an emitted form outside the repo | `""`, `"/etc/passwd"`, a tracked symlink whose target is outside, `"../gone"` | **refuse.** Git aborts rather than answering, so there is no attestation to make (E11, E13) |
| **Non-contributing** — emitted, matches no tracked file | `src/worktree.rs` (moved), `.claude/skills/**`, `.harness/probe/**`, `.doctrine/state/slice/` | **report on stderr + `validate` finding**, then attest (E7) |
| **Observable** — matches at least one tracked file | anything tracked | **must be clean, or refuse** — this is the claim probe proper |

The first row is a **mechanical necessity, not a judgement about the memory**:
the probe cannot run, so no verdict of any kind is available. That is what keeps
the shrunk D10 coherent rather than arbitrary — the only refusal that survives is
the one where refusing is the sole option. Every entry that git *can* answer for,
it answers for, and the answer is binary: contributes or does not.

`validate` raises the non-contributing entries as a corpus-health finding so the
overclaim is aggregated rather than emitted as per-attestation noise.
`scope.commands` is *exempt by kind*, not a failed entry: it is structurally
non-path (E5), a property of the schema rather than of this tree.

***What `verified_sha` asserts, stated rather than implied*** (RV-307 F-25/F-33).
The round-3 draft answered non-contribution with a stderr advisory and § 5.2
rejected it — *a warning does not make an unobservable claim committed* — then
adopted that same advisory for two of four classes. F-25's charge was that
condemning a mechanism in one paragraph and relying on it in the next is not a
position but an inconsistency. Sustained; the inconsistency is resolved by
adopting the advisory **explicitly**, with the reading stated outright and the
residual routed:

> A `verified_sha` asserts that **everything git could observe about this claim
> was committed and unchanged at that commit** — not that every declared entry was
> observed. A memory declaring a path git does not track carries an attestation
> over a proper subset of what it declares.

That is the weak reading, and under DEC-020 it is the design's **only** reading.
The strong reading — every declared entry observed, or no stamp — is the round-4
blanket refusal, and measurement falsified it: it treats a memory about harness
behaviour as defective for scoping the harness, and the remedy it prescribes (edit
the scope) is itself a claim edit, so it clears the axis and guarantees the memory
is unverified in exchange for making it verifiable. The blanket rule costs 36
active items; DEC-020 costs **zero** — no memory loses a stamp. Nothing in this
design may assert the strong reading anywhere else (F-33): the two contracts
cannot coexist as separate normative readings, because a planner could implement
either and remain textually compliant.

The weak reading's residual is real and is **not** closed here: a consumer of the
stamp cannot distinguish a full attestation from a partial one, because the
shortfall lives on stderr and in `validate`, not on the record. Closing it means
persisting the covered surface — a new field, i.e. a schema change and its own
slice, exactly OQ-3's shape. Routed as **IMP-318** and carried as **R8**, on the
R5 precedent: state the gap, do not paper it.

**The seam is the refusal path in `run_verify`, not a side-channel.** Surface
construction returns the observable pathspec set *and* the list of
non-contributing entries; the command tier refuses on any **probe-aborting**
member before it probes anything, and passes the remainder to the reporter.
Detection is `git ls-files --error-unmatch` per entry — the same plumbing, no new
dependency, and **no second git query**: the history probe an earlier draft
required here is gone with DEC-020. E7 is the wording of that report; E11 and E13
the refusals.

This is the correction for RV-307 F-1 and F-6. Excluding `.doctrine/**` wholesale
excluded the memory *being verified* — items live at
`.doctrine/memory/items/<key>/` — so `verify` would have stamped a HEAD that
provably lacked the attested body, and would have ignored a modified
`.doctrine/adr/001/layering.toml` that a memory explicitly scopes. **81 items in
this corpus carry `.doctrine/**` scopes.** Doctrine's ownership of the path
constant makes the exclusion legal under POL-002; it does not make the excluded
evidence irrelevant.

### 5.3 Data, State & Ownership

No schema change in objectives 1–2; `dirty_under` returns a value and owns no
state.

`MEMORY_SHIPPED_DIR` (`.doctrine/memory/shipped`) and `MEMORY_ITEMS_DIR`
(`.doctrine/memory/items`) are both *under* `.doctrine`, so one exclusion root
covers them. Only `MEMORY_MASTERS_DIR` (`memory`, repo-root) sits outside — and
it is contributed only when the directory exists (E4). The item under attestation
is then re-admitted as a *positive* pathspec in `claim_pathspecs`; the two sets are
independent, so no re-inclusion magic is needed (git offers none).

**Objective 3 is a deliberate exception.** The declared-boundary signal, and
IMP-318's attested-coverage field, *are* schema changes — that is precisely why
DEC-020 deferred them out of SL-230 and why they belong here. Their shape is
open (OQ-A).

### 5.4 Lifecycle, Operations & Dynamics

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
`checkout_state_id`. What must be committed is the memory's own body and every
declared path **git can observe** — the weak reading of § 5.2, which is the only
reading this design carries (RV-307 F-33). A declared path git does not track is
reported, not required.

**This costs the `record` → `verify` convenience, deliberately** (RV-307 F-1). A
freshly recorded memory's directory is untracked, so `verify` now refuses until it
is committed. The alternative was a `verified_sha` naming a commit that provably
did not contain the attested prose — demonstrated in a scratch repo, where
`git cat-file -e "$verified_sha:.../memory.md"` exited 128 while the drift count
that was supposed to catch it printed 0, then and forever. A worthless stamp is
worse than an extra `git commit`. The refusal says which of the two questions
failed, and what to do about it.

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

**So `validate` keeps its existing raw seam and gains only SL-230 D5's own-directory
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

SL-230 D5's own-directory count is unaffected by F-27: the uid directory is reached by
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
- **I6** — a successful attestation's `verified_sha` **contains the attested
  body**. The point of the claim probe; pinned by T24 as **byte equality** —
  `git show "$verified_sha:<uid-dir>/memory.md"` equals the on-disk body.
  Existence (`cat-file -e`) is *not* the assertion and never was sufficient: any
  stale ancestor blob at that path satisfies it (F-14).
- **I7** — the claim surface names **real tracked files**, never a symlink
  standing in for them: it is rooted at the canonicalised uid directory (§ 5.2,
  F-15). Pinned by T28, which verifies *by key* against a modified tracked body
  and requires a refusal.
- **I8** — nothing a memory *declares* can subtract from what it is *measured
  against*. Scope entries are emitted magic-prefixed (§ 5.2, F-18), so the uid
  directory is present in the claim surface unconditionally. Pinned by T30.
- **I9** — nothing **bearing evidence** on `verify`'s claim surface is
  uncanonicalised: the uid directory and every declared scope alike is
  symlink-resolved before it becomes a pathspec — a concrete path whole, a pattern
  by its longest **whole-component** prefix, the text up to the last `/` before
  the first wildcard (§ 5.2, F-15/F-20/F-26/F-32). Total by construction over the
  entries that matter: an outside target is refused (E13) and an absent path is
  emitted inert, matching nothing, so it carries no evidence to canonicalise.
  **Scoped to `verify` deliberately** (F-27):
  canonicalisation is required for a dirtiness question and wrong for a historical
  one, so it must not reach `validate`'s drift count (D11). Pinned by T28, T33,
  T36, T37.
- **E2** — masters and shipped never reach the gate at all: `run_verify` resolves
  through `items_root` alone (`src/memory.rs:3378-3380`), so `verify` is
  items-only by construction and no master or shipped memory is addressable by it.
  (Repo-empty masters additionally carry `anchor_kind = None`, never
  `CheckoutState`.) Checked because the claim probe made the question live —
  `.doctrine/memory/shipped/` is gitignored (`.gitignore:44`, 0 tracked files), so
  had `verify` reached it, its whole claim surface would have been untracked and
  the probe would have passed vacuously. It cannot. Nothing to do; recorded so the
  next pass need not re-derive it.
- **E4** — `memory/` absent (every client project) → that exclusion root is
  simply not contributed.
- **E5** — `scope.commands` is not path-shaped and contributes no pathspec to
  `claim_pathspecs`; a memory scoped only by command has just its item directory
  in the claim surface.
- **E6** — a memory with an empty scope has a claim surface of exactly its own
  item directory. Still meaningful: the body must be committed.
- **E7** — **every** non-contributing scope entry is **reported on stderr at
  verify time and raised by `validate` as a corpus-health finding** (D10,
  DEC-020). Silent narrowing of the claim surface is a false attestation reached
  quietly; the operator is told when the evidence surface is smaller than the
  declared scope, and the corpus-wide verb makes the backlog visible rather than
  per-attestation noise. `scope.commands` is exempt by kind and is not reported as
  a defect (E5). *Narrowed by F-25, then widened back by DEC-020:* there is no
  stale class to carve out — E7 covers the whole of non-contribution, which is
  what makes it the design's single answer to it.
- **E8** — a **gitignored** scope entry is *kept*, not dropped (§ 5.2, F-16).
  Ignore rules do not bind tracked files, so dropping would discard real evidence
  from a force-added path; keeping is inert when the path is genuinely untracked.
- **E9** — the absolute-inside / absolute-outside classification is a property of
  the **checkout location**, not of the scope string. The four items scoped
  `"/workspace/doctrine/…"` resolve inside the primary tree and outside a linked
  worktree, so the same memory has a narrower claim surface when verified from a
  dispatch worktree — announced by E7 rather than silent.
- **E11** — an empty or whitespace-only scope entry is **dropped before emission
  and refused**, never prefixed: a bare `:(literal)`/`:(glob)` matches the entire
  index (F-23), and raw it aborts the probe. No bare magic prefix is ever emitted.
- ~~**E12**~~ — **withdrawn by DEC-020.** It refused a *stale* entry, discriminated
  by `git rev-list --all`; F-31 showed that discriminator is ref-set-dependent, and
  no derived instrument replaces it (§ 5.2). Stale entries now take E7 like every
  other non-contributing entry. Retained as a struck id because criteria ids are
  immutable — E12 is never reused.
- **E13** — a scope entry whose **emitted form is outside the repository** is
  malformed and refuses: an absolute literal, a tracked symlink pointing out
  (F-26), or a non-resolving outside-shaped string such as `"../gone"` (F-32).
  One class, because the test is what the entry does to the probe — git aborts
  `exit 128` instead of returning a verdict — not where it points and not whether
  it resolves.

**Status of I9 and E7 (F-37, F-36).** I9 is **falsified as written** — F-37's
three routes contribute without being canonicalised. E7 asserts a `validate`
consequence for which D11 provides no mechanism. Both are retained here rather
than deleted so the replacement is written against the reasoning that failed,
not from scratch; neither may be treated as holding.

*Criteria ids are immutable* — **E12** is struck and never reused (withdrawn by
DEC-020, above). **E10** was never minted; the gap is inherited, not a deletion.

## 6. Open Questions & Unknowns

- **OQ-2** — should own-directory drift feed *retrieve-side* `staleness`, not just
  `validate`? Load-bearing on whether either `validate` or `retrieve::git_facts`
  leaves the raw seam (RV-307 F-24/F-27). Both are the same corpus-wide
  reclassification by different routes. Routed as **IMP-317**, which closes
  `wont-do` if the answer is "no" — in which case R7 is restated as intended
  rather than provisional. Tracked as **QUE-175**. **This slice must answer it**;
  SL-230 could only defer it.
- **OQ-3** — a body digest stamped at verify time would make invalidation
  git-independent and path-independent, covering uncommitted edits and masters
  (which have no `verified_sha`). Needs a new persisted field. Tracked as
  **QUE-173**; scope with OQ-6 and IMP-318.
- **OQ-5** — should the *source* leg narrow to the memory's declared scopes too,
  so a dirty `src/` file no memory claims against stops blocking? Raised by the
  F-6 disposition and deliberately not taken in SL-230: it changes what the
  anchor means for every memory at once, where the claim probe only adds a check.
  I3 is preserved as-is. Reopenable here.
- **OQ-6** — **should non-contribution ever refuse, and on which entries?**
  Deferred out of SL-230 by DEC-020 after three derived instruments were refuted.
  The answer that survives cloning is a **declared** boundary — a signal on the
  record marking evidence git is not expected to observe — not one derived from
  local repository state. A persisted field, so it shares OQ-3's and IMP-318's
  shape and cost. **This is objective 3 and the reason the slice exists.**
- **OQ-A** — do the declared boundary, IMP-318's attested coverage, and QUE-173's
  digest land as **one** schema change or as sequenced ones? DEC-020 argues they
  are one shape; that is an argument, not a measurement.
- **OQ-B** — what is `validate`'s contribution mechanism (F-36), and what is its
  continuation policy when one memory's surface is probe-aborting (F-29's shape)?

## 7. Decisions, Rationale & Alternatives

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
- **D10 — non-contribution is reported and attested over; it is not classified.**
  *Settled by **DEC-020** (user ruling, RV-307 round 7) after being revised four
  times: forced by F-6 round 3, bounded by F-21's census, re-cut by F-25, refuted
  again by F-31.* `verify` attests over every non-contributing scope entry, names
  each on stderr, and `validate` raises them (E7). The **only** refusal is the
  probe-aborting entry — empty (E11) or an emitted form outside the repository
  (E13) — because there git returns no verdict at all. That is a mechanical
  necessity, not a judgement about the memory, which is what makes this cut
  principled rather than merely smaller.
  *Alternative:* warn and proceed, silently (round-3 draft) — rejected then and
  not restored now: the advisory is adopted **explicitly**, with the weak reading
  stated (§ 5.2) and the residual routed, which is a position rather than the
  inconsistency F-25 charged. *Alternative:* refuse on everything (round-4) —
  rejected on **measurement**: 36 active items stop verifying, including a class
  of harness memory git can never observe. *Alternative:* split by filesystem
  existence (round-5) — rejected: `.claude/skills/**` is absent from a source
  checkout and present in an installed one, so the rule refuses a protected class
  in one tree and admits it in another (F-25). *Alternative:* split by
  `git rev-list --all` history (round-6/7) — rejected: `--all` means *reachable
  from this clone's refs*, so deleting a branch flips an entry from refuse to
  attest while its commit survives (F-31). **The pattern, not three unlucky
  choices:** every derived instrument reads local repository state, and a shallow
  clone, a pruned repo and a dispatch worktree legitimately disagree — so a fourth
  would fail identically. *Alternative:* a **declared** boundary on the record —
  the answer that does survive cloning, and deferred only because it is a schema
  change (OQ-6, with OQ-3 and IMP-318). *Alternative:* record the covered surface
  in the attestation — the honest closure of the weak reading's residual, same
  deferral. Routed as IMP-318, carried as R8.
  **Cost of the chosen cut: zero** — no memory loses a stamp, where the round-6
  cut cost 4 and the round-4 blanket cost 36.
- **D11 — the claim-surface constructor serves `verify` alone.** *Forced by
  RV-307 F-19, bounded by F-24, then narrowed by F-27/F-28/F-29.* Round 4 widened
  it to `validate` to avoid a parallel implementation; round 6 showed the two
  verbs ask different questions — `verify` asks *dirty now*, where canonicalisation
  is mandatory, and `validate` asks *commits since*, where canonicalising against
  today's checkout erases a committed symlink retarget (1 → 0, measured). Sharing
  the surface would have shipped a false negative in the drift count, and could
  not have been built anyway: `collect_all` unions items and shipped, discarding
  the item-directory provenance the constructor requires. `validate` therefore
  keeps its raw seam and gains only SL-230 D5's own-directory count; its defects and
  `retrieve::git_facts`'s are named, carried as R7 and routed as IMP-317.
  *Alternative:* convert all three now — rejected as scope expansion under cover
  of a bug fix, and now known to need a dataflow change rather than a call-site
  swap. *Alternative:* build `validate` a second, history-stable constructor here
  — rejected: it changes corpus-wide staleness ranking, which is OQ-2's deferred
  decision, not a body-write slice's.

*Inherited, and each still open to attack.* D9's first gate question is worded
"is the code dirty?" at three sites, which F-39 shows contradicts § 4 — the
boundary is the *claim*, not the code. D11's integration claim was already
withdrawn once (F-28) and is now short a mechanism (F-36).

## 8. Risks & Mitigations

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
- **R8 — an attestation does not record what it covered** (RV-307 F-25;
  **widened by DEC-020**). Under D10's weak reading a `verified_sha` may attest
  over a proper subset of the declared scope, and the shortfall lives on stderr
  and in `validate` rather than on the record — so a downstream consumer cannot
  tell a full attestation from a partial one. **DEC-020 widens the exposure
  deliberately**: with the stale class no longer refusing, a moved source path is
  now also attested over, so the risk covers every non-contributing entry rather
  than only the never-tracked ones. That is the accepted price of not deriving a
  boundary from local state, and it is why the residual is routed rather than
  tolerated. Closing it needs a persisted coverage field (OQ-3/OQ-6's shape);
  routed as IMP-318. Mitigated meanwhile by E7's report and the `validate`
  finding, which make the shortfall visible to an operator even though it is
  invisible to a consumer.
- **R-A — the claim surface may not be totalisable from git alone.** F-37 is the
  second failed totality claim over path resolution (F-26 was the first), and the
  failures were not adjacent: one was about which shapes *can* be resolved, the
  other about whether resolution failure implies non-contribution. Before
  asserting a replacement, enumerate the reachable shapes — `..` aliases, sparse
  checkout, `skip-worktree`, literal filenames containing glob metacharacters,
  case-insensitive collisions, `core.quotePath` — and probe each. Mitigation is
  method, not text.
- **R-B — `validate`'s contribution probe is undesigned** (F-36). It is the
  mandatory sink for DEC-020's reporting, so the ruling is currently applied
  normatively and not mechanistically.
- **R-C — R4 runs unmitigated meanwhile.** SL-230 ships invalidation without this
  relaxation, so every claim-field edit costs a re-verify against today's stricter
  gate. DEC-027's accepted tradeoff, and the reason to sequence this next rather
  than later.
- **R-D — hostile scope input is not totally handled** (F-38). NUL cannot cross
  the argv boundary, so it is outside E11/E13's exit-code taxonomy entirely, and
  newline-bearing entries split E7's report across lines with no framing rule.

## 9. Quality Engineering & Validation

Model test: `memory_verify_allow_dirty_stamps_checkout_state_id` (`:9123`);
fixture: `GitScratch` (`:5617`).

**Inherited matrix — not yet sufficient.** F-37 and F-36 are unpinned by
construction (they falsify what the table asserts), so this matrix is a starting
point that must grow, not a gate to satisfy.

| # | Test | Asserts |
|---|---|---|
| T7 | verify, unrelated `.doctrine/**` dirty, memory committed | succeeds, stamps **HEAD commit** (not `checkout_state_id`) |
| T8 | verify, memory dir untracked (`record` → `verify`) | **refuses** (D9); message names both the cause and `git commit` |
| T9 | verify, source tree dirty | still refuses; message names `--allow-dirty` |
| T10 | `--allow-dirty`, source tree dirty | unchanged, stamps `checkout_state_id` |
| T10b | `--allow-dirty`, **only the claim** dirty (source clean after exclusion) | stamps a real `checkout_state_id` from the unexcluded capture — **not** empty, not a bare commit (I4, F-13) |
| T11 | `capture(root)` == `capture_with(root, &[])` | I1 — identical frames on clean, dirty, unborn, non-repo |
| T14 | `memory/` absent | exclusion root not contributed; no error |
| T17 | verify, **staged-only** corpus change | excluded; succeeds (index probe leg) |
| T18 | verify, **unstaged/binary** corpus change | excluded; succeeds (worktree diff leg) |
| T19 | verify, **untracked** corpus file outside the memory | excluded; succeeds (untracked leg) |
| T23 | verify on the clean-after-exclusion path while `.git/index.lock` is held | completes — I2 canary; fails if `write-tree` creeps back in |
| T24 | after a successful verify | `git show "$verified_sha:<dir>/memory.md"` equals the on-disk body **byte-for-byte** — I6. Existence (`cat-file -e`) would pass against any stale ancestor blob (F-14) |
| T24b | body **tracked but modified** (not untracked), verify | **refuses** — the case where existence and equality disagree, and the untracked leg does not fire |
| T25 | verify, memory scopes `.doctrine/adr/**`, an ADR under it modified | **refuses** — scoped corpus dirt is claim-relevant (F-6) |
| T26 | claim pathspec construction: absolute-inside-repo, absolute-outside-repo, gitignored-but-tracked, resolves-but-unmatched | normalised / **refuses** / **kept** / reported, per the § 5.2 algorithm (F-6, F-16). The absolute-outside case must assert `verify` **does not abort** — an unfiltered entry makes git `fatal` |
| T27 | verify with a **malformed** scope entry (absolute-outside from this checkout; empty string; tracked symlink whose target is outside) | **refuses**, naming the entry and the reason (D10 malformed, E11, E13). The symlink case is the F-26 collision: it must take the *malformed* branch, not the non-contributing one |
| T27b | verify with `scope.commands` and no path scopes | **succeeds** — commands are exempt by kind, not a failed entry (E5) |
| T27c | verify with a once-tracked-but-moved (`src/worktree.rs`) and a never-tracked (`.claude/skills/dispatch-agent/SKILL.md`) scope entry | **both succeed**, each named on stderr and each raised by `validate` (DEC-020, D10, E7). Retargeted: it previously required the first to refuse. Its job now is the inverse — proving the two are treated *alike* |
| T28 | verify **by key** (not uid), tracked memory, `memory.md` modified | **refuses** — I7. Fails if the claim pathspec is built from the key symlink, where all three legs read clean (F-15). Must use the key form; the uid form passes either way |
| T30 | memory whose `scope.paths` carries `:(exclude)<its own uid dir>`, body modified | **refuses** — I8. The hostile entry is inert under `:(literal)` and cannot subtract the uid directory (F-18). Fails loudly if any entry is interpolated raw |
| T31 | `validate` staleness on a memory scoped **only by globs** | **still not flagged** — the raw seam's `!scope.paths.is_empty()` gate is retained deliberately (D11 narrowed). Pins the *known* gap so R7 cannot be silently closed or silently widened; retargeted by F-27 from an assertion that `validate` was repaired |
| T32 | `validate`'s drift count over a memory scoped by a **tracked symlink that was retargeted in a commit** | counts the retarget — asserting `validate` does **not** canonicalise. Fails if `verify`'s surface leaks into the staleness seam, which would return 0 where the raw path returns 1 (F-27). Equality between the two verbs' surfaces is explicitly **not** asserted; round 5's T32 asserted it and was wrong |
| T33 | memory scoping a **tracked symlink** whose target content changed | `verify` **refuses** — I9. Must probe the *claim* leg specifically, not the source leg, or a dirty tree passes it for the wrong reason. The symlink alone reads clean (F-20) |
| T34 | memory with an **empty-string** scope entry | refuses per T27, and — the discriminating half — the constructed surface is asserted **not** to contain a bare `:(literal)`/`:(glob)`, which would match the whole index (F-23, E11) |
| T35 | `validate` over a corpus containing non-contributing scopes | each is raised as a health finding, once per entry, `scope.commands` excluded (E7). Once-tracked entries **do** appear alongside never-tracked ones — under DEC-020 `validate` is the single sink for the whole of non-contribution, not a remainder after `verify` has refused some |
| T36 | memory whose `scope.globs` pattern is rooted at a **slug symlink** (`.doctrine/adr/001-module-layering/**`), target content modified | **refuses** — I9 step 3. Fails if only concrete paths are resolved: `:(glob)<link>/**` matches 0 files and reads clean, `:(glob)<resolved>/**` matches and reads dirty (F-26) |
| T37 | claim-surface construction over one non-resolving entry, under **three ref states**: never tracked; tracked on a live branch; that branch then `git branch -D`'d | **identical outcome all three times** — attests, with the entry reported. The DEC-020 regression test: it fails the moment any history- or ref-derived discriminator is reintroduced. Round 7's T37 asserted the opposite (once-tracked refuses) and encoded the premise F-31 falsified; the ref-deletion arm is included because varying history *content* alone cannot catch ref-set dependence (F-31) |
| T38 | `scope.globs` entry with a wildcard **inside** a path component (`foo*/bar`, tracked `foobar/bar`) | **observable, and clean-or-refuse** — the whole-component prefix is empty, so the entry is emitted unchanged and matches (F-32). Fails if the prefix is split at the wildcard *character*: `foo` does not resolve, and the entry is misrouted to non-contributing. T36 covers a symlink-rooted *directory* prefix and cannot catch this |
| T39 | scope entry that is **outside-shaped and does not resolve** (`../gone`, `/tmp/no-such`, `:(glob)/tmp/no-such-*/**`) | **refuses** as malformed (E13) — and the discriminating half: `verify` must **not abort**. Git exits 128 on these rather than returning a verdict, so an unguarded entry takes the process down (F-32's second limb). T27 covers *resolvable* outside targets only |

Closure: **every test in § 9 green** (stated as a set, not a numeric range, so a
test added by a later review cannot fall outside the gate by omission — RV-307
F-9); `doctrine check gate` clean; **REV-034 applied** so SPEC-007, REQ-147 and
the implementation agree.

## 10. Inherited review record

This slice has **no ledger of its own yet**. Open one when this design is ready
for adversarial review, and seed it from the findings below.

**Source ledger: RV-307** (`.doctrine/review/307/review-307.toml`), attached to
SL-230. Eight rounds, 39 findings, 29 of them on this gate. `review show` prints
the brief only — read the toml for charges and responses.

Inherited **open** (disposed `descoped` against SL-230 by DEC-027 — unanswered
work with a new owner, not resolved work):

| Finding | Severity | What it says |
|---|---|---|
| F-36 | blocker | DEC-020's `validate` sink has no implementation mechanism |
| F-37 | blocker | non-resolution does not imply non-contribution; three reproduced routes bypass canonicalisation |
| F-38 | major | NUL / newline scope entries escape E11/E13's total contract |
| F-39 | major | code-only wording at D9; IMP-317's title names the rejected shared constructor |

Inherited **contested** — the raiser returned these and they concern text that
moved here, so they are this slice's to answer: **F-25** (partial attestation),
**F-26** (I9 totality and the class collision), **F-32** (prefix splitting and
probe abort).

Inherited **verified** — settled, do not re-litigate without new evidence: F-1,
F-2, F-6, F-7, F-11, F-13, F-15, F-16, F-18, F-19, F-20, F-21, F-22, F-23, F-24,
F-27, F-28, F-29, F-30, F-31, and the governance pair F-4 / F-5 (REV-034).

**Terrain that is settled and must not be reforked:**

- **DEC-020** — non-contribution is reported, never classified. Three refuted
  instruments (F-21, F-25, F-31). A fourth *derived* instrument is not a finding;
  a *declared* boundary is the open path (OQ-6).
- **D11 narrowed** — the claim-surface constructor serves `verify` alone;
  `validate` keeps the raw seam because it asks a historical question where
  canonicalisation erases a committed symlink retarget (measured 1 → 0).
- **The weak reading of `verified_sha`** is the only reading (F-33). Do not
  reintroduce the strong one.
- **R7 / IMP-317** — both historical consumers stay raw pending OQ-2. Adoption is
  a dataflow change, not a call-site swap (F-28).
