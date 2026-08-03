# RFC-025 · C3 ingestion probe — the sixteen hazard rows

Companion to `README.md`; the authority is `results-c3.tsv`. Citation legend and
the limits on all of this are in `README.md` — read them first.

**M-A** = fetch into quarantine. **M-B** = bundle. Both fixtures are real
repositories: **light** (a small TypeScript project) and **heavy** (a ~169 MB
clone of the doctrine repository itself). `dissolution` means the hazard has no
refusing stage *by construction* — the model gives it nowhere to land, which is a
result, not a gap (R-D).

## Result

**All sixteen rows carry scored results. Every scored cell is `pass`.** The
matrix is complete; the two `n/a` cells and one capped altitude are structural
and explained below.

| row | hazard | boundary | light M-A/M-B | heavy M-A/M-B | altitude |
|---|---|---|---|---|---|
| H1 | result committed on a history rebased off B — pinned OID is not a descendant of the contracted base | `conform/ancestry-not-descendant` | pass / pass | pass / pass | model-level |
| H2 | ref moved between the doorbell and the read | **dissolution** | pass / pass | pass / pass | model-level |
| H3 | a merge commit inside the result range | `conform/ancestry-merge-commit` | pass / pass | pass / pass | model-level |
| H4 | edit outside the contracted slice's design-target selectors | `conform/undeclared-path` | pass / pass | pass / pass | model-level |
| H5 | a governance-path touch — plain edit, non-ASCII path, rename out | `conform/forbidden-path` | pass / pass | pass / pass | model-level |
| H6 | capsule sets `core.hooksPath` and writes executable hooks | **dissolution** | pass / pass | pass / pass | model-level |
| H7 | a blob over the sandbox disk cap, plus a pathological deep tree | `harvest/resource-cap` | pass / pass | pass / pass | model-level |
| H8 | a `160000` gitlink at a declared path, plus a `.gitmodules` | `conform/gitlink`, `conform/gitmodules` | pass / pass | pass / pass | model-level |
| H9 | a symlink out of the tree; a `.gitattributes` naming a filter driver | **dissolution** at ingest + containment at `verify` | pass / pass | pass / pass | model-level |
| H10 | two results from the same base B; the second meets stage 4's CAS precondition | `advance/stale-base` | pass / pass | pass / pass | model-level |
| H11 | a hostile **test file** at a declared path, run by the fixture's own suite | `verify` (light) · `conform/undeclared-path` (heavy) | pass / pass | pass / pass | model-level |
| H12 | env-file execution surfaces (`.envrc`, `flake.nix`) | **dissolution** | **n/a** / **n/a** | pass / pass | **unproven-beyond-rust** |
| H13 | a hostile bundle — absent, invalid, unsafe path, oversized | `harvest/bundle-*`, `harvest/resource-cap` | **n/a** / pass | **n/a** / pass | model-level |
| H14 | doorbell abuse — duplicate ring, lost ring, spoofed ring | `harvest` (tokenless) | pass / pass | pass / pass | model-level |
| H15 | the harness killed at each stage in turn | **dissolution** | pass / pass | pass / pass | model-level |
| H16 | accepted ref advanced after B was pinned, before harvest | `advance/stale-base` | pass / pass | pass / pass | model-level |

## The rows that need a sentence

**H2 — dissolution, and the reason is the interesting part.** The harvester pins
the OID *itself*, then fetches, then compares — so its guard covers only the
window between its own `rev-parse` and its own fetch, inside one process. A
mutate upstream of both reads makes pin and fetch agree, and git offers no seam
that moves a ref *during* a fetch. There is no deterministic instantiation on
M-A and the reason is structural, not a rig limitation (F-P05-13).

**H5 — three forms, and they mask each other.** Conform leg 3 returns on the
first match, so the three forms each have to clear leg 2 separately
(F-P05-18/22). The rename-out form has **no heavy instantiation**: heavy's sole
design-target `.doctrine/` prefix is empty at B and leg 3 reads a two-dot tree
diff (F-P05-21) — which is why guard (c) is light-only.

