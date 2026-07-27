# Notes SL-232: Corpus-aware memory verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-27 · design (**RV-314 run; F-1/F-10/F-11 + F-3 amended into
design.md**; F-2 open) · 81e3e732

### Produced — RV-314 round (this sweep)

- **RV-314** — design-facet ledger, raiser `inquisitor`. **14 findings**: 4
  blockers (F-1, F-2, F-3, F-10), 6 major, 4 minor. External codex pass raised
  F-1–F-10; F-11–F-14 local. 10 verified, F-3 closed, **F-1/F-2/F-10 open**.
- **DEC-069** — measurement and reporting are two surfaces. Measurement = uid dir
  ∪ declared selectors (magic-prefixed) ∪ uncovered symlink closure. Contribution
  reporting stays index-first per DEC-053.
- **DEC-070** — evidence domain is tracked-or-non-ignored commit-eligible.
- **DEC-071** — claim measurement inherits `capture()`'s stable-checkout boundary.
- **DEC-076** — REQ-146/REQ-155 land as **REV-034 rows**, not a second revision.
- **REV-034 amended** — 4 `modify` rows (REQ-147 primary, SPEC-007, REQ-146,
  REQ-155); title widened to the turnover; stale `SL-230 needs` prose corrected.
- **`design.md` amended** — § 5.1 `observe_dirt -> Dirt`; § 5.2 NEW (the split);
  § 5.2a the old rule rescoped to reporting; I7/I8/I10 restated, **I9 struck →
  I9′**; D3 revised, D12 narrowed, D18–D20 added; R6 deepened; **T56–T63**;
  § 10 carries the finding table.
- **RFC-011 case notes** — three entries (prime's selector precondition, codex
  `--as inquisitor` rejection, dev-binary restatement cost).

### Produced — earlier rounds

- **`design.md` REWRITTEN** — wholesale, not patched. The STALE banner is gone;
  the document now carries the decisions. Reference legend added (§ 0), REV-034
  inventory re-taken in-document (§ 5.6), test matrix rebuilt (§ 9).
- **DEC-053** — claim surface is built from the index, never the filesystem.
- **DEC-054** — `validate`'s two unknowns unify; **ISS-257 absorbed**; objective 7.
- **DEC-055** — the undeterminable state is **flat**; `verified_sha`'s kind is not
  discriminated here. Carries the sha256 falsifier that kills the width shortcut.
- **QUE-175 answered `yes`**; **IMP-317 split**, limb (a) taken as objective 4.
- **IMP-325** — discriminate `verified_sha`'s kind on the record. Routed with
  IMP-318/QUE-173 under OQ-A's authored-input vs machine-output ruling.
- **ISS-258** — `memory validate` takes **73s** on this corpus; Check 1 rescans
  the entity catalog per relation. Orthogonal to this slice; found while sizing
  objective 7.
