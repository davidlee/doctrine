# Design SL-230: Memory body-write seam

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-230, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§8), A1 (§10). -->

## 0. Scope note — narrowed by DEC-027

**The corpus-aware `verify` gate is no longer part of this slice.** At RV-307
round 8 the user split it out into **SL-232**, which inherits the gate design,
its decisions (D3, D9, D10, D11), its invariants (I1–I4, I6–I9, E2, E4–E9,
E11–E13), its test matrix, and the SPEC-007 amendment REV-034. See **DEC-027**
for the boundary and the rationale.

What remains here is the **body-write seam** — the reusable prose-write path, its
CLI and MCP verbs — and the **attestation invalidation** that a writable body
makes necessary. That half carries 7 RV-307 findings, all verified, and has been
quiet since round 4.

**One consequence, stated rather than buried.** D4's original rationale was that
mandatory re-verification is *"affordable only because the gate relaxation makes
verifying cheap — the two halves pay for each other."* The split breaks that
pairing: this slice ships invalidation against **today's** stricter gate, so every
claim-field edit costs a re-verify with no relaxation to offset it. That is R4,
carried **unmitigated** until SL-232 lands, and accepted deliberately — R4 is
friction, not incorrectness, and the state it replaces has the same friction
*plus* a stamp that survives a claim change (§ 2.1). See DEC-027.

## 1. Design Problem

A memory is two tiers — `memory.toml` (structured, edit-preserving) and
`memory.md` (prose body). **Every write verb in the product reaches only the
first tier.** There is no supported path to author or amend memory prose on the
CLI or over MCP; the only options are hand-editing the `.md` (the raw-file write
the guardrails forbid) or the internal `seed_by_key`. A memory is therefore born
as a title and a summary with an empty body, and correcting a stale body means
leaving the tooling entirely.

Supplying the verb exposes a second defect it must also close: **nothing
invalidates an attestation when the claim it attests to changes** (§ 2.1).
Body-write does not create that defect, but it turns a hand-edit-only footgun
into a first-class, one-command operation.

