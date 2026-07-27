# Notes SL-232: Corpus-aware memory verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-27 · design (**design.md written**; ready for adversarial review) · 26df9de1

### Produced

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

### Learned

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

**Next action: open a fresh RV and seed it from `design.md` § 10.** Default
external reviewer **codex mcp** — not read-only, or it cannot write the ledger.
Drive review verbs from the primary tree.

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
  objective 1 is what makes clean re-verification possible.
- **OQ-5** — open, with the § 4 tension named and a reopen trigger.
- **OQ-3 / QUE-173, IMP-318, IMP-325, IMP-317 limb (b), ISS-258** — routed, not
  built here.
- **Governance** — REV-034's inventory is re-taken in `design.md` § 5.6.
  **REQ-146 and REQ-155 are new rows.** Whether they land as added REV-034 rows
  or a second revision is a `/reconcile` call.
- ~~**OQ-2**~~ ~~**OQ-A**~~ ~~**OQ-6**~~ ~~**OQ-B**~~ ~~**E13's fate**~~
  ~~**F-38**~~ ~~**F-39 limb 1**~~ — all answered; see `design.md`.
