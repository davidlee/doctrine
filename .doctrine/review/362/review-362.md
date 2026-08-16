# Review RV-362 — reconciliation of SL-251

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

> **Provenance.** Authored 2026-08-16 04:31 as `RV-358` on the capsule ref
> `refs/capsule/d/heads/work`, which was never merged. Recovered and re-minted
> here after `SL-251` had already closed under `RV-361` — id 358 belongs to
> `SL-238`'s design review on `edge`. Findings, dispositions and responses are
> verbatim; only ids were repointed. See `## Reconciliation Outcome`.

## Brief

**Surface reviewed.** The primary worktree at `eca2c9a11`, branch `work` — a
**solo** slice, not dispatched. There is no candidate interaction branch and no
`review/*` / `phase/*` evidence refs; the code under review is the working tree
itself, spanning `de32c856b..d7001375c` (PHASE-01..07) plus the two knowledge
commits after it.

**What SL-251 claimed.** Three scope objectives: (1) the `design apply` payload
contract becomes *fetchable* rather than exemplified — `DEC-224`'s three
renderings from one generator (a `doctrine design contract` verb, a one-line
pointer in `design apply --help`, and a generated reference document sealed
`customization = "fixed"` per `DEC-226`); (2) the contract is *pinned* against
the types so drift is a test failure; (3) the surface is *total* over the
payload — `DEC-227`'s full recursive wire closure rooted at `ApplyRequest`, not
the nine-row `WRITER_ACTS` table the slice's own scope names as the wrong axis.

**Lines of attack.**

1. **Does the pull surface actually answer where it claims to?** `DEC-224` makes
   the JSON the *primary* rendering and `PHASE-06/EX-3` claims the verb needs no
   root, no snapshot and no I/O. Probed live from outside a doctrine project and
   from a freshly installed client project — not by reading the test.
2. **Is the totality claim real, or is it `WRITER_ACTS` again?** `delegation` is
   the tell: absent from `WRITER_ACTS`, and scope §3 commits to covering it. Type
   set and top-level key set compared against `render-sample.md`, the
   hand-derived evidence that predates the generator.
3. **Do the pins pin, or do they mirror?** The slice's own record says its
   longest review thread (`RV-357` `F-6`, five rounds) turned on an oracle that
   read its answer off the thing it was checking. The coverage equality's
   independence, the removal probe's read-path direction, and the exhaustiveness
   barriers are the load-bearing claims.
4. **Does the evidence trail survive the phase sheets?** Runtime sheets are
   disposable; every `VA-` criterion is discharged by an agent's reading and is
   worthless at audit if unrecorded. Each of the seven phases' `VA` results
   checked for a written result, and the unrecorded ones re-discharged here.
5. **Does the mechanical conformance delta name real drift?** `slice
   conformance`'s undeclared / undelivered cells read as leads, and the
   `attestation.rs` selector is the known suspect — `PHASE-02/EX-10` found the
   design wrong about which file holds the closure structs.
6. **Is what the slice deliberately did NOT close still honestly stated?**
   `ISS-333`'s silent top-level discard is an explicit Non-Goal. Probed live,
   because a slice that cures discoverability is exactly the one whose prose is
   tempted to overclaim.

**Invariants held to.** `STD-001` (the address spelled once; the contract
rendered from one source, not restated); `POL-002` (no repo-private id, no
`sec-N`, no host-project convention on a shipped surface); `ADR-001`
(`design_run` is leaf, out-degree 0 — the extern region must be *injected* from
the command tier, never imported into the leaf); `ADR-019` (an `install/` asset
and its `publication/manifest.toml` row land together); the
behaviour-preservation gate (every wire-type addition `#[cfg(test)]`, existing
suites green unchanged); and the boot rule that `PHASE-NN` / `EN-`/`EX-`/`VT-`
ids are immutable-append.

## Synthesis

### The closure story

