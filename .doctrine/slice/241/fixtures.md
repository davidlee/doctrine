# Fixture build sheet — SL-241

The BUILD SHEET, not design. Design § 5.3 settled the design questions via
DEC-101 and D5; this file says what to type. Concrete literals throughout — if a
row here does not tell you the path, the command, or the byte to change, it is
not doing its job.

**Naming every fixture in one place is what stops four variants quietly
collapsing into one** (EX-10). Each entry carries its base, its delta over that
base, and the criterion that consumes it.

**This sheet is AMENDED, never rewritten** (EX-11). A later phase adding a
variant appends here; it does not restate F1–F5.

---

## Where fixtures live

```
$SPIKE_CAPSULE_ROOT/            default ~/capsules — a rig PARAMETER (EX-8),
                                out of repo by operator ruling
  fixtures/
    heavy/
      repo/                     F4 — the clone
      interpretation-surface.txt
    light/
      repo/                     F1 — the ledger project
      interpretation-surface.txt
    light-inrepo/               F2 (PHASE-05 T1) — F1 plus a committed
      repo/                       in-repo copy of the declaration
      interpretation-surface.txt
    light-plan/                 F3 (PHASE-05 T1) — F1 plus a plan whose
      repo/                       PHASE-01 is driven to `completed`
      interpretation-surface.txt
  probes/
    r2/repo/                    R2 probe apparatus (PHASE-01 T7), disposable
```

The `interpretation-surface.txt` files sit **beside** each repo, never inside
it. That is the F-5 provenance invariant made structural rather than asserted,
and F2 is the variant that deliberately undoes it.

Authored sources live in the rig at `scripts/spike-capsule/fixtures/<name>/`.
Provisioners are `scripts/spike-capsule/control/fixture-{heavy,light}.sh`.

**I6 runs first in every provisioner**, before any directory is created. A
mis-resolved root is the failure the guard exists to prevent, and a guard that
runs after `mkdir` is decoration.

---

## F1 — light base · BUILT (PHASE-01 T5)

| | |
|---|---|
| **base** | none — built from `scripts/spike-capsule/fixtures/light/` |
| **build** | `control/fixture-light.sh [--force]` |
| **lands at** | `$SPIKE_CAPSULE_ROOT/fixtures/light/repo` |
| **consumed by** | PHASE-01 EX-4/EX-5/EX-12 · PHASE-03 EX-11 (happy-path self-test) · PHASE-04 step 0 · PHASE-05 light column |

`ledger`: a TypeScript money utility, dependency-free and offline — Node 26
strips types natively and `tsc` comes from the environment, so there is no
`npm install` and no network in the provisioning path.

Deliberately unlike this repo (D5), because a fixture conventioned like this
repo would pass every row for reasons that say nothing about portability:

- trunk branch **`mainline`**, not `main` or `edge`
- commit subjects **`[add] …`**
- `package.json` scripts: `build` `clean` `test` `lint` `format` — all five are
  RUN at provision time, not merely read out of the file
- git identity pinned local: `ledger fixture <fixture@spike-capsule.invalid>`

History is exactly four commits, and the red→green is **observed**:

```
1  [add] ledger scaffold — npm scripts, strict tsconfig
2  [add] failing test for cents conversion and rendering     ← npm test MUST fail here
3  [add] cents conversion, rendering, and a formatter        ← npm test MUST pass here
4  [add] doctrine install, scratch slice SL-001, design-target selectors
```

Doctrine state: `doctrine install --yes`, one slice **SL-001** carrying the
single `design-target` selector `src/**`.

`tsconfig.json` excludes `src/**/*.test.ts`. The tests import `node:test` and
`node:assert/strict`, which `tsc` cannot resolve without `@types/node`; pulling
that in would mean a `node_modules` and a network round-trip. Nothing
type-checks the tests and nothing needs to — Node runs them directly.

---

## F2 — light + IN-REPO declaration copy · BUILT (PHASE-05 T1)

