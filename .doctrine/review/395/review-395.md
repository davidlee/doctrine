# Review RV-395 — reconciliation of SL-267

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Filled by the auditor, 2026-09-26. **Subject:** `SL-267` (lifecycle `started`;
all six phases `completed`). Audited artefact set: `design.md` (locked run
`dr-01a0d8ff-b311`, revision 35), `plan.toml`/`plan.md`, the runtime phase
sheets, `notes.md` (the three per-axis ledgers + the PHASE-06 client-read), and
the delivered corpus itself.

**The invariant this audit is asked to falsify:** *in a client repo, every
citation in `install/**`, `memory/**`, `plugins/**` either resolves to one of
`DEC-311`'s admissible forms or stands alone, and every CLI claim is true of the
binary.* The design names the acceptance test, not the diff (`design.md` sec-8):
a client read in a scratch repo, where a repo-private id does **not** resolve.

Lines of attack (what is probed, and where the bodies would be buried):

1. **The delivered corpus, not the source tree.** `DEC-311` form 3 grounds a
   claim on *a shipped memory key present in the shipped corpus*. Probe: is the
   key universe the auditor uses the **materialised** set the client receives,
   or the source `memory/` set? Three masters are known-unreachable (`ISS-215`,
   a non-goal) — any citation to them is a non-resolving citation to the client.
2. **Did the acceptance test run where resolution fails, for every channel** —
   `RV-391` `F-7`'s question, re-asked of the *memory-key* channel specifically.
3. **Departures vs the frozen artefacts.** `design.md` sec-1/sec-7 forbid any
   `src/**` change; `plan.toml` PHASE-06 is verification-only. Both were departed
   from (user-authorised). Probe: are the departures real, textual, and is the
   design/plan prose that now misstates them identified?
4. **Is the evidence re-derivable?** The design's own memory warns that a `VA`
   over gitignored runtime state leaves nothing an audit can recompute. Probe:
   are the phase boundaries a usable conformance signal, and do the `VT`
   attributions agree with the committed boundary rows?
5. **Gates and goldens** — `check gate`, `doctor`, `publication validate`,
   `e2e_claude_install`, and whether the asserted numbers reproduce.
6. **The ledger's own honesty** — the slice falsified four of its own earlier
   claims (`IMP-484` false gap; two PHASE-02/03 rows; three understated
   denominators). Probe: are the corrections landed, and are the stale rows
   still asserted as true anywhere a reader would trust them?

## Synthesis

**Judgement.** SL-267 does what it claims, and its evidence is unusually
re-derivable: the three per-axis ledgers, the per-channel F-7 control, the
re-reads through the *render* rather than the source, and a self-falsification
habit that caught four of its own claims before the audit did. On the audit's
independent re-derivation: `doctrine check gate` exit 0 (124 suites, zero
failures), `doctor` 51, `publication validate` 98, `e2e_claude_install` 13/13,
zero dangling `[[mem.*]]` keys in the shipped corpus, zero non-illustration
repo-private ids or paths outside the do-not-sweep classes, and both new
published addresses resolving. `slice conformance` reports 0 undelivered and 83
conformant.

The audit nevertheless found the slice's two blind spots, and both were in the
last mile — between "the source tree is right" and "the client receives the
right thing".

**The 13 findings.** Three were repaired in the audit unit of work (`fix-now`):
`F-1` (seven citations in five shipped masters named three memory keys that
never reach a client), `F-6` (the shipped id/kind glossary was incomplete and
stale), and `F-13` — the sharpest — PHASE-06's **final commit edited four
shipped masters without re-materialising**, so the delivered corpus still carried
four repo-private path citations while every gate stayed green. `F-13` is design
R5 written large: a skipped `memory sync` is invisible to `doctor`, `check gate`,
`publication validate` and `e2e_claude_install` alike, and only a re-run of
`cargo build && doctrine memory sync` (`0 new, 9 changed`) exposes it. That the
same class of defect was one commit away from closure is the strongest possible
evidence for the slice's own thesis — a written rule does not hold without a
gate.

Two findings are design-text corrections routed to reconcile (`F-2`, `F-10`),
and one is per-slice ledger hygiene (`F-7`). The remaining seven are `tolerated`
drift with real, named causes: the shared-worktree boundary footgun (`F-3`,
observation `cfb2e4097`), the two user-authorised plan departures (`F-4`, `F-2`
again), the e2e `store_allowlist` tension (`F-9`), the engine-internal/commented
payload tokens (`F-11`), and the tier-5 evidence items that understated their own
bound (F-8, F-12). No finding is a blocker; no finding was downgraded to avoid
the gate.

