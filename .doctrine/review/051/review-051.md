# Review RV-051 — design of SL-086

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**The Inquisition arraigns the design of SL-086 — four small CLI changes that
touch the most commonly invoked surfaces in the agent's toolchain: `memory find`,
`memory retrieve`, and a new `doctrine status` dashboard. Small changes cast long
shadows; the design's silence on a default must be tortured into confession.**

The Inquisitor presses four principal lines of attack:

1. **The `find` default-limit heresy.** The design's common resolution block
   applies `RETRIEVE_LIMIT_DEFAULT` (5) to both `find` and `retrieve`, silently
   capping `memory find` at 5 results where it currently shows all. Neither the
   scope, nor the design decisions, nor a single acceptance criterion
   acknowledges this behavioural regression. If the design intends to default-
   limit find, the intent must be inked; if not, the code must be absolved.

2. **Boot staleness schism.** D13 declares a clock-based signal (`fresh` ≤ 5 min)
   with `age_seconds` in the JSON output. But the existing vessel —
   `boot::boot_check` — computes staleness as a *content diff* (recomputed vs
   on-disk), with no clock at all. The design conflates two signals into one
   display without a reconciliation rule. A file may be content-fresh at 10
   minutes old, or clock-fresh but content-drifted. What does `staleness:
   "fresh"` mean?

3. **`flag_query` — the unnamed flag.** The design renames the `--query` field to
   `flag_query` to avoid a positional name collision, but omits the necessary
   `#[arg(long = "query")]` attribute. Without it, clap generates `--flag-query`
   — a silent CLI break. The design also omits `flag_query` from the explicit
   `FindRetrieveArgs` extraction list, risking duplication across Find and
   Retrieve.

4. **Dependencies the design cannot see.** The `status` command references
   `priority::surface::next_work` — a function that does not exist (the actual
   surface exports `next`). The `args-struct ceiling` rationale conflates clap
   struct field count with the `too_many_arguments` function-arg lint. And
   `--page` / `--offset` mutual exclusion with `default_value_t = 0` on `offset`
   rides a clap behaviour subtlety the design does not interrogate.

The accused shall answer for every charge. The ledger shall record every
confession.

## Synthesis

**VERDICT: The design is substantially sound but tainted by seven heresies — one
blocker, two major, four minor. The core architectural choices (FindRetrieveArgs
extraction, D1–D13 decisions, pure/impure split for status) stand. But the
design has looked upon the abyss of silent behavioural regression and failed to
cry out.**

The **blocker** (F-1) is a mortal danger: the design's shared pagination
resolution silently caps `memory find` at 5 results, changing behaviour for
every `memory find` invocation in the agent toolchain. The scope says nothing
about this; no VT guards it. Confessed and absolved: the design must split
find's limit resolution (unlimited by default) from retrieve's (5 by default).

The **major heresies** (F-2, F-3) concern boot staleness and flag naming — two
domains where the design's pen strayed from the existing code's truth. The boot
staleness design (D13) invents a clock-based signal that does not exist in the
current `CheckReport`; the `flag_query` rename omits the `#[arg(long = "query")]`
attribute that would preserve the user-facing flag name. Both confessed — fix
the design text and the code follows.

**Minor taints** (F-4 through F-7) are imprecisions and underspecifications that
a diligent implementer would catch, but the design should not make them catch
anything. Correct `next_work → next`, clarify the `args-struct ceiling` rationale,
acknowledge the `default_value_t + conflicts_with` subtlety, and guard `--limit
0`. Each is a single-sentence fix; none hides a deeper architectural problem.

### Ordered penance (corrective sequence)

1. **F-1 (BLOCKER)**: Split the limit resolution — `find` defaults to unlimited
   (`None`), `retrieve` retains `RETRIEVE_LIMIT_DEFAULT` (5). Add VT: 'find
   without --limit shows all results'. Update design §2 code blocks.

2. **F-3 (MAJOR)**: Add `#[arg(long = "query")]` to `flag_query` in design §1.
   Add `flag_query` to the FindRetrieveArgs extraction list in §6.

3. **F-2 (MAJOR)**: Reconcile boot staleness — let content-diff (CheckReport.
   stale) drive the primary signal; keep `age_seconds` as informational. Update
   D13 and the BootSection design.

4. **F-4 (MINOR)**: Correct `priority::surface::next_work → next` in the
   data-sources table.

5. **F-5 (MINOR)**: Correct the `args-struct ceiling` rationale — the extraction
   is for DRY, not for `too_many_arguments` (the functions already suppress it).

6. **F-6 (MINOR)**: Add design note acknowledging `default_value_t +
   conflicts_with` interaction. Add VT: '--page 2 alone does not error'.

7. **F-7 (MINOR)**: Add `--limit 0` guard (reject with clear error).

### Standing risks

- The `status` command depends on `git log` in the impure shell — the same
  `src/git.rs` seam. No new risk, but the dependency should be tested (git not
  installed / not a repo). The design does not specify git-absent behaviour.
- The `#[command(flatten)]` on enum variant fields with a positional arg in the
  same variant — clap handles this correctly, but it's a less-travelled derive
  path. The golden tests are the proof.

### Consciously tolerated

- `run_find` / `run_retrieve` function parameter counts remain past the
  `too_many_arguments` ceiling after all additions — the existing `#[expect]`
  attribute handles it. A future slice may extract a `FindRetrieveInput` struct
  for the function layer too, but that is out of SL-086 scope.
- The `MemoryFindRow` struct and `format_find_json` function will be dead code
  until wired — use per-symbol `#[expect(dead_code)]`.

---

The Inquisition has spoken. The design shall be purified; the code shall not be
written until the penance is complete. Let the record show that SL-086's design
came to this tribunal whole and left confessed — the stronger for it.

> **HERESIS URITOR; DOCTRINA MANET**