| | |
|---|---|
| **base** | F1 |
| **delta** | the declaration installed **inside** the repo and committed — a fifth commit, `[add] in-repo interpretation-surface copy — the F-5 exposure, made live` |
| **build** | `control/fixture-light.sh --variant inrepo [--force]` |
| **lands at** | `$SPIKE_CAPSULE_ROOT/fixtures/light-inrepo/repo` |
| **consumed by** | PHASE-05 EX-11, VA-3 · guard probe (e) · **QUE-201's only evidence input** |

**As built (PHASE-05 T1).** One provisioner, three variants — `--variant`
selects the delta, and a second script would have forked the whole
red→green→install sequence to change what happens after it. Both copies are
installed from the **same** authored source (`fixtures/light/`), so
"byte-identical at provision" is a property of the build rather than of a second
edit. Provision asserts three things the probe load-bears on: the copy is inside
the repo, it is **tracked** (a capsule clones the repo, so an untracked copy is
an exposure no capsule can see), and the two copies are byte-identical.

In the rig as drawn the F-5 substitution attack is **not live**: the declaration
sits outside the repo, so a capsule cannot rewrite it. This variant
*manufactures* the exposure, simulating QUE-201 candidates 1 and 2 (a
`doctrine.toml` block and a dedicated manifest both live in the repository the
capsule clones).

The probe: a capsule rewrites the in-repo copy — narrowing `interpret:` so the
payload-planting audit looks clean, or replacing `verify:` with a no-op so the
attested run attests nothing — and **trusted-side behaviour must be
byte-identical** to the run against F1. The invariant under test is that the
declaration is read from the contracted base `B`, never from the harvested
result `S`.

Without this variant QUE-201 settles post-spike on argument alone.

---

## F3 — light + plan + phases · BUILT (PHASE-05 T1)

| | |
|---|---|
| **base** | F1 |
| **delta** | SL-001 gains `plan.toml` with at least one phase, that phase driven to `completed` |
| **build** | `control/fixture-light.sh --variant plan [--force]` |
| **lands at** | `$SPIKE_CAPSULE_ROOT/fixtures/light-plan/repo` |
| **consumed by** | PHASE-05 EX-15, EX-6, VA-4 — the H10/H16 conflict sub-probe |

Design § 5.3's "a fixture needs no plan and no phases" is scoped to **the
pipeline**, and is true there because `prepare-review`'s phase-completion gate
is out of it. The H10/H16 sub-probe runs `prepare-review` explicitly — "staging
and all" — so it meets that gate and needs a fixture that satisfies it.

Provisioned **up front** rather than on demand: discovering it mid-phase costs a
full rebuild.

Scored separately and **counts toward nothing** (EX-6) — the sub-probe leg is an
incumbent-layer regression check, not capsule-model evidence.

**As built (PHASE-05 T1), and the one thing T5 must ride.** The scaffolded
PHASE-01 is left as `doctrine slice plan` authors it — empty `name`/`objective`,
`validate` clean — because the criterion is a phase that exists and completes,
not a phase that says something. What matters is where the completion LIVES:
`.doctrine/state/slice/001/phases/phase-01.toml` is **runtime state and
gitignored**, so `phases: 1/1` is a property of *this directory*, not of the
repository. A sub-probe that cloned the fixture first would find no completed
phase and could read `prepare-review`'s refusal as a finding about the candidate
layer, which would be a fact about the fixture. Provisioning asserts both halves
— the plan is tracked, the tracking is **not** — so the constraint is stated by
the build rather than discovered by the probe. Build sequence:

```
slice plan 1 → slice phases 1 → slice status 1 started
             → slice phase 1 PHASE-01 --status in_progress → --status completed
5  [add] plan for SL-001 with PHASE-01 driven to completed
```

---

## F4 — heavy base · BUILT (PHASE-01 T4)

| | |
|---|---|
| **base** | this repository |
| **build** | `control/fixture-heavy.sh [--base <rev>] [--force]` |
| **lands at** | `$SPIKE_CAPSULE_ROOT/fixtures/heavy/repo` |
| **consumed by** | PHASE-01 EX-3/EX-5 · PHASE-04 P-C1a cost baseline · PHASE-05 heavy column, H11/H12 |