**Standing risks.** (1) The three unreachable signposts stay with `ISS-215`; the
repair removed the *broken citations*, not the cause, so re-adding the routing
rows is that item's follow-through. (2) The deferred drift gate (`ISS-309` part
2 / `QUE-227`) is now carrying a second, distinct gap: a rebuild-freshness check
that compares the materialised corpus against its sources. (3) The `folder`
column of the glossary kind table is undocumented and only partly consistent; a
faithful rebuild needs the engine's layout authority rather than invention.

**Tradeoffs consciously accepted.** Verifying by reading the render and the
delivered copy costs more than a grep, and the slice paid it — but the `F-13`
discovery shows the delivery step itself was not covered by the acceptance test,
because the read ran from a rebuilt scratch repo rather than from doctrine's own
materialised tree. The slice ships with `ISS-215`'s user-visible symptom reduced
but not removed, and with the drift gate still deferred; both are recorded, not
silently absorbed.

**Verdict.** The design's closure intent is met on the source corpus and — after
the audit repair — on the delivered corpus. Reconciled: the design text needs two
corrections for departures and one named disposition class; everything else is
terminal. Hand off to `/reconcile`.

## Reconciliation Brief

Artefact: `review-395.md` § Synthesis (audit reasoning) + `review-395.toml` (13
findings, all terminal). In-audit repairs already landed: `3dbe856c5` (F-1, F-6,
F-13 materialisation), `861930ab1` (the F-13 friction observation), plus RV-391
`F-7` verified (F-5).

### Per-slice (direct edit)

- **`design.md` sec-1 (“Boundary”) and sec-7 (surface table)** — record the
  user-authorised, **textual** `src/**` departure: PHASE-05 corrected the taught
  `doctrine prompt resolve` command string in `src/mcp_server/tools.rs` and two
  `src/boot.rs` boot-test literals; no behaviour change. The absolute “makes no
  semantic change to `src/**`” / “No `src/**` change” is now inaccurate. The
  load-bearing change is the **prose correction**; registering the two paths via
  `doctrine slice selector add` is optional (the selector set describes the swept
  corpus, so `slice conformance` will keep calling them undeclared by design).
  *(F-2, `tolerated` away by authority for the change itself; the design text is
  what reconcile fixes.)*
- **`design.md` sec-5 (disposition classes)** — add the **scope-field matcher**
  class: a `.toml` `paths`/`globs` entry is retrieval data, not a prose citation;
  a doctrine-private matcher is recorded, never swept. Five shipped `memory.toml`
  entries carry private matchers (`memory/`, `src/`, `doc/*.md`, `install/hymns/`).
  Today the judgement lives only in `notes.md` § PHASE-03 D3. *(F-10.)*
- **`.doctrine/slice/267/notes.md`** — annotate the two falsified ledger rows so
  the earlier sections do not assert a resolution the PHASE-06 read falsified:
  PHASE-02 `doctrine.toml.example:56 | inline | “the y/N prompt”` (the commit
  never touched the line) and PHASE-03 `EX-2`'s `grep 'install/hymns' -> zero`
  (the path survived at `mem_88193c…:39`). Both sites are repaired; only the
  ledger text is stale. Add the audit section (F-1/F-6/F-13) and refresh § Harvest.
  *(F-1, F-6, F-7, F-13.)*
- **`slice-267.md` / `slice-267.toml`** — no change needed. The scope, selectors
  and relations already cover what was delivered; `ISS-215`'s cause was refined
  (`gather_assets` drops key-named dirs) but the item is unchanged and correct.

### Governance/spec (REV)

- **None.** The slice's governance deliverable — the grounding rule and its
  single home — is `ADR-024`, already `accepted`, and it needs no revision: the
  audit found no amendment to the rule, only corpus instances that violated it.
  `POL-002`'s deliberate non-revision (design sec-5) stands.

### Residuals — recorded, not reconciled (owner named)

- **`ISS-215`** — the three unreachable signposts (`mem.signpost.doctrine.
  {concept-map,rec,rfc}`, real dirs skipped by `gather_assets`). F-1 removed the
  broken citations; restoring the memories and re-adding the routing rows is
  this item's follow-through.
