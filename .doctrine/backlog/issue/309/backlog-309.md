# ISS-309: Shipped assets cite repo-private ids that collide in client repos

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The defect

Doctrine's shipped corpus — everything embedded under `install/` and published
through `doctrine library show` — cites **this repository's own entity ids and
file paths**. Those assets are read by agents working in *client* repos, where
neither resolves correctly.

The path half is an ordinary broken link. The id half is worse, and is the
reason this is an issue rather than a tidy-up.

**Entity ids are per-repo sequential.** A client repo mints its own `DEC-101`,
its own `SL-233`, its own `ADR-007`, counting from its own installation. So a
shipped asset citing `DEC-101` does not dangle in a client repo — `doctrine
knowledge show DEC-101` returns *a different, unrelated record*, with no error
and no signal that the citation was never about that record. A broken link
announces itself. This does not: it silently substitutes one claim for another,
in guidance an agent is being asked to act on.

## Enumerated ledger (swept 2026-09-26)

No longer a sample. Every citation site in the shipped corpus was enumerated —
`grep -rnE '\b(SL|ADR|RV|REV|PRD|SPEC|IMP|ISS|CHR|DEC|RFC|REQ|POL|STD|IDE|QUE|RSK|ASM|REC|CM)-[0-9]{3}\b'`
over `install/` and `memory/` — then classified. Counts are citation *sites*, not
distinct ids. Line numbers are as-swept; the id is the durable anchor.

### Scope extension — `memory/` was omitted and is in scope

The original scope read "everything embedded under `install/`". The **shipped
memory corpus is a shipped asset by the same test**: RustEmbed-embedded,
materialised into `.doctrine/memory/shipped/`, retrieved by agents in client
repos. It carries the same defect — a `[[mem.signpost…]]` citation of `ADR-007`
fails silently in exactly the way `install/review-ledger.md`'s does. It was
outside the stated scope, which is why the original sample missed it.

### Raw totals (verified)

Sites: `install/` 140, `memory/` 24, both 164. Of those, **59 are `ADR-`**. The
raw counts include the illustration classes ruled out below, so they are an
upper bound, not a violation count — the inventories that follow are the real
ledger.

### `install/` — published reference docs

Site-by-site (`file:line` → citation → classification):

**`claude-activation.md`** — all six lines are violations.
`1-2` shared header (`ADR-005`, `ADR-019`, path `install/claude-activation.md`);
`5` and `10` `SL-250` (*"since SL-250 the plugin/marketplace path retired"* — the
fact survives without the id); `56` `SL-195` + `POL-002`; `60` `SL-195`.

**`dispatch-mechanics.md`** — all seven lines are violations. `1-2` shared header;
`6` `CHR-036` (and CHR-036 is the chore that *produced* this doc — an author
note leaking into the artifact); `138` `ISS-234`; `312` `SL-211`; `369-370`
`ADR-006`/`ADR-008`/`ADR-011`/`ADR-012`.

**`harvest.md`** — `1-2` shared header, and `91` `ADR-005` used as the *name of a
rule* (*"ADR-005 conformance"*). Lines `55`, `62`, `63`
(`IMP-241`/`ISS-102`/`DEC-011`/`QUE-023`) are fill-in-the-blank scaffolding —
**not** violations.

**`review-ledger.md`** — `1-2` shared header; `9` `ADR-007`; `100` `SL-147`;
`163` `ADR-007`; `168` `RFC-026 P10 trial`; `219` `IMP-098`; `220` `ISS-314`.
Two are worse than a collision: `RFC-026 P10` is unrecoverable for a client (both
the id and the internal proposal number), and `ISS-314` is a *live open issue in
this repo* — the reader is told a defect exists in `derived_status`, in code they
do not have.

**`routing-process.md`** — `77` `ADR-019`; and `70`, **the sharpest finding in
the corpus**: `SL-233`'s `OQ-1`, `RV-325` `F-4` is the *worked example* of the
rule "introduce a doc-local id qualified by that artefact's durable id", in a
clause whose whole premise is that the reader has not memorised ids. The example
commits the error the rule forbids. Lines `65-66`
(`SL-023`/`ADR-005`/`REQ-059`) are the reference-form illustration — **not** a
violation.