SL-251 set out to make `doctrine design apply`'s payload contract **fetchable
rather than exemplified**, and it did. The measurement that justified the slice —
RFC-026 `E8.7`, fifteen of thirty-three source reads on one design run spent
looking up payload shape — has a direct answer now: `doctrine design contract`,
which this audit ran from a directory with no `.doctrine` ancestor and got 195
lines of prompt rendering and 23 KB of JSON, both exit 0, empty stderr, no root,
no snapshot, no I/O. The same invocation is the one a refusal now prints, which
this audit also reproduced end to end on a freshly installed client project:

    Error: parse the apply payload as JSON: unknown field `cursour`, expected one
    of `subject`, `question`, … at line 1 column 152
      doctrine design contract --format prompt

Serde's own message survives verbatim, the address rides an indented
continuation, and the pasted invocation answers. `cursor` — the one key a
resuming agent must set, and the omission that provoked the slice — now appears
three times over: on `TraversalDeclaration` in the contract with its sparse
semantics stated, in the corrected worked example in the turn envelope
(`DEC-228`), and in the published reference document.

**Totality was the claim most at risk of quiet failure, and it holds.** Scope §3
named the trap precisely: `WRITER_ACTS` has nine rows against `ApplyRequest`'s
ten act fields, and a surface keyed naively off it would ship a payload field no
caller could discover. `delegation` is the tell, and it is there — thirteen
top-level keys render, three flattened from `SubmissionEnvelope` and ten act
fields. Beyond the root, `DEC-227`'s full recursive closure: this audit compared
the generator's output against `render-sample.md`, the hand-derived evidence that
predates the generator, and the type-name sets are **identical at 25** — ten
structs, fourteen enums, the root — zero on either side only.