*(The third problem in the original framing — `verify` refusing on a dirty
corpus — is SL-232's.)*

## 2. Current State

| Surface | Behaviour | Site |
|---|---|---|
| `memory record` | scaffolds `memory.md` from a template — title + summary only | `render_memory_md` `src/memory.rs:1577` |
| `memory edit` | parses `memory.toml` into `toml_edit::DocumentMut`, writes via `write_atomic`; **never opens the `.md`** | `run_edit` `src/memory.rs:3991` |
| MCP `memory_record` / `memory_edit` | metadata fields only; no `body` | `src/mcp_server/tools.rs:310`, `:902-1010` |
| Verification axis | `[review].verification_state`, `[review].reviewed`, `[git].verified_sha` — written **only** by `stamp_verification` | `src/memory.rs:3350-3362` |
| `memory validate` | staleness = commits touching **scoped paths** since `verified_sha` | `src/memory.rs:3424` |

Two facts establish that no seam is being duplicated:

- **No verb of any entity kind writes a prose tier from user input.** Twelve
  checked (`spec edit` — descent scalars only, `src/spec.rs:348`; `backlog edit`
  — status/resolution, `src/backlog.rs:1873`; `adr` has no edit verb; the rest
  scaffold from templates). `seed_by_key` (`:1785`) is the sole body-write seam
  and is internal to install seeding.
- **This slice adds no git call.** `capture()`'s three callers —
  `src/retrieve.rs:532` (read path), `src/memory.rs:1708` (`record`),
  `src/memory.rs:3382` (`verify`) — are all left alone. The body affordance
  changes what `memory_scaffold` *writes*, not how `record` *anchors*, so the born
  anchor cannot move. (I1 made that guarantee normative for the gate's
  `capture_with` and is SL-232's; nothing here calls for it.)

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

## 3. Forces & Constraints

| Authority | Constraint |
|---|---|
| **SL-005 § 5.2 (review #7)** ✓ | "v1 scaffolds a template containing title + summary only — no editor, no stdin, no `--body`. **Richer body capture is a later mutation verb.**" A deferral, not a prohibition — this slice is that verb. No governance step needed. |
| **ADR-001** ✓ | `entity` = engine ("the hub — kind-agnostic directory-entity scaffold", imports `fsutil` only), `memory` = command, `fsutil` = leaf. Downward edges only. |
| **SPEC-007** | Owns the memory system, including arbitrary path/glob/command scopes. Its retired clean-tree contract is **SL-232's** to amend (REV-034); nothing in this slice's scope touches it. |
| **SL-008 D6** | `thread_expiry` is reviewed canon — not loosened. This slice only feeds it honest input (E1). |
| **STD-001** | Named constants, not path literals. The only new string this slice introduces is the body filename passed to `write_body`, which follows the established `dir.join("memory.md")` idiom (12+ existing call sites; no constant exists today) rather than minting a single-use one. Stated because the doc's previous STD-001 discharge — reusing `DOCTRINE_PATHSPEC` (§ 10) — left with the gate. |
| **SL-232** | Owns the gate. This slice must not specify exclusion sets, claim surfaces, or pathspec construction. Where invalidation needs a git fact that the gate also needs, it is stated locally (§ 5.4 staleness) rather than cross-referenced. |

## 4. Guiding Principles

- **Attestation is about the claim.** A memory attests that its body is true of
  what it declares. A change to the body — or to what it asserts against — is a
  claim change and must invalidate the stamp.
- **Build the seam once.** This is the product's first prose-write path; its shape
  is inherited by every kind that follows.
- **Validate everything, then write.** A rejected argument must leave both tiers
  untouched (RV-307 F-3).
- **Supplied is not changed.** Re-setting a field to its existing value must not
  clear a valid attestation.

## 5. Proposed Design

### 5.1 System Model

```
command tier   memory.rs ──────────────┬──────────── run_record / run_edit
                                       │             composes the edit, owns policy
engine tier    entity.rs  write_body(dir, file, text, mode) -> Result<bool>
                                       │
leaf tier      fsutil.rs  write_atomic
```

One change at each of two altitudes: a kind-agnostic body writer at engine, and
policy composition at command. The leaf tier is reused unchanged.

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

### 5.3 Data, State & Ownership

No schema change. No new persisted field. The verification axis keeps its three
existing fields; `write_body` owns no state and returns a changed-flag.

### 5.4 Lifecycle, Operations & Dynamics

**Body on `record`** rides the existing scaffold seam, not `write_body`: `Draft`
and `RecordArgs` gain `body: Option<&str>`; `memory_scaffold` substitutes it for
`render_memory_md`'s output — exactly `seed_by_key`'s existing move
(`fileset.get_mut(1)` → `body.clone_into(b)`, `:1827-1828`). The transactional
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
the TOML **computation**. Steps 3–5 apply the body-dependent mutations to the same
in-memory document, after validation has already passed.

*Why the explicit re-stamp (RV-307 F-12).* There is no separate `stamp_updated`
helper to compose — `apply_edit` stamps `updated` itself, in a terminal block
gated on its own internal `changed` flag, which counts metadata fields only and
cannot see the body. Left as drafted, a body-only edit would never stamp
`updated`, falsifying T5. Step 5 re-stamps explicitly; the write is idempotent when
`apply_edit` already stamped (identical date), and `apply_edit`'s signature is
untouched — which matters under R3, since its existing suite must stay green
unchanged.

So `updated` is stamped, and the verification axis cleared, **iff the body
genuinely changed** — a `--body` that replaces content with itself is a full
no-op on both tiers (content and mtime hold), consistent with the existing
`apply_edit` no-op guard. A crash between step 2 (the body write) and step 6 (the
TOML write) leaves a changed body with a stale `updated`, never the reverse (R1) —
steps 3–5 are in-memory, so the whole window is that one gap.

Full two-tier atomicity would mean routing `edit` through the fileset/rollback
machinery — a large refactor of a shared write path, disproportionate to a crash
window this narrow. Stated, not buried.

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
memory's own body file** — `<uid-dir>/memory.md` — since `verified_sha`, alongside
the existing scoped-paths count.

**The pathspec is the body file, not the item directory.** *Narrowed by the
confirming pass; the directory form was unshippable.* The cause is structural
rather than incidental: `verify` stamps `verified_sha = <HEAD at verify time>`, the
stamp must then be committed, and that commit necessarily touches the memory's own
directory. So `rev-list verified_sha..HEAD -- <uid-dir>` is ≥ 1 for every memory
whose stamp has been committed — the check would report the *sanctioned* flow as
drift, which is the opposite of D5's purpose. Narrowing to `memory.md` avoids it,
because the stamp lands in `memory.toml` alone.

*Measured (RV-313 F-1; restated at reconcile).* On the primary tree at HEAD
`46c4eac83`, 2026-07-27, over the **48 reachable-anchored** memories in the local
corpus: the shipped `memory.md` form flags **3**, the rejected item-directory form
flags **25** — an **8.3× discrimination**. Saturation, the falsifier this narrowing
exists to avoid, is decisively absent. **The property D5 relies on is the
discrimination ratio, not the absolute count** — the count is a function of corpus
age and mix, which is why this figure carries its denominator, date and tree.
The design's original figures (**30 of 30** directory / **11 of 30** shipped) were
taken against a 30-memory anchored corpus that has since grown past 115; they no
longer reproduce, in the safe direction. PHASE-06's **VA-1** derives its absolute
expectation (~11 of 30) from those superseded figures — the criterion id is
immutable and is **not** edited; this paragraph is the surface carrying the current
measurement.

Residual, stated rather than left to be found: a hand-edit of a claim field *in the
TOML* (title / summary / scope) escapes D5. D8 covers those on the verb path, and
the complete answer is a body digest — OQ-3, which is SL-232's.

**Stated residual — corpus reach is ~42%, not corpus-wide (RV-313 F-2).** § 1's
closure promise reads corpus-wide; this qualifies it. Both staleness checks — D5's
and the pre-existing scoped-paths one — compute drift only when the anchor is
*reachable*: `commits_touching` returns `None` for a `verified_sha` that is not an
ancestor of HEAD, and both call sites (`let Some(n) = … && n > 0`) let that `None`
fall out and emit nothing, rendering *cannot determine* identically to *no drift*.
Measured at audit: **67 of 115** anchored memories carry a non-ancestor
`verified_sha`, so both checks are silently inert on them and reach is **~42%**.
The ancestry guard inside `commits_touching` is correct and must stay — the defect
is at the two call sites, and the conforming shape already exists one module away
(`src/coverage.rs:150-166`, `IsStale::{Fresh,Stale,Unknown}` with a documented
`None ⇒ Unknown` contract over the same seam). Pre-existing in the scoped-paths
check; SL-230 inherited it into D5 rather than introducing it, which is why it does
not block this slice. Owned by **ISS-257**; whether it is *also* a SPEC-007
conformance gap is the reconcile REV's question (RV-313 F-6).

*How the path is obtained — by construction, not by resolution.*
`memory_health_findings` receives `(root, &[Memory], today)` and no directory
(`src/memory.rs:3400`); `run_validate` discards `_dir` from `resolve_show`
(`:3482`); the corpus-wide branch calls `collect_all` (`:2826`), which unions
`items/` and `shipped/` deduped by uid. So the path is built —
`root / MEMORY_ITEMS_DIR / memory.uid / "memory.md"` — and is therefore the
canonical uid form by construction. That is why the F-15 symlink hazard (**git does
not traverse a symlink in a pathspec**, and every key in `items/` *is* a symlink to
the uid directory) cannot arise here: no key form ever reaches this check. This
slice needs no resolution step at all, and must not grow SL-232's general
canonicalisation rule (I9). `collect_all`'s lost provenance is harmless for a
reason worth recording rather than assuming — the check is already guarded on a
non-empty `verified_sha`, and **no shipped memory carries one** (31 shipped, 0
anchored: they are minted unanchored, R5/F-7). Every row reaching D5 is an item. If
that ever changed, a shipped row would yield an unmatched pathspec, which
`rev-list` counts as 0 — degrading to "no drift found", never to an error.

**D5's count sits outside the existing `scope.paths` gate.** The pre-existing
staleness check is gated on `!memory.scope.paths.is_empty()` (`:3413-3414`); D5's
count must be a separate condition, not an extension inside that one. **26 of 56**
anchored local memories declare no `scope.paths` at all — signposts and patterns
typically do not — and those are precisely the rows a hand-edit reaches, so folding
D5 into the existing guard would omit 46% of the anchored corpus from the check
that exists to catch hand-edits. Stated because the fact that the guard exists left
this slice with R7. Pinned by T41.

### 5.5 Invariants, Assumptions & Edge Cases

- **I10** — a rejected `edit` leaves **both** tiers byte-identical. Every fallible
  step precedes every disk write (§ 5.4, RV-307 F-3). Pinned by **T40** and by the
  ordering in § 5.4. *Not* by T22: T22 pins the `body_mode`-without-`body` totality
  rule (F-10), which rejects before any write is contemplated, so it passes
  whatever the step order is — the invariant F-3 actually raised needs a failure
  *inside* `apply_edit` alongside a valid body, which is T40.
- **I11** — a `--body` that replaces content with itself is a **full no-op**:
  content and mtime hold, `updated` is not stamped, and the verification axis is
  **not** cleared. The one-way door this protects is destructive — a no-op write
  that cleared a valid attestation (§ 10, A2). Pinned by T6 (content + mtime) and
  T12b (`updated`, verification axis).
- **I12** — the verification axis is cleared **iff a claim field genuinely
  changed**, by comparison and never by which flags were supplied (§ 5.4
  `claim_snapshot`, RV-307 F-8/F-17). Pinned by T12 and T12b (body), T20 and T21
  (the field split), T20b and T29 (comparison, not flag-counting).
- **I5** — `thread_expiry` untouched.
- **E1** — thread memories vanish from `find`/`retrieve` after a body edit until
  re-verified (SL-008 D6 feeding on honest input). Correct but surprising —
  the verb says so on stderr.
- **E3** — body content `-` is unreachable inline; use stdin.
- **I13** — the commit that *writes* a verification stamp is never counted as drift
  against that stamp. D5's pathspec is the body file, so the stamp's own
  `memory.toml` commit cannot fire (§ 5.4). Pinned by T42 — the falsifier the
  item-directory form failed 30 times out of 30.
- **E14** — an idempotent `--path-scope` stamps `updated` while the attestation
  survives. `apply_edit` counts the scope arms changed unconditionally
  (`:3944-3978`) where `claim_snapshot` compares, so this is the one field where
  the two diverge (RV-307 F-17). Stated because it looks like a bug and is not.
  Pinned by T29.
- **E15** — a missing or non-table `[review]` / `[git]` table is **silently
  skipped** by `clear_verification`, so a hand-corrupted `memory.toml` keeps a
  **stale attestation** through a genuine claim change. I12's "cleared iff a claim
  field genuinely changed" (§ 5.4) therefore holds for well-formed TOML only. The
  tolerance is **forced, not chosen**: `clear_verification` is step 4, *after* the
  body write, so a fallible clear would reintroduce exactly the mutation-on-failure
  that **I10** forbids and that **T40** catches. Refusing here would trade a narrow
  residual for the one-way door I10 exists to hold shut (RV-313 F-4). Stated rather
  than left latent; the residual is accepted, not mitigated.

*Criteria ids are immutable.* I1–I4 and I6–I9, and E2, E4–E9, E11–E13, moved to
SL-232 with the gate; the ids are **not** reused here, which is why this slice's
new invariants begin at I10 and its new edge cases at E14.

## 6. Open Questions & Unknowns

- ~~**OQ-1**~~ — **closed by RV-307 F-8**; answered as **D8**. Title, summary and
  scopes clear verification alongside body; status / lifespan / review_by / trust /
  severity do not.
- **OQ-4** — when do other kinds adopt `write_body`? This slice wires memory only.
  The seam is built kind-agnostic (D1) so that adoption is a caller change, not a
  redesign — but no second kind is converted here, and none should be smuggled in.
- **OQ-7** — does `write_body` belong on the MCP surface for *other* kinds once
  they adopt it? Out of scope; noted so the question is not rediscovered as a gap.

*OQ-2, OQ-3, OQ-5 and OQ-6 moved to SL-232 — they are all gate questions.*

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
- **D4 — body edit clears the verification axis.** *Its original rationale is
  superseded:* the draft read "affordable only because the gate relaxation makes
  re-verifying cheap; the halves pay for each other", and D3's relaxation left with
  the gate, so the cost now stands unoffset (R4, § 0, DEC-027). The decision
  survives the loss of its affordability argument, because the alternative is a
  stamp that lies — but it is no longer a cheap decision, and § 0 is where that is
  accounted for.
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
- **D5 — own-*body* staleness in `validate` only.** Closes the hand-edit inversion
  without re-ranking the corpus as a side effect of a body-write slice. *Narrowed
  by the confirming pass* from the whole item directory to `memory.md`: the
  directory form counts the verify stamp's own commit and so reports drift on 30 of
  30 anchored memories (§ 5.4, I13). *Alternative:* defer D5 to SL-232 alongside
  OQ-3's body digest — rejected, it costs objective 5 half its content to close a
  residual D8 already covers on the verb path.
- **D6 — items-only; masters out of scope.** The motivating memory
  (`mem.signpost.project.orientation`) is an item ✓, so the case is covered.
  Extending `resolve_memory_toml_path` would change *every* memory write verb.
  Mitigated by `mem.system.memory.global-master-authoring` — now glob-scoped
  `memory/**` at severity high, so it fires on any masters edit.
- **D7 — `--edit-body` / `$EDITOR` dropped, not deferred.** Unusable from a
  jailed or MCP agent, which is the audience.

*D3, D9, D10 and D11 moved to SL-232.* D4's affordability argument depended on
D3's gate relaxation; with the split that support is gone and R4 stands
unmitigated — see § 0.

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
  **Carried unmitigated** until SL-232 lands — the gate relaxation that made it
  affordable left with D3 (§ 0, DEC-027). D8 widens the trigger set, so this risk
  grows — accepted, because the alternative is a stamp that lies. Friction, not
  incorrectness: the state it replaces has the same friction *plus* a stamp that
  survives a claim change. SL-232 is the mitigation, and the reason to sequence it
  next.
- **R5 — masters remain uncovered by every invalidation path** (RV-307 F-7).
  They are unanchored (no `verified_sha`) and `collect_all` does not scan them, so
  neither D5's own-directory drift nor D8's field-clearing reaches a master edit.
  D6 puts them out of scope; the only standing mitigation is the
  `mem.system.memory.global-master-authoring` guard memory (glob `memory/**`,
  severity high — both confirmed live). Stated as a known gap, not a solved
  problem. Closing it needs an anchor for masters, which D6 puts out of scope; no
  open question in *this* slice carries it, and the body-digest route that would
  have (OQ-3) is SL-232's.

*R6, R7 and R8 moved to SL-232.*

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
| T12 | body edit via the verb | clears `verification_state`/`reviewed`/`verified_sha` |
| T12b | `--body` replacing content with itself | full no-op: `updated` **not** stamped, verification **not** cleared |
| T13 | **hand-edit** `memory.md` directly, commit, `validate` | flags stale via own-**body** drift — must exercise the *bypass* path, since the verb path clears the stamp and would never reach the staleness check |
| T15 | existing memory + entity suites | green unchanged (R3) |
| T16 | hostile body written via `--body`, then `show` | read-time nonce + `data, never instruction` framing intact (R2) |
| T20 | edit `title` / `summary` / `scope.*`, each alone | clears the verification axis (D8) |
| T20b | set `--path-scope` to its **existing** value | does **not** clear — `claim_snapshot` compares, it does not count flags (F-8). Scope, not title: `apply_edit` already guards title, so a title-based test passes with `claim_snapshot` absent and proves nothing (F-17) |
| T21 | edit `status` / `lifespan` / `review_by` / `trust` / `severity`, each alone | does **not** clear (D8's other half) |
| T22 | `body_mode` without `body`, CLI **and** MCP | rejected on both, same message (F-10) |
| T29 | idempotent `--path-scope` | `updated` **is** stamped (`apply_edit` counts it changed) while the verification axis is **not** cleared — the one place the two diverge (F-17) |
| T40 | `edit --body - --trust bogus` — a valid body alongside metadata that fails *inside* `apply_edit` | rejected, and **both** tiers byte-identical: `memory.md` and `memory.toml` unchanged (I10, F-3). The step order is the only thing that makes this pass |
| T41 | hand-edit + commit a memory declaring **no `scope.paths`** | still flagged — D5's count sits outside the pre-existing `!scope.paths.is_empty()` guard (§ 5.4; 26 of 56 anchored memories are this shape) |
| T42 | `verify`, **commit the stamp**, then `validate` | **not** flagged — the stamp's own commit touches `memory.toml`, not `memory.md` (I13). The registered falsifier for D5's pathspec: the item-directory form fails this 30/30 |

*The gate's tests — T7–T11, T14, T17–T19, T23–T28, T30–T39 — moved to SL-232.
Ids are not reused; a test added here starts at T43.*

**T13 note.** It exercises `validate`'s own-body staleness (D5) via the
**hand-edit bypass**, which is the only path where it has meaning: under D4 a verb
edit clears the stamp, so there would be no `verified_sha` left to compare and the
staleness check would never run (§ 10, A4).

Closure: **every test in § 9 green** (stated as a set, not a numeric range, so a
test added by a later review cannot fall outside the gate by omission — RV-307
F-9); `doctrine check gate` clean. **REV-034 is no longer this slice's gate** — it
moved to SL-232 with the contract it amends (DEC-027).

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

Eight rounds. Full charges, evidence and responses are on the ledger
(`review-307.toml` — `review show` prints the brief only). **Counts in this
section are a standing staleness hazard** — they were wrong at round 7 (RV-307
F-35) because a round updates the ledger and not the prose. Prefer
`doctrine review status RV-307` over any number written here.

**Read the table below with DEC-027 in hand.** The ledger stays attached to this
slice because it is append-only and it reviewed *this* document across eight
rounds. But at round 8 the gate was split out into **SL-232**, so roughly
two-thirds of the rows below land in sections that are no longer here: any row
whose home is **D3, D9, D10, D11, I1–I4, I6–I9, E2, E4–E9, E11–E13, R6–R8, the
claim-surface algorithm (SL-232's § 5.2), or T7–T39** now resolves against
`.doctrine/slice/232/design.md`, not this file. **Read that "§ 5.2" as SL-232's
section, not as a bare number** — *this* file's § 5.2 is Interfaces & Contracts,
and F-10's rule lives there, locally. The rows are kept, unedited, because
deleting them would falsify the record of what this document was reviewed for.
SL-232 § 10 carries the inherited-findings view organised by *state* rather than
by round.

Rows that remain local to this slice: F-3, **F-7**, F-8, F-9, F-10, F-12, F-17,
and the internal pass A1–A4. Two corrections to the membership DEC-027 recorded,
which had the count right and the names wrong: **F-14 is not local** — it lands on
I6 and T24, both SL-232's ("*T24 proves only that some blob exists at the path*"),
so SL-232's inherited set must pick it up. **F-7 is local** and was missing — it
lands on R5, which is retained here. Seven either way.

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
| F-25 | D10 attested surfaces missing declared evidence | D10, § 5.2 weak reading, R8, T27c |
| F-26 | I9 unapplicable to patterns / absent paths; class collision | § 5.2 algorithm, I9, E13, T27, T36 |
| F-27 | canonicalising a historical query erases retarget drift | D11 narrowed, I9 scope, T32 |
| F-28 | `validate`/`retrieve` lack the constructor's `dir` | D11, R7 cost correction |
| F-29 | no `validate` policy for the malformed class | dissolved by D11 narrowing |
| F-30 | R7's insertion overwrote R6's heading | § 8 R6 restored |
| F-31 | the history discriminator is ref-set-dependent, not checkout-stable | **DEC-020** → D10 shrunk, E12 withdrawn, T37 inverted |
| F-32 | textual wildcard-free prefix ≠ path prefix; outside-shaped non-resolving entries abort the probe | § 5.2 steps 3–4, I9, E13, T38, T39 |
| F-33 | strong and weak attestation contracts both asserted | § 4 principle, § 5.2 weak reading, § 5.4, `slice-230.md` |
| F-34 | routing records left outside the integration sweep | QUE-175 body |
| F-35 | pointer-only history still carried counts | this section's preamble |

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
sharpest, and its lesson is below: the responder had probed the prefix rule with a
check that could not distinguish the two outcomes — **F-17's defect, committed by
the party that raised F-17.** **F-20** is F-15 one axis over. **F-24** found the
third scope consumer that D11 had claimed did not exist. Rules at E11, I9 and R7
respectively.

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
holds in**. A rule that was right for `verify` was promoted to a universal and
handed to a verb asking a different question, producing a false negative in the
one place meant to catch what `verify` misses (F-27). The remedy for
over-specificity was over-generality. Rules at I9 and D11.

**F-25 changed a decision for the second time.** The round-5 cut split
non-contribution by whether the path exists on disk; the round-6 cut replaced that
with git history. What survives is the *practice*, not either rule: a class
boundary of this shape is taken on a census of the whole corpus, and re-measured
when the discriminator changes. The rule itself was superseded again at round 7 —
see D10 and DEC-020, which are authoritative.

F-26 and F-29 were dissolved structurally rather than patched: the scope rule is
now an ordered algorithm whose classes are decided by probe outcome, and
`validate` no longer builds a surface at all, so it needs no policy for one.

**Where round 6 left the design.** The rate had not decayed, and its findings were
not polish — two of them refuted a decision (D11) and a class boundary (D10)
rather than correcting an edge. What changed was the *kind* of defect: no longer
"which adjacent path inherits a dead assumption" but "does each rule hold across
the domain it is stated over".

#### Round 7 — external pass on the round-6 remediation

Verified F-19, F-20, F-27, F-29, F-30. Contested F-6, F-22, F-24, F-25, F-26,
F-28; raised **F-31** and **F-33** (blockers), F-32, F-34, F-35. Every new charge
was reproduced by the responder before disposition.

**The round ended the classification attempt rather than re-cutting it.** F-31
refuted the third instrument for one class boundary, and the user ruled the
question out of this slice — **DEC-020**. See D10 and § 5.2; both are
authoritative, and this paragraph deliberately says no more.

**The dominant cost driver, named.** Two of seven rounds went on successive
instruments — filesystem existence, then `rev-list --all` — whose *stability* was
asserted and never probed. The lesson generalises past this slice and past
memory: **a property of a tool (stable, total, deterministic) is a claim needing
a falsifier, not a premise.** Measuring that a discriminator *works* is not
evidence that it is *stable*; the probe must vary the local state the instrument
reads. Recorded in `.doctrine/rfc/011/case-notes.md`.

F-33 is the fifth instance of the § 10 pattern at the level of the *core
contract*: the weak and strong readings of `verified_sha` coexisted as separate
normative claims, so a planner could have implemented either and stayed textually
compliant. Fixed by deleting the strong reading, not by softening the weak one.

The gate for `/plan` is the ledger's, not the author's confidence.