**`using-doctrine.md`** — `1-2` shared header; `52` `PRD-010`; `179` `ADR-004`;
`182` both `src/relation.rs` and `ADR-010` — a private source path in a published
doc.

### `install/` — projected integration assets

These land at a stable path in the client repo, so the failure is not merely
unresolvable — it is resolvable to the *wrong* client entity.

- `git-hooks/pre-commit` — `2` `SL-228 PHASE-02`; `6`, `15`, `24` `ISS-234`
  (once in operator-facing stderr a client will actually read).
- `agents/claude/dispatch-worker.md` — `36` `src/worktree/mod.rs`.
- `design-prompts/exploring.toml` `1` `SL-233 PHASE-16`/`DEC-101`, `17`
  `IMP-372`.
- `design-prompts/inquiring.toml` `1` `SL-233 PHASE-08`/`DEC-101`, `8` `DEC-104`
  / `RV-325 F-5`; `21` also cites `sketches/thin-adapter.md`.
- `design-prompts/reviewing.toml` `1` `SL-233`/`DEC-101`, `15` `DEC-103`, `22`
  `IMP-373`.
- `design-prompts/drafting.toml` `1` `SL-233`/`DEC-101`, `6` `DEC-104` /
  `RV-325 F-2`.
- `design-prompts/reviewing.md` `100`, `105` `RFC-026 P10 trial`.
- `templates/` — `ADR-004 §5` in all seven `knowledge-*.toml`; `ADR-004` +
  `SL-048 "the cut"` in `backlog.toml:14,16` and `backlog-risk.toml:20,22`;
  `SL-048`/`SL-060`/`SPEC-018`/`ADR-010` in `slice.toml:14,16,18,19`; `ADR-007 D-C8`
  + `ADR-004` in `review.toml:4,12`; `SPEC-002` in `rec.md:3`; `SPEC-002 F7/D7`
  and the internal plan term `(Slice B)` in `rec.toml:4,12`; `ADR-013` in
  `revision.toml:9`, `revision.md:3`, `review.md:3`.
- `manifest.toml` `56` cites `install/using-doctrine.md`.

**The hard case — `sketches/thin-adapter.md`.** Its only real home is
`.doctrine/slice/233/sketches/thin-adapter.md`. There is no correct client-repo
spelling, so it cannot be fixed by repathing: the reasoning it carries is
load-bearing for two runbook steps, so it must move into the shipped corpus or be
inlined.

### `memory/` — shipped memories (11 of 35 keyed items)

- `mem.signpost.doctrine.dispatch` — `26` `ADR-002`/`ADR-005`; `28` `CHR-036`;
  `31-32` `ADR-006`/`ADR-008`/`ADR-011`/`ADR-012`. Also carries a **dangling
  wikilink** to `mem.signpost.doctrine.dispatch-claude-arm-wrong-base` — a key
  that does not exist in the corpus.
- `mem.signpost.doctrine.file-map` — `19` `ADR-013`; `37` `ADR-005`.
- `mem.signpost.doctrine.revisions` — `3` `ADR-013`.
- `mem.signpost.doctrine.reference-docs` — `25` `ADR-005`.
- `mem.signpost.doctrine.review` — `4` `ADR-007`.
- `mem_019ec92b30127db…` — `13` `ADR-007`.
- `mem.pattern.doctrine.core-loop` — `24` `ADR-009`.
- `mem_019e9a12789f7ac…` — `21` `ADR-009`.
- `mem_019e9a1244d37f…` — `12` `src/git.rs`.
- `mem_019f176f71537d…` — `10` `src/slice.rs` `rec_discharges` (private path
  *and* private function name); `36-39` `SL-165`, `REQ-316`, `REV-014`,
  `REQ-317`, `REC-093`, `REC-094`, `SL-064`. This one already carries a
  **hand-added disclaimer** — see below.