**The pins pin.** The slice's own record says its longest review thread
(`RV-357` `F-6`, five rounds and four repairs) turned on an oracle that read its
answer off the thing it was meant to check, so independence was the thing to
probe rather than accept. It survives inspection: `reached()` records sites from
arrival and its doc comment states the constraint explicitly ("nothing in this
function or below it reads the table for the left side"); `declared_sites(PAYLOAD)`
derives the other side; and the positive control empties one `Seq`, asserts the
equality **breaks**, and asserts it loses *exactly that site and nothing else*
before the real union is asserted clean. That is a control that can fail, not a
gesture at one.

`doctrine check gate` is green. `doctrine slice verify-vt 251` is clean across
all seven phases after `F-1`'s repair.

### What the audit changed, and why it is small

Twelve findings, **no blocker**. Nothing in the shipped behaviour was wrong. The
findings cluster into three kinds, and the split is the interesting part:

- **Two spent criteria** (`F-1`, `F-2`) — both *void on contact*, both fixed or
  tolerated at audit on the append-never-renumber rule.
- **Four records that outran their artefacts** (`F-3`, `F-4`, `F-9`, `F-10`) —
  evidence in commit messages rather than sheets, a harvest stamped four phases
  back, a duplicate issue, a memory carrying a falsehood. All repaired here.
- **Four obligations for `/reconcile`** (`F-5`..`F-8`) — two selector-registry
  rows, one REV on `SPEC-029`, one direct-edit pass over five design.md claims.
- **Two regressions that rode the slice's range without being its work**
  (`F-11`, `F-12`).

### Standing risks

**The design predicted a tool's behaviour and was wrong three times.** The slice
named this class itself, in notes.md, after `A3` and the clap cost-estimate cost
it two rounds: *an unverified claim about what a tool cannot do, load-bearing on
a design decision*. Having named it, it produced a third instance —
`sec-6`/`sec-8` pin 7 reasoning from clap's help renderer, which this project
replaces. The recorded remedy ("cite the source whenever asserting a limit of
clap, serde or cargo") is a discipline, not a mechanism, and discipline did not
catch the third. `F-8` carries the corrections; the class stays open.

**A VT's `test_file` was mis-sited four times, and the fourth would have gone
green.** Three would merely have redded the build — a leaf-sited test naming a
command-tier symbol is a layering violation `just check` catches. `PHASE-06/VT-1`
was different: a leaf-sited golden can only pass `extern_fixture()`, PHASE-05's
two-row stub, so it would have pinned the *shipped, published* document to a test
fixture and passed, leaving `install/design-payload-contract.md` describing a
knowledge region that does not exist with a green test underneath it. `DEC-140`
decided this in the general case on 2026-08-04 and `/plan` does not enforce it.
Carried out as `IMP-438`. **A defect class that usually fails loudly and
occasionally fails silently is worth mechanising precisely because vigilance
calibrates to the usual case.**

**Grep-keyword VT gates are proxies, and the better implementation can lose.**
`F-1` is the live instance: `CARGO_MANIFEST_DIR` names an implementation token
that a correct DRY refactor routes through a shared helper, and satisfying the
grep would have meant spelling the env var twice in violation of `STD-001`. The
criterion was retired rather than the code bent, but the general hazard —
verification keyed on tokens rather than seams — is unaddressed.

**`slice conformance` is structurally blind between phase boundaries.** Two
unrelated regressions (`F-11`, `F-12`) landed in the two-commit gap between
PHASE-03's recorded end and PHASE-04's recorded start, and the mechanical delta
could not see either, because a boundary is the half-open range `(start, end]`.
Both were found by reading the range by hand. Conformance says *where to look*,
never *whether it passes* — and here it could not even say where.

**`RecordKind::ALL` is hand-maintained** (`ISS-364`), so the extern region's kind
list rests on a barrier the slice deliberately did not build. `sec-3` states this
at its real strength and `sec-8` pin 5 takes its oracle from an exhaustive match
rather than from `ALL`, which is the strongest available answer short of deriving
`ALL` from the enum. Out of scope here, and correctly so.

### Tradeoffs consciously accepted

**`ISS-333`'s mechanism survives, and the slice must not be recorded as having
closed it.** Verified live at this audit: a misspelt *top-level* key parses,
bumps the revision, writes a receipt, prints no change row and exits 0. That was
an explicit Non-Goal — the alternative was a hand-written `Deserialize` or a
deserialize-to-`Value`-and-subtract, both rejected. What SL-251 closes is the
*probing loop*: nobody now needs to submit-and-read-the-refusal to learn the
shape. `F-9` carries that distinction into the discharge statement.

**The JSON grew a fifth top-level key that `EX-3` did not enumerate.** `extern`
was appended at execution (`PHASE-05/EX-10`) on a scope argument, not a compiler
one: `DEC-227` commits the surface to totality and `DEC-224` makes the JSON the
*primary* rendering, so a JSON omitting the region — whose seven admissible
`kind` tokens and per-kind facet keys exist nowhere in the type closure — would
have made the primary surface strictly less informative than the secondary, and
reproduced the slice's founding defect inside the artefact built to cure it. The
criterion was appended to record the departure rather than letting the output
quietly out-run it. Accepted.

**The evidence trail lives in commit messages rather than phase sheets for three
of seven phases.** Commits are authored and diffable where sheets are disposable
runtime state, so nothing was lost — `d7001375c` carries `PHASE-07`'s owed
help-diff bound in full (281 renderings, 251 byte-identical, 27 differing by a
restored trailing full stop, 3 by a PID in a pre-existing panic). But an auditor
cannot distinguish an unrecorded discharge from an unperformed one without
re-running it, which is what this audit did. Swept to notes.md; accepted as a
process cost, not a defect in the work.

**Two waived VTs at close** (`PHASE-02/VT-2`, `PHASE-06/VT-1`), both with recorded
reasons, both with a successor criterion carrying the real obligation. Waivers
are a cost; a criterion that cannot fail is a larger one.

### Not this slice's, but on the record

The primary worktree is on branch **`work`**, not `edge`. AGENTS.md's edge/main
split says the primary tree stays on `edge` and a feature is landed from a
worktree. This does not affect any finding — the code is identical whatever the
branch is called — but `/close` should resolve it deliberately rather than
discover it.

## Reconciliation Brief

### Per-slice (direct edit)

- **`slice-251.toml` selector registry — the load-bearing change for both
  conformance findings.** Prose alone leaves `doctrine slice conformance 251`
  red; the registry is what it reads.
  - `F-5`: `doctrine slice selector add` for `src/commands/cli.rs` (authorised by
    `PHASE-07/EX-10`; without the renderer change `EX-4`'s mechanism is inert)
    and for `src/design_run/artifact.rs` (PHASE-05 hoisted
    `cites_a_repo_private_id` to module level so PHASE-06's `VT-2` shares the
    detector rather than copying it — `STD-001`).
  - `F-6`: `doctrine slice selector rm` for `src/design_run/attestation.rs` —
    declared, never delivered, and `PHASE-02/EX-10` proved it never could be:
    all twelve closure structs are in `submission.rs`.

- **`design.md` — five claims execution disproved (`F-8`), plus the two
  conformance mirrors (`F-5`, `F-6`).** Each already has its evidence written and
  its convicting criterion cited; this is transcription, not re-adjudication.
  - `sec-4`: "All six are const fn, so the token array stays a const" — false for
    `Provenance` and `ReviewDisposition`; every variant of both carries a field,
    so a const initialiser must drop a temporary (E0493), and two cannot be
    constructed from the leaf at all. Those two take the naming arm; the single
    source is recovered at test time by `VT-1`'s per-variant walk.
    (`PHASE-01/EX-9`.)
  - `sec-7` touch table + `sec-8` pin 1: "those fixtures live in `submission.rs`
    and `attestation.rs`" — `attestation.rs` holds none of the twelve closure
    structs; it contributes the closure's *enums* plus `ReviewRef`.
    `design.md:600-601` already says "submission.rs (every wire struct)" and
    contradicts them. (`PHASE-02/EX-10`.) **Add `src/commands/cli.rs` and
    `src/design_run/artifact.rs` to the same touch table** — the `F-5` mirror.
  - `sec-8:2012-2016`: the single `Id`-row rule is wrong. `IdKind::declarable` is
    a predicate over *declaration subjects*, and `DelegationAct`'s `id` keys
    correctly declare `Id(&[IdKind::Delegation])` while `declarable(Delegation)`
    is false (`ids.rs:90`) — the rule fails four correct rows. Implemented as
    two: a per-row structural membership check, plus the two engine-derived
    claims asserted at their own sites. (`PHASE-03/EX-10`.)
  - `sec-6:1518-1522` + `sec-8` pin 7's help bullet: both reason from **clap's**
    help renderer. `main.rs:285-292` routes every `--help` through
    `render_subcommand_help`, which read `get_about()` alone; `long_about`
    occurred zero times in the tree. Sound about clap, wrong about doctrine.
    (`PHASE-07/EX-10`.)
  - `sec-6:1591`: `contract` as "a fifth variant beside the existing four" — it
    is the **sixth** (`Start`, `Show`, `Apply`, `Resume`, `Materialise` are five).
    (`PHASE-06/EX-1`.)

- **`ISS-333` — the discharge statement (`F-9`).** The duplicate is already
  merged (`ISS-346` closed `duplicate`, linked `related`), so closure can only
  land on one id. What is owed is the prose: SL-251 closes the **probing loop**
  (the contract is fetchable, so nobody must submit-and-read-the-refusal to learn
  the shape) and does **not** close the mechanism — verified live at this audit,
  a misspelt top-level key still bumps the revision and exits 0. Do not let the
  statement read as a fix.

### Governance/spec (REV)

- **`SPEC-029` `spec-029.toml:27` → REV modify (`F-7`).** "Front the capability
  through one command family — start, show, apply, resume" becomes the six-verb
  family: `start`, `show`, `apply`, `resume`, `materialise`, `contract`. A
  two-verb correction — the line was already stale by `materialise` before this
  slice shipped `contract`.

### Not brief items — recorded so reconcile does not go looking

- `F-1`, `F-2` — plan criteria. `plan.toml`'s `EN-`/`EX-`/`VT-` ids are
  immutable-append and are **not** a reconcile write surface. Both are already
  resolved at audit (`PHASE-06/VT-3` appended, `VT-1` waived with reason;
  `PHASE-07/VA-1` tolerated with its substance discharged in this ledger).
- `F-3`, `F-4` — harvest. Swept into `notes.md` at the audit tail.
- `F-10` — memory corpus. Corrected at source.
- `F-11`, `F-12` — tree hygiene. `git rm`'d at audit; `.claude/settings.json`'s
  trailing newline restored.

## Reconciliation Outcome

Written 2026-08-16, **after** `SL-251` closed. This ledger was authored at
04:31 on the capsule ref `refs/capsule/d/heads/work` and never merged; `RV-361`
audited the same slice on `edge` at 18:51 with no way to see it, and the slice
transitioned `done` at `0ea962268` under `RV-361` alone. The ledger was
recovered from the ref afterwards and landed here — re-minted as `RV-362`
because id 358 was already `SL-238`'s design review, so this is a fresh id
carrying the original twelve findings verbatim, not a merge. Its four delegated
findings were then discharged in a second reconcile pass. No finding text, no
disposition and no response was altered in the transfer; the only edits were id
repointing (`RV-358` → `RV-362`, `IMP-434` → `IMP-438`, `IMP-435` → `IMP-439`,
each a genuine same-kind collision with an item `edge` had already allocated).

### Already satisfied by RV-361's pass

- **`F-5`** — `doctrine slice selector add` for `src/commands/cli.rs` and
  `src/design_run/artifact.rs`. Landed as `RV-361` `F-3`/`F-4`, registry first
  and the `design.md` §7 touch table as its mirror.
- **`F-6`** — `doctrine slice selector rm` for `src/design_run/attestation.rs`.
  Landed as `RV-361` `F-2`, together with the second site in `sec-8` pin 1 that
  cross-referenced the same table.
- **`F-8` items 2 and 4** — the `attestation.rs` fixture attribution and the two
  clap-renderer passages (`sec-6` Point 3, `sec-8` pin 7). Corrected under
  `RV-361` `F-2` and `F-3`.
- **`F-9`'s prose half** — `ISS-333`'s discharge statement, written at
  `RV-361`'s pass and already carrying the distinction this ledger asks for:
  the probing loop is closed, the mechanism is not, and disclosure is not repair.
- **`F-11`** — the three stray root-level `capsule-*.md` duplicates. Already
  gone on `edge` by a different route (they now live at `install/agents/claude/`),
  and `.claude/settings.json`'s trailing newline is present.

### Landed by this pass

- **`F-1`** — `plan.toml`: `PHASE-06/VT-1` retired with its `waived_reason`, and
  `VT-3` appended carrying the same obligation keyed on `repo_root` rather than
  `CARGO_MANIFEST_DIR`. This is the one finding where the two ledgers reached
  different answers: `RV-361` `F-1` read the same FAIL, judged the substance met
  and disposed it `aligned`, which is defensible but leaves `slice verify-vt 251`
  reporting a red criterion on a closed slice. This pass took `RV-362`'s answer.
  `verify-vt` is now clean — two waivers, no FAIL. Append-only throughout: no id
  was renumbered and `VT-1` still exists, spent.
- **`F-7`** — `REV-054` (`reconcile-sl-251`), `done`. One `modify SPEC-029` row,
  surfaced for manual landing and landed by hand at `spec-029.toml:27`: the
  design command family reads *start, show, apply, resume, materialise, contract*.
  The line was stale by `materialise` before this slice added `contract`.
- **`F-8` items 1, 3 and 5** — three `design.md` direct edits, each marked
  *Corrected at reconcile* in place rather than rewritten over:
  - `sec-4` — "All six are `const fn`" is four; `Provenance` and
    `ReviewDisposition` carry a field on every variant (`E0493` in a const
    initialiser) and two of those variants are not constructible from the leaf,
    so both take the naming arm and the single source is recovered at test time
    by the per-variant walk. The division is four / eight / two (`PHASE-01/EX-9`).
  - `sec-8` — the single `Id`-row rule against `IdKind::declarable` is wrong:
    `declarable` is a predicate over declaration *subjects*, and `DelegationAct`
    correctly declares `Id(&[IdKind::Delegation])` while `declarable(Delegation)`
    is false, so one rule fails four correct rows. It is two rules
    (`PHASE-03/EX-10`).
  - `sec-6` — `contract` is the sixth `DesignCommand` variant, not the fifth;
    the "Governance touchpoints" bullet is corrected to match and now cites
    `REV-054` (`PHASE-06/EX-1`).
- **`F-9`'s structural half** — `ISS-346` linked `related` to `ISS-333` and
  closed `duplicate`, with the rationale written into its body, including the
  `Declaration`-denies-unknown-fields correction it also carried.
- **`F-10`** — `mem.fact.design-run.apply-payload-vocabulary` corrected at
  source. It claimed neither `ApplyRequest` nor `Declaration` denies unknown
  fields; exactly three of the twelve wire structs do
  (`submission.rs:123`, `:1042`, `:1094`), and that asymmetry is the whole of
  `ISS-333`.
- **`F-12`** — `.doctrine/workflows/drive-slice.js` removed, with the dangling
  `.claude/workflows/` link. `SL-254` `PHASE-10` deleted it deliberately,
  `18e35c2e5` resurrected the materialised copy, and `install.rs` says three
  times that no embedded source remains to regenerate it.
- **Carried out of the slice** — `IMP-439` (conformance is blind to commits
  between phase boundaries, the defect that hid `F-11` and `F-12`), plus three
  friction observation records from the audit session.

### Withdrawn / tolerated

- **`F-2`** — `PHASE-07/VA-1` void on contact, tolerated at audit with its
  substance discharged in the ledger. A misspelt *top-level* key produces no
  refusal to read, because `ISS-333`'s silent discard is an explicit Non-Goal of
  this slice; the criterion named an input the slice deliberately does not
  refuse.
- **`F-3`, `F-4`** — harvest, swept into `notes.md`. `RV-361` `F-5` covered the
  staleness; this pass added the six `PHASE-04..07` durable findings the first
  ledger had swept and the second never saw.

### Where the two ledgers disagree

Only at `F-1` / `RV-361` `F-1`, and only on remedy, not on fact: both read the
same FAIL, both established that `PHASE-06/EX-5`'s property is met and
mutation-verified in `cba1f257f`, and both concluded no code was wrong. `RV-361`
stopped there; `RV-362` also retired the spent criterion. Taking the stronger
remedy costs nothing and removes a standing red from a closed slice's evidence
surface.

Everywhere else the two agree, including where they overlap on the
`attestation.rs` trap, the clap-renderer correction, the undeclared `cli.rs` and
`artifact.rs` touches, and the discipline that `ISS-333` must not be recorded as
fixed. `RV-362` found five things `RV-361` did not (`SPEC-029`, the three other
`design.md` claims, the `ISS-346` duplicate, the memory falsehood, the orphaned
workflow); `RV-361` found three `RV-362` did not (the `IMP-434` id-collision
residue in the recorded source-deltas, the `skills-lock.json` drift inside
`PHASE-02`'s boundary, and `slice selector doctor`'s intent conflation, carried
out as `ISS-440`). Neither pass is a superset of the other, which is the
strongest available argument that the twelve findings were worth recovering.
