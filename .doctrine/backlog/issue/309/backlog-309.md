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

Sites: `install/` 141, `memory/` 24 — the audited pair; plus `plugins/` 74 when
the skills were brought into scope (see below). Of the `install/`+`memory/` 165,
**59 are `ADR-`**. The raw counts include the illustration classes ruled out
below, so they are an upper bound, not a violation count — the inventories that
follow are the real ledger. Per-file figures are *citation lines* as grepped, not
a partition of the site total.

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
- `manifest.toml` — `5` `SL-227`; `35` `SL-018`; `47` `SL-231`; `56` the path
  `install/using-doctrine.md`; `62` `SL-233`, `PRD-019`, `SPEC-023`.

### `install/` — config and scaffolding assets

Two of the highest-severity sites in the whole corpus are here, because they are
not prose *about* a concept — they are shipped config files a client reads
directly and edits.

- **`doctrine.toml`** — `3` `ISS-055`; `11` `ADR-009 §2`. This is a *projected
  base backing*: it lands at `.doctrine/doctrine.toml` in every client repo, so
  the client's own config file ships doctrine-private ids in its comments.
- **`doctrine.toml.example`** — `5` `ISS-055`; `9` `SL-028` + `ADR-009 §2`; `36`
  `ADR-003 §8`; `48` `SL-148`; `83` `SL-108`; `89` `SL-166`; `100` `SL-198`;
  `113` `SL-254`. Here each id is doing real work — it is often the only thing
  naming *why* a knob exists — so inlining the fact matters more than deleting
  the id.
- `design-prompts/delegation.md` `19` — `IMP-483`.

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

### Scope extension 2 — `plugins/` (the skills)

Not in the original sweep. Brought in when scoping SL-267, on this evidence:

- **74 sites across 14 skill files** — comparable to `install/` + `memory/`
  combined, and on the most context-resident shipped surface of the three, so
  excluding it would fix the corpus agents read least and skip the one they read
  most.
- **All genuine, none illustrations** — and in places worse than a collision.
  `spec-product/SKILL.md` `150-151,208,229` instructs the reader to mirror
  `PRD-001` as "the canonical shape", which in a client repo is *the client's
  own PRD-001*. `spec-tech/SKILL.md` `56-61` teaches the C4-level spec taxonomy
  by pointing at doctrine's own `SPEC-003`/`SPEC-004`/`SPEC-005`. Misinstruction,
  not merely an unresolvable reference.
- **Already governed, and violating the rule.**
  `mem.pattern.doctrine.shipped-skill-platform-independence` (glob `plugins/**`)
  bans exactly this, in writing. So this sub-corpus is a *rule known to exist and
  broken anyway* — the strongest evidence available for part 2 of the fix, since
  the rule's presence demonstrably did not prevent the drift.

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
patched the symptom. It recurred elsewhere regardless.

It then recurred *during this audit*: while the ledger above was being written, a
concurrent in-flight edit to `install/design-prompts/delegation.md` introduced a
fresh `IMP-483` citation into that shipped asset — in good faith, citing the
author's own originating item, with no signal available to them that the citation
is wrong in a client repo. Recorded as observation
`01a0d8f4-89dc-7832-b52b-ba4e9d62ef68`; it lived in another agent's uncommitted
work, so that record is a snapshot rather than a landed fact.

That is the case for part 2 of the fix below, and it is the strongest evidence
available for it: a written POL-002 rule for a sibling sub-corpus, a fully
enumerated defect list, and an audit still in progress did none of the work of
preventing the next occurrence. A defect invisible to the author writing it will
be re-introduced by the next author, however well the existing instances are
swept.

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

   **Proving the sweep worked is not "the id is gone."** Removing a citation and
   grep-confirming zero hits proves the deletion ran, not that the reader is now
   correctly served. This repo has already been bitten by exactly that confusion
   (`mem.pattern.doctrine.reseat-renumbers-does-not-retarget`: a renumber is not a
   retarget, and a zero-hit search is a claim about the search). The evidence
   that matters is that the *replacement* resolves in a client repo — a published
   `reference/<name>.md` address, or prose that stands without an id. Read the
   post-sweep text as a client agent with no corpus; do not count removed ids.
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

## Status — part 1 delivered by `SL-267`; part 2 open (2026-09-26)

**Part 1 (the sweep) landed** in `SL-267` (shipped-corpus conformance). All three
sub-corpora were swept (`install/`, `memory/`, `plugins/`) and the vocabulary
question above was settled: the grounding rule — inline prose, a published
`reference/<name>.md` address, a shipped memory key, a skill name, a CLI verb, or
an in-corpus relative path where the target sits beside the citing file — is
recorded once as the accepted `ADR-024`. Evidence: `SL-267`'s `notes.md`
(three per-axis ledgers + the client-read control) and audit `RV-395`.

**Ledger corrections, for part 2** — the inventory above is a floor, not a bound:

- The id regex omits the knowledge-kind prefixes (`CON`/`EVD`/`HYP`/`CPT`),
  membership labels (`FR-`/`NF-`), and the whole doc-local-id dimension
  (`D-N`, `F-N`, `INV-N`, `S-N`, `PHASE-NN`).
- The `plugins/` inventory is stale by growth: 35 `SKILL.md` on disk, ~104 sites
  across 16 files — not "14 files / 74 sites" — and its "none illustrations"
  claim is wrong (five sites).
- The `memory/` denominator is 13 masters, not 11.

**Part 2 (the check) remains open.** A second, *distinct* integrity gap was
found while closing `SL-267` and is filed as `IMP-487`: a skipped re-embed or
`memory sync` leaves the **materialised** corpus stale while every gate stays
green. Part 2's slice should carry both checks — content (this item) and
freshness (`IMP-487`) — as they do not subsume each other.