- **`ISS-309` part 2 / `QUE-227` (the deferred drift gate)** — now carries two
  distinct gaps: the citation gate, and a **rebuild-freshness** check (source vs
  materialised) that F-13 exposed. Also the corrected denominators for `ISS-309`
  (prefix set, skill-corpus growth, memory count) and `CHR-080` (10 divergent
  claim groups) — F-8/F-12.
- **`install/glossary.md`'s `folder` column** — undocumented convention and a
  `phases` value inconsistent with the rest; a faithful rebuild needs the
  engine's layout authority. (F-6 residual.)


## Reconciliation Outcome

Reconcile pass for `SL-267`, consuming the brief above. Findings confirmed
terminal before writing (13/13). No REV items.

### Direct edits applied

- **`design.md` sec-1 (Boundary)** — records the user-authorised, **textual**
  `src/**` departure: PHASE-05 corrected the taught `doctrine prompt resolve`
  command string in `src/mcp_server/tools.rs` and two `src/boot.rs` boot-test
  literals; no behaviour change. *(RV-395 F-2)*
- **`design.md` sec-3 (constraints)** — "No `src/**` change" → "No semantic
  `src/**` change", with a pointer to the Boundary note. The third copy of the
  same claim, located while editing sec-1; leaving it would have contradicted the
  corrected sec-1. *(RV-395 F-2)*
- **`design.md` sec-7 (surface impact)** — new row for the two `src/` paths, and a
  note that the design-target selector set describes the swept corpus, so
  `slice conformance` still reports them undeclared (no registry change: the
  selectors are not the departure's surface). *(RV-395 F-2)*
- **`design.md` sec-5 (disposition classes)** — new `scope-field matcher` row and
  note, naming the class the design previously left unaddressed (a `.toml`
  `paths`/`globs` entry is retrieval data, not a citation). *(RV-395 F-10)*
- **`.doctrine/slice/267/notes.md`** — the PHASE-02 `doctrine.toml.example:56` row
  and the PHASE-03 `EX-2` evidence bullet are marked **FALSIFIED** with the
  PHASE-06 repair commit, so the earlier sections no longer assert a resolution
  the read falsified. The audit section and Harvest refresh landed during the
  audit itself. *(RV-395 F-1, F-6, F-7, F-13)*

### REVs completed

- **None.** The brief named no governance/spec item. `ADR-024` is the grounding
  rule's single home and needs no amendment: the audit found no defect in the
  rule, only corpus instances that violated it, and those were repaired during
  the audit (`3dbe856c5`). `POL-002`'s deliberate non-revision stands.

### Withdrawn / tolerated

- `F-1`, `F-6`, `F-13` — `fix-now`, repaired during the audit (`3dbe856c5`,
  `861930ab1`).
- `F-5` — `fix-now`: RV-391's control-routed `F-7` verified against PHASE-06
  `VA-1`.
- `F-3`, `F-4`, `F-8`, `F-9`, `F-11`, `F-12` — `tolerated`, rationale in each
  finding's disposition (boundary footgun; the authorised PHASE-06 repair;
  understated plan terrain; the e2e allowlist; commented tokens; tier-5 evidence
  denominators).
- `F-2`, `F-7`, `F-10` — `verified` → the direct edits above.

### Residuals — owner named, not reconciled here

- **`ISS-215`** — the three unreachable signposts
  (`mem.signpost.doctrine.{concept-map,rec,rfc}`). F-1 removed the broken
  citations; restoring the memories and re-adding the routing rows is its
  follow-through.
- **`ISS-309` part 2 / `QUE-227`** — the deferred drift gate, now carrying two
  distinct gaps: the citation gate, and a **rebuild-freshness** check (source vs
  materialised) that F-13 exposed; plus the corrected denominators for `ISS-309`
  and `CHR-080` (F-8, F-12).
- **`install/glossary.md` `folder` column** — undocumented convention, partly
  inconsistent; a faithful rebuild needs the engine's layout authority (F-6
  residual).

### Notes

- The locked design run `dr-01a0d8ff-b311` is **spent**; `design.md` was edited
  out of band per `mem.pattern.reconcile.edit-design-out-of-band`. The section
  fingerprints diverge from the file and the run's `review_pass RV-391` reads
  `STALE` — expected at reconcile, not corruption. No regress/adopt/re-lock loop
  was run.
- No `plan.toml` criterion was touched (immutable-append) and no selector-registry
  change was made.

Reconcile pass complete — handoff to `/close`.
