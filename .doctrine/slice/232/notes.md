# Notes SL-232: Corpus-aware memory verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-27 · design (**RV-314 rounds 2–3; 20 findings, all
disposed; six blockers answered-but-UNVERIFIED**) · eb1dc203

### Produced — RV-314 rounds 2 & 3 (this sweep)

Two external adversarial rounds on the warm codex thread. **Each refuted the
previous round's amendment.** Six findings raised, all reproduced by the
responder before raising, all disposed `fix-now` with prose written.

- **RV-314 → 20 findings.** Round 2 raised F-15/F-16 (blockers) + F-17 (major);
  round 3 raised F-18/F-19 (blockers) + F-20 (major). F-2 closed.
- **DEC-080** — the symlink closure splits **emission** from **discovery**.
  Emit every lexically-eligible joined target as `:(literal)<target>`
  unconditionally; index re-expansion only walks to deeper `120000` entries.
  I8 restated to bind **derived** pathspecs, not only declared ones.
- **DEC-081** — `scope.unobservable`'s producer: `memory edit` alone, replace
  semantics, `num_args = 0..=1` so a bare flag clears, `record` and the embedded
  template untouched, `ClaimSnapshot` unchanged by construction. No backfill.
- **DEC-082** — the 18-state cube is a **content/existence projection**, showing
  the legs jointly *necessary* not sufficient. R-E promoted to I9′'s **third
  bound**.
- **DEC-087** — `--attr-source=<empty tree>` joins `NORMATIVE_FLAGS`. Closes
  attribute conversion as a *class*. I6 **kept, not narrowed**.
- **CON-002** — doctrine-wide git **2.40** floor (new; none previously declared).
  Unmet ⇒ **legible refusal via capability probe**, never silent degradation.
- **IMP-326** — the ~33-entry `unobservable` corpus backfill, HEAD-stamped.
- **IMP-327** — scope arrays are clearable via MCP but not the CLI (pre-existing).
- **`design.md` amended** — § 5.1 the raw-byte legs subsection; § 5.2 the
  emit/discover split, the constructed uid base + its **three** safety
  conditions, the index-flag exclusion; § 5.2a step 4 and the derived-string
  prefix rule; I6/I7/I8/I9′ restated; R-E promoted in § 8; **T64–T77**; § 10
  carries 20 findings and the two round lessons.
- **RFC-011 case notes** — two entries (the acquittal-asymmetry finding; the
  self-probing-narrows-but-does-not-close note).

### Produced — RV-314 round 1

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

### Measured — RV-314 rounds 2 & 3 (git 2.54.0, HEAD `eb1dc203`)

Every figure below was reproduced by the responder, not accepted from the
reviewer.

- **Detached symlink target** — surface `[:(literal)link]` reads
  `tracked=0 / untracked=0 / index_rc=0`; the control emitting the target
  literally reads `tracked=145 / index_rc=1`. F-15.
- **Derived pathspec injection** — a tracked symlink whose blob is
  `:(exclude)uid/**` gives `tracked=0 / index_rc=0` raw versus
  `tracked=152 / index_rc=1` literal-prefixed. F-16.
- **Unmatched literal pathspecs are inert** — `rc=0` on all three legs, and a
  real signal beside them still reports (`tracked=143 / rc=1`). This is what
  makes unconditional emission affordable and what killed F-20's optimisation.
- **Index-flag blindness** — `assume-unchanged` and `skip-worktree` both read
  `0/0/0` while tracked and modified on a stable checkout. F-17 → DEC-082.
- **`.gitattributes` defeats all three legs** — `eol=crlf`: HEAD `…0a` vs
  worktree `…0d0a`, `cmp` NOT identical, `0/0/0`, `git status` empty. **Clean
  filter: `CANONICAL` vs arbitrary attacker content, `0/0/0` against the
  `--cached` leg § 5.1 specifies.** F-19.
- **The repair measured** — `--attr-source=<empty tree>` raises the tracked leg
  `0 → 172` (filter) and `0 → 156` (eol).
- **Empty-tree oid is hash-algorithm dependent** — `4b825dc6…` sha1,
  `6ef19b41…` sha256. Must be derived (`git hash-object -t tree /dev/null`).
- **Capability probe** — `git --attr-source=<oid> rev-parse --git-dir` exits 0
  supported, 129 `unknown option` otherwise. Non-zero is the sufficient test.
