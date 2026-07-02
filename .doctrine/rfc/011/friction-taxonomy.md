# RFC-011 — Friction taxonomy & remediation ledger

> Contributed by an autonomous improvement pass (branch `fable-loop`, 2026-07-03).
> This is a **synthesis of `case-notes.md`**, not a replacement — `case-notes.md`
> stays the raw instrument. Each friction below is classified by root cause and
> paired with its disposition (fixed here / OBE / by-design / deferred). Commit
> refs are on `fable-loop`; the User cherry-picks what to keep.

## 1. Friction classes (the taxonomy)

The case-notes collapse to six root-cause classes. Token cost is dominated by the
first three: they force a *wrong action* or a *defensive re-derivation*, not just
a re-read.

| Class | What it is | Token failure mode | Case-note instances |
|---|---|---|---|
| **C1 CLI-shape guess-miss** | A command's real arg shape (flag vs positional, required flags) differs from how a skill/doc models it | Agent copies the shape → hard error → recovery round-trips | `slice phase --status` (positional-looking), `memory retrieve` (no positional key), `worktree land --fork` (branch vs path), `backlog edit` (docs said "prompts"), `arm-spawn --slice` (diagnostic-only) |
| **C2 Governance-mechanics gap** | A gate/algorithm's *granularity or surface* is undocumented, so an agent reasons at the wrong level | Wrong claim asserted, caught late (external review) → rework | ADR-001 edge granularity (top-level collapse); `slice conformance` reads the selector registry, not §6 prose |
| **C3 Brief/skill surface mismatch** | One skill emits an instruction naming a surface a *downstream* skill won't write | Downstream stalls, asks the human, or edits the wrong tier | reconcile brief named a `plan.toml` edit (immutable-append, off-surface); prose-vs-registry duality |
| **C4 Empty-body reverse-engineering** | An entity's actionable content lives only in the title / external memory, not its body | Multi-round scavenger hunt to reconstruct the task (~7k tokens for a 2-line change) | IMP-103 empty IMP body templates |
| **C5 State-vs-summary drift** | A compaction summary / snapshot lags on-disk reality | Defensive re-probing of true state on every resume | sl181 post-compact resume (~6 probes); stale boot Memory index |
| **C6 Env/provisioning** | A worktree/jail is missing a gitignored build input or has a baked path | Build/test fails opaquely before any real work | hand-created worktree lacked `web/map/dist/`; `cargo test` marker-worktree false-fails; `env!(CARGO_MANIFEST_DIR)` baked path |

## 2. Remediation ledger (this pass)

Fixed at root, each a clean commit, verified against code before editing:

| Friction | Class | Fix | Commit |
|---|---|---|---|
| ADR-001 edge granularity undocumented (drove a false tangle-safety proof) | C2 | Added a "Gate edge model" subsection: edges are top-level→first-segment, BTreeSet-deduped; sub-classification refines the direction check only, never the tangle ratchet. Verified vs `tests/architecture_layering.rs`. | `ceff6e90` |
| reconcile brief named an off-surface `plan.toml` edit; conformance prose-vs-registry duality | C3 | `/audit` brief-writing step forbids `plan.toml` items (immutable-append) and requires conformance findings to name the `slice selector` registry verb; `/reconcile` catches a bad item and hands back. | `a7e9ec0e` |
| `*/new` slug symlink stranded by path-scoped `git add <dir>/NNN/` | C1/C6 | `/close` commit guidance: stage `git add <kind>/NNN*`; check for a dangling `NNN-slug`. | `fe3eed9a` |
| `worktree land --fork` bare `no-such-fork` when a *path* was passed | C1 | Pure `no_such_fork_message` enriches that one refusal with a path-vs-branch hint; VT-golden token preserved (e2e `.contains`). | `e0e7b7b1` |
| `backlog edit` documented as "(prompts)" — actually requires `--status` | C1 | Corrected the verb table + two prose claims; `handover` `slice phase … in_progress` positional→`--status` flag. Audit verified all other verb shapes correct. | `305c8638` |
| `CoverageStatus` rendered via `{:?}` Debug (`InProgress`), asymmetric with kebab input | C1-adjacent | Single-source `as_kebab` formatter (= `parse_status` vocab) at both render sites; round-trip test locks the pair. | `07f9a4a2` |
| cordage `explain()` foreign node returned per-overlay singletons, rustdoc promised empty | correctness | In-range guard in `explain` (`node.0 >= node_count` → empty cone). | `1dacc7a8` |
| `next` value cell showed ABSENT for value-bearing kinds scored at default 1.0 | correctness | `value_cell` renders the effective default (`1.0*`) for value-bearing kinds, ABSENT for valueless; `DEFAULT_VALUE` single-sourced. The e2e golden had itself encoded the bug. | `e30e482e` |

## 3. Not fixed — and why

- **By-design (won't-fix):** worker confinement blocking authored `.doctrine/`
  writes (ADR-008); `cargo test` red under a worker marker (the marker IS the
  mechanism; the funnel's marker-cleared diff is the real check); `slice phase
  --status` / clippy `unwrap_used` denials / `References`-needs-`Role` (standard
  CLI/lint/relation shapes); the vtgate comment-keyword match (an accepted POL-002
  weakness — threat model is omission, not an adversary).
- **OBE (already fixed on `edge`):** `memory retrieve` phrasing (boot-footer
  reworded + `doctrine_onboard` MCP path); ISS-058 `append_edge` contiguity;
  and a whole block of recent IMP items (IMP-112/177/185/214, ISS-035). *Lesson:
  verify-before-trust — roughly half of the older case-note frictions were stale.*
- **Deferred (larger than one clean autonomous increment):** `arm-spawn --slice`
  coord-resolution guard (hot dispatch/SL-190 code); IMP-183 estimate/value
  render on non-slice show surfaces (signature churn across the backlog render +
  JSON parity + goldens — borderline slice-worthy); IMP-019 cordage `golden_net`
  independent value oracle (test hardening, no runtime bug); ISS-205 cordage
  `env!` baked path (no clean local red — passes when compiled in place).

## 4. Cross-cutting observations

1. **C1 dominates and is the cheapest to prevent.** A skill that cites a wrong
   command shape taxes *every* agent that copies it. A one-time audit of skill
   command citations against `--help` ground truth (done here — 22 skills, 4
   real traps) is high-leverage and repeatable as a lint.
2. **The `--help` text is the authority; skills drift from it.** Consider a CI
   check that extracts `doctrine …` citations from skill bodies and diffs them
   against the CLI's actual arg model.
3. **Goldens can encode the bug.** The IMP-211 `next` golden pinned the wrong
   output (a value-bearing row shown as `·` while scored 1.0). A golden asserts
   *current* behaviour, not *correct* behaviour — pair value-surface goldens with
   an independent oracle (cf. IMP-019's gap).
4. **Empty authored bodies (C4) are a silent tax.** The storage rule keeps bodies
   free-form, but a non-obvious item whose problem statement lives only in the
   title or an external memory forces reconstruction. Capturing the actionable
   statement in the body at creation time is the cheap fix (author discipline,
   not tooling).