```
git clone --no-hardlinks -- <repo> $ROOT/fixtures/heavy/repo
git -C … switch --detach <B>
```

**`--no-hardlinks` is not an optimisation knob.** A local clone hardlinks object
files by default, so the fixture and canonical would SHARE them, and a hostile
fixture corrupting a shared object corrupts canonical. It is the difference
between a copy and an alias.

Nine assertions at provision, all observed holding on the first real build
(101M of its own packs, pack inode link count 1):

- HEAD at exactly `B`; no remotes; not shallow
- commit count equal to the source's at `B`; `merge-base --is-ancestor <root> HEAD`
- `.git/objects/info/alternates` absent or empty
- `credential.helper` empty on the **effective** cascade — asserting the clone's
  *local* config would pass vacuously, since `clone` never populates it. The
  host's value is reported before being neutralised, so the assertion checks the
  neutralisation rather than the absence of a thing that was never there.
- declaration present as a sibling, and **absent inside the clone**

Declaration (`design.md` § 5.2's literal, verbatim):

```
exec:      cargo nix direnv just rustc
interpret: build.rs flake.nix .envrc rust-toolchain.toml **/build.rs
verify:    cargo test
```

`interpret:` names a **surface, not an inventory**: this repo carries no
`build.rs` and no `rust-toolchain.toml` today. `build.rs` is listed because H11
plants one.

---

## F5 — per-row instantiations · NOT BUILT (PHASE-05)

Not a fixture. The hostile rows mutate F1 and F4 **in place, per cell**, and
`probes/c3/matrix.tsv` is the source of truth for the full row set (PHASE-05
EX-1). Recorded here so the four the criterion names have a stated home:

| row | instantiation | on | expected |
|---|---|---|---|
| **H5** | `.doctrine/`/`.claude/` touch, in three forms: a plain edit, a **non-ASCII** path (`.doctrine/naïve.md`), and a **rename out of** `.doctrine/` | F1, F4 | refusal at conform **leg 3** — see below |
| **H8** | a `160000` gitlink entry at a **declared** path, plus a `.gitmodules` | F1, F4 | refusal at conform leg 4 (tree mode) |
| **H11** | hostile `build.rs` (F4) / `postinstall` script (F1) writing outside the workspace, plus a network canary | F4, F1 | sentinel absent, canary unreached |
| **(e)** | capsule rewrites its own in-repo declaration | **F2** | trusted-side behaviour byte-identical |

`Hnn.{mutate,planted,assert}` per cell, and **`Hnn.planted?` is mandatory**
(EX-3): a cell that passes because there was nothing to plant computes as "holds
under both" and stamps the row `model-level` — the strongest claim in the table
— on a probe that never ran. `n/a` is a legal recorded outcome, with a reason,
excluded from the altitude computation. A silent pass is not.

**H5 is why leg 3 exists, and PHASE-01 T7 proved it must.** `slice conformance
--against … --strict` has no `.doctrine/`/`.claude/` predicate at all: probed
against a slice that DECLARES a `.doctrine/` path, a touch and a rename-out both
read **conformant at exit 0**, while the import belt refuses `doctrine-touch`.
A pipeline that ran only `--strict` would score H5 as "no kill" — R4 in its most
damaging direction, since it would read as a defect of the capsule *model*.
Details in `notes.md` § R2.

Edges 3–6 of the R2 probe are already the positive controls this row needs, and
are re-runnable: `control/probe-r2.sh`.

---

## Not fixtures

- **The per-run quarantine repository** (PHASE-03 EX-2) is created and destroyed
  per pipeline run. Real, separate, disposable, `fetch.fsckObjects=true` — never
  a namespace inside canonical.
- **`probes/r2/repo`** is PHASE-01 T7 apparatus: a disposable clone of F1, so
  the base fixture stays pristine (EX-11). It builds its own SL-002 (declaring
  `.doctrine/probe/**`) and SL-003 (no selectors), which exist only to separate a
  forbidden-path refusal from an undeclared-path one.