- **clap 4 `num_args = 0..=1`** — absent `None`, bare `Some([])`, valued
  `Some([v])`, repeated appends; value count caps at 1 so the positional falls
  through. Only misfire: bare flag immediately before the positional, which
  always errors loudly (`<REFERENCE>` is the sole positional).
- **This repo has no `.gitattributes`** — DEC-087's usability cost has live
  population 0 here.

### Measured — RV-314 round 1 (HEAD `743e7fe61`)

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

### Learned — RV-314 rounds 2 & 3

- **Check the AXIS, not just the property — the round-1 lesson generalised.**
  Three findings across three rounds are one mistake: verifying something *true
  that is not load-bearing*. I9 asserted soundness where the hazard was
  completeness (F-1/F-10). The prefix rule was proved for *declared* strings
  where the hazard was *derived* ones (F-16). The uid was validated for its
  *alphabet* where the hazard was its *identity* (F-18) — **committed in the same
  round that recorded the previous two**, in prose claiming to close that class.
  A true property stated confidently is the most reliable way to stop looking.
- **An external reviewer's ACQUITTALS need re-derivation more than its findings.**
  Codex cleared the clean-filter limb after probing `diff-index --quiet HEAD`;
  § 5.1 specifies `--cached`, and against that leg the miss is total and hides
  *arbitrary* content. The finding it did raise was real but materially
  understated. Convictions arrive with a probe attached and get re-run;
  acquittals arrive as prose and get believed. Cost of catching it: one 12-line
  probe. Cost of missing it: a blocker ships as a wording fix.
- **A repair to a REUSE defect must be checked against every consumer of the
  reused instrument.** DEC-069 moved declared entries off the index and left
  derived ones behind, because the fix was written where the finding pointed
  rather than where the *class* lived. That gap became two more blockers.
- **Pre-empt the reviewer on your own new prose — it is much cheaper than a
  round-trip, and it still will not save you.** Self-probing found the uid-base
  gap before codex reported; it did not stop me getting the axis wrong.
- **Neutralise at the instrument, not per artefact.** F-19's per-file raw-byte
  check would have needed the concrete-path enumeration D18 already rejected;
  one flag on `NORMATIVE_FLAGS` closed the whole class instead. When a hazard is
  the *tool's* view, fix the tool invocation.
- **Relations move via `doctrine link`, not hand-edited `[[relation]]` blocks.**
  Hand-wrote an illegal `constrains` label into CON-002; `doctor` caught it.

### Learned — RV-314 round 1

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

**Next action: round 4 on the warm codex thread
`019fa1a1-4834-7a60-981d-f85e9a7f572f`, then verify the six blockers if it
holds.** Rounds 2 and 3 each refuted the previous round's amendment, so a fourth
pass is the expected cost, not pessimism. **Instruct it to re-derive its own
acquittals** — round 3's most serious finding was hiding inside one. Default
reviewer **codex mcp** — not read-only, or it cannot write the ledger. Drive
review verbs from the primary tree.

- **Six blockers ANSWERED, none VERIFIED** — F-1, F-10, F-15, F-16, F-18, F-19.
  Every remedy has prose in `design.md`; none has survived a pass. `dispose ≠
  verify`, and the close-gate is the only thing tracking the difference. **The
  design must not go to `/plan` on this state.**
- ~~**RV-314 F-2**~~ — **CLOSED** (DEC-081, § 5.3). Producer is `memory edit`
  alone. Backfill declined → IMP-326; CLI/MCP clear asymmetry → IMP-327.
- **DEC-087's two unpaid obligations** — (a) `capture()` byte-identity (I1/T59)
  must be **re-run, not reasoned about**, in a no-`.gitattributes` fixture (T74);
  (b) `checkout_state_id` inputs change in attribute-using repos, so persisted
  values are not reproducible there. Population 0 here; both stated, neither
  discharged.
- **CON-002 is a new doctrine-wide constraint** — the git 2.40 floor did not
  previously exist. It binds every invocation (`NORMATIVE_FLAGS`), not just
  `verify`, because scoping the flag to verify's probes would split-brain
  `capture()` against `verify`.
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
- ~~**R-E**~~ — **PROMOTED, no longer merely a risk** (DEC-082, RV-314 F-17).
  Index bits (`S`/`h`) suppress the measurement on a stable checkout while
  tracked — a counterexample *inside* I9′'s other two bounds, so I9′ was false
  until R-E became its **third named bound**. Pinned by expected-blind T64, which
  also discharges F-13's symmetry complaint. Live population **0**; still affects
  the anchor leg, so still wider than this slice.
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