- `mem_88193c2859d72f…` — `72` *"Point of truth:
  `.doctrine/spec/tech/023/spec-023.md` (SPEC-023 …)"*. **The sharpest memory
  violation**: it names a private spec as a point of truth, so a client agent is
  directed to a document that does not exist in their repo and told it is
  authoritative.

### Not violations — do not sweep these up

The sweep must be surgical. Three classes look like hits and are correct:

- **Reference-form illustrations.** `glossary.md`'s kind↔abbr table,
  `routing-process.md`'s `SL-023`/`ADR-005`/`REQ-059` list, the
  reference-forms headers in `templates/*.md`, and the commented payload examples
  in `templates/{spec-product,spec-tech,members,interactions}.toml` are the docs
  that *define* what an id looks like. A client's own `PRD-001` is the right
  referent. (`glossary.md:44` is a grey case — it teaches form using ids that
  happen to be real here. Prefer obviously-synthetic shapes; a wording pass, not
  a sweep.)
- **Client-structure references.** `install/glossary.md:116`,
  `install/project-orientation.md:49`, `install/templates/seed-onboarding.md:50`
  reference `.doctrine/spec/` and `.doctrine/adr/` as directory conventions, and
  the `install/*.md` names cited inside templates (`adr-nnn.md`, `phase-01.md`,
  `handover.md`, `research.md`) are *the client's own* files. The distinction
  throughout is *citing the client's structure* versus *citing this repo's
  contents*.
- **Fill-in-the-blank scaffolding.** `harvest.md:55,62-63` and similar.

### The hand-patched instance is the argument for the check

`mem_019f176f71537d…` already needed an in-body disclaimer to stop its citations
misleading a client — someone hit this defect before it was written down, and
patched the symptom. It recurred elsewhere regardless. That is the case for
part 2 of the fix below: a defect invisible to the author writing it will be
re-introduced by the next author, however well the existing instances are swept.

### Asymmetry worth not relying on

`ADR-` citations may be *less* wrong in practice than `DEC-`/`SL-`/`ISS-` ones,
because a client repo is less likely to have minted thirty ADRs. Two reasons that
does not license skipping them: "less likely to collide" is not a property to
rely on, and it fails silently in exactly the same way when it does; and at 59 of
the 164 raw sites it is the **single largest class**, so the cheap fix ("ADRs are
probably fine") would leave most of the defect standing.

## Why it has gone unnoticed

Nothing checks it. Publication validation confirms that a declared asset has a
backing and that a backing is reachable; it has no view of whether the asset's
*prose* refers to anything a reader can resolve. And the failure mode is
invisible from inside this repo, where every citation resolves perfectly.

## Shape of the fix

Two parts, and the second is what stops the drift returning.

1. **The sweep.** Decide per citation: inline the fact (usually right for a
   one-clause rationale), drop it, or replace it with something a client can
   reach — a published address under `reference/`, or a description that stands
   without the id. `sketches/thin-adapter.md` is the hard case: the reasoning it
   carries is load-bearing for two runbook steps, so it needs a home in the
   shipped corpus or an inlined summary.
2. **A check.** A lint over the shipped corpus refusing repo-private entity-id
   and path citations, run by `doctrine doctor` / `check gate`. Without it the
   corpus re-drifts on the next asset, because the defect is invisible to the
   author writing it.

Open: whether shipped assets should be able to cite *anything* durable, and if
so what the vocabulary is. A stable published address (`reference/<name>.md`) is
resolvable everywhere and is the obvious candidate; entity ids are not, in any
form, because the namespace is per-repo by construction.

## Origin

Surfaced during `SL-244`'s design run, from the constraint that decided `DEC-127`
— a client repo has no access to this repo's spec, so any repo-external consumer
or citation must reference an ensured-up-to-date published copy rather than a
private artefact. `DEC-127` establishes the rule for the asset it was deciding;
this item owns the existing corpus and the check.