**H9 — scored as two entries.** Inert at ingest (no tree is materialised
trusted-side, I4) *and* containment at `verify`. Two boundaries, two entries;
collapsing them would lose one.

**H11 — the fixtures refuse at different stages, and that is correct.** On light
the hostile test file sits at a declared path and reaches `verify`; on heavy it
does not, and dies earlier at `conform/undeclared-path` (F-P05-33: heavy's
instantiation has no reachable trigger under its own selectors). The observable
is sentinel-absent on the **host** path — inside the capsule `/tmp` is a tmpfs
(F-P04-12) — never absence of error output. **The network half is measured and
NOT scored**: the verify capsule shares the network namespace by construction
(F-P05-32, D-P05-14/15).

**H12 — the one capped altitude.** The light fixture has no `.envrc` and no
`flake.nix`, so there is nothing to plant. A **structural** absence (F-7), and it
caps the row at `unproven-beyond-rust` (A-3): the hazard held on heavy, but
portability beyond a Rust project is not established. Both light cells are `n/a`
for that reason and are excluded from the altitude computation. H12 could not be
instantiated as authored — both its eval surfaces needed re-homing (F-P05-26).

**H13 — the asymmetry that is itself a verdict input.** Four legs on M-B, all
pass on both fixtures. On M-A the row has no subject at all: it reads no
capsule-authored artifact, so the `n/a` is structural (F-6). This is **EVD-010**,
the `disputes` edge against the bundle's "cleanest trust story".

**H14 — a tokenless `harvest` refusal, deliberately.** Three legs: a duplicate
ring (a no-op by content-addressing, I2), a lost ring (polled to the wall-clock
deadline), and a spoofed ring naming another capsule — content is never read, so
it carries no authority.

**H10 / H16 — stage 4's two halves.** H10 lands on the CAS *precondition* after a
first result has landed; H16 advances the accepted ref after the contract pinned
B and before harvest, landing on the precondition from the other side. Both take
the strict `assert_outcome` clause because nothing transferred (F-14). Getting
stage 4's ordering backwards reds exactly here. Each also owns a
**`counts-toward-nothing`** candidate-layer leg — see `guards.md`.

## Falsification

Every row's assertions were falsified before the row was believed: mutants were
planted that *should* red each clause, and each redded its own clause and nothing
else. The discipline that produced it, and the three occasions it caught a
vacuous clause, are F-P05-38 (*a clause that cannot fail is not a control*),
F-P05-30, F-P05-22 and F-P05-42.

**One gap, recorded rather than repaired.** T4a–T4e's falsification drivers were
never git-tracked and are gone (F-P05-39). The scored `results.tsv` is whole, so
the evidence stands; what was lost is re-runnability of those specific mutants.
**Do not reconstruct them from prose and re-run them under the old claims'
names** — an inferred contract's green is new evidence wearing an old label. T5's
and T6's rounds *are* committed and re-runnable, in `drivers/`.

## Two operator findings left open

Both are in `src/`, which PHASE-05 held untouched (S4).

- **ISS-305** — on the candidate layer a *conflict* refusal is ledgered
  (`candidate create` exits **zero**; verdict is `status="conflicted"`) while a
  *staleness* refusal is status-borne. The clap help asserts the opposite
  (F-P05-40).
- **`harvest/fsck-failed` stands for two causes**, and the harvester fscks the
  whole quarantine rather than the range it ingested (F-P05-28, EVD-007).

## Cost, for whoever runs this next

A heavy cell is **~1.5–2.5s**, not the minutes it was feared to be —
`git clone --no-hardlinks` of the 169 MB fixture reflinks in 0.2s (F-P05-10). A
full heavy verify running a real suite end-to-end is 376–413s; that cost is the
client project's tests, identical under either mechanism. The heavy **worker**
capsule measures 195 MB against a 256 MiB cap — ~24% headroom, so a row planting
more than ~60 MB trips `harvest/resource-cap` for a reason about the fixture
rather than the model.