- **`slice-232.md` rescoped** — seven objectives; R-E/R-F/R-G added; OQ-A `no`.
- **`probes/`** — nine executable probes plus README, falsifiers in-header.
  Added this round: `populations.py` (decision populations), `control-chars.py`
  (RV-307 F-38's two obligations).
- **Selectors recorded** — `design-target`: `src/git.rs`, `src/memory.rs`,
  `src/retrieve.rs`, `src/mcp_server/tools.rs`.
- **RFC-011 case notes** — six entries.

### Decisions taken

- **Index-first replaces the ordered algorithm** (DEC-053).
- **ISS-257 absorbed and unified with F-36** (DEC-054), anchored on REV-041.
- **Flat undeterminable state** (DEC-055). Width discrimination **falsified**.
- **OQ-A `no`**, **OQ-2 `yes`** (QUE-175).
- **OQ-6 answered** — `scope.unobservable`: one field, `Vec<String>`, **exact
  string equality** against `paths ∪ globs`, bare array (no `reason`), V1–V5.
  Never subtracts (I8); never clears the attestation.
- **OQ-B answered** — the contribution probe is `expand_scope_entry`, **shared**
  between the verbs because contribution is a *now* question; the historical
  drift seam stays unshared. RV-307 F-27's cut is **history-vs-now, not
  verify-vs-validate**. Continuation policy rides B18. E14 excludes ADR-002
  masters.
- **E13 (and E11) re-justified as lexical malformed REPORTS, not refusals** —
  DEC-020 grounded the refusal in mechanical necessity, DEC-053 removed the
  mechanism, so keeping it would be the judgement DEC-020 forbids. Compelled, not
  chosen. I10 minted as the replacement totality claim.
- **F-38 split three ways** — write-verb rejection (MCP route; argv closes the
  CLI route), read-side malformed report (hand-edit route), report framing via
  the **existing** `scrub_line`.
- **OQ-5 left open, with the tension named** — § 4's own principle points toward
  `yes`; taking it deletes I3. Reopen trigger recorded.

### Measured this round (HEAD `377022dfa`)

Three figures **did not reproduce** from `slice-232.md` and are corrected in
`design.md`:

- **ISS-257's population** — 34 of **59 attested**, reach 42.4%. Not "67 of 115
  anchored": Checks 2/4 gate on `verified_sha`, so *attested* is the denominator.
  Ratio survived, absolute was ~2× overstated.
- **OQ-6's declarable split** — **33 ignored-root / 26 not**, not 20/39. The old
  split used a fixed root list omitting `.agents/skills/**`, `.mcp.json`,
  `.worktrees/**`, `docs/claude/workflows.md`, `web/map/dist`.
- **`verified_sha` carries two value kinds** — 24 of 59 attestations are
  `checkout_state_id`, never commit-anchored. New finding → DEC-055, IMP-325.

### Measured — RV-314 round (HEAD `743e7fe61`)

- **"81 items declare `.doctrine/**` scopes" does NOT reproduce — it is 29**
  (35 entries). RV-314 F-14. The same parser reproduces every other figure
  (440 scope entries exact; 389→390 memories), so the instrument agrees
  elsewhere. Unstamped, ~2.8× overstated, and sitting in § 1's Design Problem.
- **The HEAD × index × worktree cube** — 18 states, 16 dirty; all detected by the
  three legs. `H=A, I=B, W=A` is decisive: only the index leg fires.
- **Ignored-untracked, if counted as evidence** — 19 `.doctrine`-scoped memories
  match 2,983 files; **39 memories / 15,319 corpus-wide**. Killed the inclusive
  domain (DEC-070).
- **I2 holds under the split** — all three legs read-only, each completes with
  `.git/index.lock` held.

### Learned — RV-314 round

- **Check an invariant's POLARITY, not just its truth.** I9 was true and useless:
  it asserted *soundness* (nothing false enters) where the hazard was
  *completeness* (real evidence omitted). Two blockers survived eight adversarial
  rounds under it. Ask what a guarantee makes impossible, then ask whether that is
  the failure you actually fear.
- **An instrument built for a REPORTING question will look correct when reused for
  a MEASUREMENT one.** Both questions read as *"which paths?"*, which is why the
  reuse was invisible. This is the fifth instance of § 5.7's convergence and the
  first where the instrument was right and the *question* was wrong.
- **A design that stamps its figures can still carry one that does not.** F-14's
  81 sat in the section a reader trusts most and re-derives least, in the
  flattering direction, in a document whose § 0 exists to prevent exactly that.
- **Mint the id BEFORE citing it.** Wrote `DEC-074` into three documents, the
  allocator returned `DEC-076` — another agent took 074/075 in between. Also hit
  the inverse: a concurrent agent's follow-up wrote into *my* DEC-069 assuming an
  id it had not been given. The slug symlink is what makes ownership recoverable.
- **Dispose ≠ verify, and the difference is the close-gate.** Blockers whose
  remedy is unperformed stay open; verifying them would clear the gate on a
  promise. F-3 earned `fix-now` + verify because the rows were actually authored.

### Learned — earlier rounds

- **When a rule keeps failing over a domain, suspect the *instrument*, not the
  rule.** The generalisable move is the reformulation that makes the failing
  taxonomy **non-load-bearing** — not enumerating harder.
- **The convergence is now a principle** (`design.md` § 5.7): four times this
  slice answered *"which instrument decides X?"* with *"none — record it at the
  source."* Field of origin, declared boundary, recorded stamp kind, lexical
  guard.
- **More information can be less correct.** The width discriminator would have
  been *more actionable* and *less true* — it emits a false claim on sha256
  repos. Ordering was (record it) > (flat) > (width), which inverts the intuition
  that richer output is better output.
- **Register the falsifier before the probe — it earns its keep.** `FAL-P1`
  ("print every entry with its verdict, not just the total") caught a bucketing
  bug on `populations.py`'s first run. A summary-only probe would have reported a
  plausible wrong number.
- **Verify the verification.** `grep` here is a ugrep wrapper with `-I`, which
  **silently skips binary files** — and `design.md` briefly contained a NUL byte,
  so three "clean" sweep results were false negatives. Use `command grep` and
  check for control bytes explicitly before trusting a negative.
- **The correct shape is often already one module away.** `coverage.rs`'s
  `None => Unknown`, `retrieve`'s B18 per-candidate contract, and `scrub_line`
  all predate their need here. Three seams ridden, none re-invented.
- **A handover's line numbers rot within a commit.** All four cited in the last
  packet had moved. Cite `file.rs::fn_name`; my own DEC-055 citation was wrong by
  60 lines and needed correcting.
- **Stamp every corpus figure with its HEAD** (RV-313 F-1).
- **Inherited, all RV-307:** a tool property is a claim needing a falsifier, not
  a premise · totality claims over path resolution have failed three times ·
  the sweep has four tiers (normative prose, routing-record bodies, queried
  metadata, and a body's own title-shaped H1).

### Open

**Next action: answer RV-314 F-2 (`scope.unobservable` has no producer), then run
an adversarial pass on the § 5.2 amendment before verifying F-1/F-10.** Codex
thread `019fa1a1-4834-7a60-981d-f85e9a7f572f` is warm with the full design,
probes and its own F-1/F-10 analysis. Default reviewer **codex mcp** — not
read-only, or it cannot write the ledger. Drive review verbs from the primary tree.

- **RV-314 F-2 (blocker, OPEN)** — `scope.unobservable` is a *declared* boundary
  with no write surface. Read path is safe by construction (`toml_edit` preserves
  unknown keys; `scope_array` returns empty when absent). Open: CLI flag on
  `memory edit`, MCP `EditParams` field, replace-vs-append, absent-field default,
  whether `record` may author it, corpus backfill. `ClaimSnapshot` must **not**
  gain it or T52 inverts.
- **RV-314 F-1 / F-10 (blockers, OPEN)** — prose written (DEC-069/070/071), not
  yet adversarially checked. Do not verify until it is.
- **RV-314 F-7 is a PREREQUISITE to § 5.2's split**, not a parallel task —
  unguarded derived targets return exit 128 on all three legs. Its exhaustion
  classification (non-contributing / malformed / probe error) is still unchosen,
  and it interacts with the D10-vs-§5.4-table contradiction.
- **RV-314 F-8** — settled in shape, open in detail: name the byte domain
  (`OsString` + widened argv) or narrow I9′ to the UTF-8 index, honestly.
- **RV-314 F-4/F-5/F-6/F-9/F-12/F-13/F-14** — verified, amendments recorded in
  each disposition; **not yet written into `design.md`**. F-14 needs the 29 figure
  restated with its HEAD stamp and moved into a probe.

- **RV-307 F-25 — answered in part only.** **R8 survives this slice.** Objective
  3 must not read as closing it.
- **R-H (new)** — index-first does **not** resolve a symlinked directory in an
  entry's ancestry (`candidate.sh` FAL-4, **failed**). A capability the inherited
  design claimed and this one does not. Recovery measured (`residue.sh` (b)),
  deliberately not built. Live population 0.
- **R-E** — index bits (`S`/`h`) suppress the measurement; no pathspec approach
  closes it (`candidate.sh` FAL-5, **failed**). Live population **0**, and it
  affects the anchor leg too, so it is wider than this slice.
- **R-F** — case-insensitive collision unmeasured, not cleared.
- **R-I (new)** — the Rust TOML parser's handling of an escaped NUL is
  unmeasured; `control-chars.py` measured Python's `tomllib`.
- **R-G** — narrowed by DEC-055: the 34 newly-visible rows drain, because
  objective 1 is what makes clean re-verification possible. **Contested by RV-314
  F-5**: the *stock* drains, the *flow* does not — `--allow-dirty` keeps minting
  non-commit `verified_sha`. Restate as stock-and-flow.
- **R6 deepened (DEC-069)** — the slice **tightens** as well as loosens: untracked
  evidence under a declared glob now refuses. Only the loosening is intuitive.
- **OQ-5** — open, with the § 4 tension named and a reopen trigger.
- **OQ-3 / QUE-173, IMP-318, IMP-325, IMP-317 limb (b), ISS-258** — routed, not
  built here.
- ~~**Governance — REQ-146/REQ-155 routing**~~ — **SETTLED** (DEC-076, RV-314
  F-3). Added REV-034 rows, authored; the revision now carries four. Superseded
  the earlier "`/reconcile` call" note, which contradicted the scope.
- ~~**OQ-2**~~ ~~**OQ-A**~~ ~~**OQ-6**~~ ~~**OQ-B**~~ ~~**E13's fate**~~
  ~~**F-38**~~ ~~**F-39 limb 1**~~ — all answered; see `design.md`.
