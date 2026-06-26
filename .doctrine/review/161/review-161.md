# Review RV-161 — design of SL-155

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This Inquisition interrogates the design of SL-155 against its own scope document,
the sanctioned conventions, and the principle that design tells truth, not
aspiration. The accused design claims six one-liners + one list verb as "targeted
edits" but contradicts the scope it purports to fulfil on a key visibility
decision, adds un-scoped surface, and leaves integration decisions unsaid. The
tribunal shall press on:

1. **D2 scope-design contradiction.** The scope says tags are NOT default columns
   ("following governance precedent"); D2 makes them default, dismissing scope's
   stated precedent as "a lag." Does design overrule scope without acknowledging
   the deviation? Or is this unresolved conflict itself the heresy?

2. **Un-scoped surface expansion.** The design adds a `slug` column to the default
   visible set. Scope never mentions slug; GOV_COLUMNS doesn't include slug.
   Where is the sanction for this addition?

3. **Design silence on integration.** The design specifies `RevisionCommand::List`,
   `run_list`, and `list_rows` but never speaks to the existing `revision`
   subcommand group structure. Does `List` live alongside `Show`, `Status`,
   `Change`, `Approve`, `Apply`, `Paths`? Is the CLI dispatch shape coherent
   with the existing command tree?

4. **Migration silence.** Existing revision TOMLs lack a `tags` field. The design
   adds `#[serde(default)]` on `tags: Vec<String>` — correct, but never
   addresses whether the `rev_scaffold` template-only change is sufficient to
   ensure `render_revision_toml` doesn't regress existing files. The
   test `render_revision_toml_includes_tags` asserts the template renders
   `tags = []`, but no test proves an existing revision without tags round-trips.

5. **Scope attribution error unchallenged.** The scope's "Affected surface" lists
   `src/spec.rs — C2 (two template comment fixes)` but C2 items are template
   files, not `src/spec.rs`. The design should correct this misattribution,
   not silently accept it.

6. **IMP-144 partial tagging.** The design wires REV into `TAGGABLE` (I1) and
   adds `tags` to `RevDoc`, but the scope says `revision show` tag rendering is
   "future work in IMP-170 G2." The design's only show-surface mention is
   `EX-06` (JSON includes tags). Is the design's tag surface for show
   intentionally partial, or an accidental lacuna?

The Inquisition expects each decision to be traceable to the scope or to carry an
explicit deviation rationale. Decisions that silently reverse the scope, add
surface the scope excludes, or leave integration unsaid are presumptively
heretical until the accused confesses or the evidence vindicates them.

## Synthesis

**Judgement: GUILTY OF HERESY, CONFESSED AND SENTENCED.**

The design of SL-155 has been tried by the Inquisition on six counts and found
wanting on all. The accused has confessed to every charge and accepted penance.

### The Root Heresy

The mortal sin is **D2's contradiction of scope**. The scope document explicitly
states tags are opt-in via `--columns`, following governance precedent. D2
reverses this, making tags default-visible, and dismisses the stated precedent as
"a lag." A design may improve upon scope, but it must acknowledge the deviation
and justify the break. To silently reverse a scope provision and dismiss its
rationale is to make design and scope bear false witness against each other — a
schism that would poison the audit. **The fix** (`fix-now`): D2 must either (a)
restore scope-consistent non-default tags with a note that governance precedent
is being followed, or (b) record an explicit deviation with rationale and amend
the scope to match.

### The Wounds That Fester

Three major findings expose sins of omission:

- **F-2 — Un-scoped `slug` column.** The column exists in `REV_COLUMNS` without
  scope sanction. Remove it or justify it.
- **F-3 — CLI integration silence.** The design names `RevisionCommand::List` but
  never shows it within the existing command tree. A reader cannot tell where
  `List` lives among `New`, `Show`, `Status`, `Change`, `Approve`, `Apply`,
  `Paths`. Add the integration context.
- **F-6 — Missing migration guard.** `#[serde(default)]` is correct for
  deserialization, but no test proves a tagless revision round-trips through the
  renderer without corruption. Add a round-trip test.

### The Minor Taints

- **F-4 — Silent scope error.** The scope misattributes C2 to `src/spec.rs`. The
  design correctly maps items to files but should note the scope error.
- **F-5 — IMP-144 show deferral unacknowledged.** The scope defers tag rendering
  in `revision show` to IMP-170 G2. The design's `EX-06` covers JSON only;
  whether the prose `show` output omits tags is unstated. Acknowledge the
  deferral explicitly.

### Ordered Penance

1. **Resolve D2 first** — alignment on tag visibility governs the list contract.
   Amend scope or design; the two must not contradict.
2. **Add CLI integration context** — show where `List` sits in the
   `RevisionCommand` enum and the dispatch `match`.
3. **Justify or remove the `slug` column** — dead surface is unworthy surface.
4. **Add round-trip test** for tagless revisions through `render_revision_toml`.
5. **Correct scope attribution** for C2 in design notes.
6. **Acknowledge show-surface deferral** for IMP-144 tags.

### Standing Risks

- The implementation is already half-built (scout confirms `list_rows`, `run_list`,
  `RevDoc.tags`, `TAGGABLE` all exist; only CLI wiring is missing). If D2's
  resolution changes the column contract, existing column definitions must follow.
- The design's contradiction with scope creates an audit poison pill: if not
  resolved before audit, the scope-design mismatch will surface as an RV blocker
  at close time.

**Let the design be purified by fire and corrected doctrine. The Inquisition has
spoken.**

> **HERESIS URITOR; DOCTRINA MANET**
