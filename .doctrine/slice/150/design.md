# Design SL-150: Family-grouped help + boot-map projection

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

`doctrine --help` is a flat, unsorted list of ~44 top-level commands — opaque to
humans, no routing structure for agents. `--help --commands` (IMP-166) adds full
per-kind verb tables but is ~150 lines, too heavy to inline on every agent boot;
most of its bulk is the repeated CRUD spine (`new/list/show/paths`), high-token /
low-marginal-information.

Onboarding to the command *surface* must be generated from the clap command tree
(else it drifts from reality). One tree must feed: a scannable human reference
and a dense, routing-grade boot map. This is a tokens-vs-breadth/clarity exercise.

## 2. Current State

- `src/commands/cli.rs`
  - `render_top_level_help(color, term_width)` — walks `Cli::command()`
    subcommands, renders a flat 2-col `command | description` comfy-table via
    `listing::render_columns`. No grouping.
  - `render_commands_table(color, term_width)` — 3-col `command | verb |
    description`, every subcommand's verbs grouped beneath it, descriptions
    first-sentence-truncated. This is `--commands`; stays as the lazy full
    reference.
  - `first_sentence(about)` — first-sentence truncation helper (reused).
- `src/listing.rs` — `render_columns` (comfy-table seam: `Column`, `RenderOpts`,
  `ColumnPaint`, `force_no_tty`). Gains `render_grouped` + a shared
  `paint_full_width_band` / row-start probe.
- `src/search.rs` — `write_context_snippet` (~322), `is_table_row_start` (~317),
  `SNIPPET_FG/BG`: the full-width-band precedent. Refactored to call the lifted
  `listing` primitive (DRY); its existing snippet behaviour must stay green
  (behaviour-preservation).
- `src/main.rs` ~185 — intercepts top-level `--help`; `--commands` switches
  between the two renderers (gains the `--boot-map` arm).
- `src/boot.rs`
  - `boot_sequence()` → ordered `(heading, SourceKind)` pairs.
  - `SourceKind` enum {Static, Governance, GovRows, Memories, Footer, ExecPath}.
  - `produce(heading, kind, root, exec)` → one `Section` body per kind.
  - `render_boot(sections)` — deterministic, byte-stable; empty → `marker()`.
  - `ExecPath` is **last** (build-volatile; confines path churn to the cache
    tail).
- Renderings go through comfy-table; `force_no_tty` is mandatory for golden
  determinism ([[mem.pattern…comfy-table custom_styling…force_no_tty mandatory]]).

## 3. Forces & Constraints

- **One source of truth** — both renderings + the boot map walk the same
  `Cli::command()` tree. No second enumeration of commands.
- **Minimal hand-maintained data** — only a family table + a spine set; the
  drift-guard test is what makes hand data safe.
- **Boot byte-stability** — the snapshot is a governance contract every agent
  loads; `doctrine boot` is idempotent and `boot --check` must stay clean
  (cf. IMP-123 byte-unchanged tests). The command-map section is build-stable
  (changes only when commands change), so it does not threaten the
  `ExecPath`-last cache invariant.
- **ADR-005** (shipped knowledge tiered; PUSH tier is compact, PULL explains):
  the boot map is PUSH-tier — dense, no glosses; the human `--help` and
  `--commands` are PULL-tier reference. The governing relation. **Why the map
  earns the PUSH seat** (the cost ADR-005 demands be defended, not assumed): the
  boot already PUSHes the Routing *table* — which skill governs which situation —
  but an agent that has routed still cannot name the *command* to run without a
  `--help` round-trip or guesswork. The command surface is the Routing table's
  natural companion: both are navigational PUSH context. It earns the seat only
  because the factored spine + infra suppression keep it to ~20 lines — without
  that compression it would be the ~150-line `--commands`, which correctly stays
  PULL. Memory stays PULL because its corpus is unbounded and task-specific; the
  command surface is bounded and universal. The factoring is what makes PUSH
  affordable here.
- **POL-002** (platform independence): no host-project assumptions; pure CLI
  surface work.
- **Pure/imperative split**: classification + rendering are pure functions over
  the clap tree; disk/exec stay in boot.rs's shell.

## 4. Guiding Principles

- The redundancy *is* the lever: factor the CRUD spine once, surface only
  semantic-bearing verbs.
- Families are navigational headers, not data.
- Auto-derive over transcription: distinctive verbs are computed (`verbs −
  spine`), never hand-listed.
- Two renderings, one tree — generated, so they cannot drift from the commands
  they describe; only the family taxonomy is asserted by a test.

## 5. Proposed Design

### 5.1 System Model

```
                 Cli::command()  (clap tree — the one source)
                        │
        ┌───────────────┼────────────────────────────┐
        │               │                             │
 render_top_level_help  render_boot_map        render_commands_table
   (human --help)        (boot map)              (--commands, unchanged)
   per-family sub-tables  dense PUSH-tier         lazy full reference
        │                  │        │
        │            SourceKind::    doctrine --help --boot-map
        │            CommandMap       (main.rs intercept)
        │            (boot.rs section)
   FAMILIES table + SPINE set  ◄── shared classification (cli.rs)
        │
   drift-guard unit test: FAMILIES ⟷ clap tree is a total partition
```

### 5.2 Interfaces & Contracts

New in `src/commands/cli.rs`:

```rust
/// A navigational grouping of top-level commands. Header-only; the members are
/// command names matched against the live clap tree.
struct Family { key: &'static str, members: &'static [&'static str] }

/// A navigational grouping of top-level commands. `suppress_verbs` keeps the
/// family's commands header-only in the boot map (infra is operational /
/// skill-driven, not boot-time authoring routing — D7). The flag rides the
/// struct so the suppression is compile-linked to the family, not matched by a
/// separate stringly-typed key list (F-4).
struct Family { key: &'static str, members: &'static [&'static str], suppress_verbs: bool }

/// The 8-family taxonomy. The ONLY hand-maintained classification; the
/// drift-guard test asserts it partitions the visible clap subcommands exactly.
static FAMILIES: &[Family] = &[
    Family { key: "change",     suppress_verbs: false, members: &["slice","revision","rfc","rec","review","reconcile","coverage"] },
    Family { key: "governance", suppress_verbs: false, members: &["adr","policy","standard","spec"] },
    Family { key: "knowledge",  suppress_verbs: false, members: &["memory","knowledge","backlog"] },
    Family { key: "relations",  suppress_verbs: false, members: &["link","unlink","needs","after","supersede"] },
    Family { key: "facets",     suppress_verbs: false, members: &["estimate","value","risk","tag"] },
    Family { key: "reports",    suppress_verbs: false, members: &["status","next","blockers","survey","explain"] },
    Family { key: "explore",    suppress_verbs: false, members: &["search","inspect","relation","concept-map","map"] },
    Family { key: "infra",      suppress_verbs: true,  members: &["install","boot","serve","config","validate","reseat",
                                           "export","reservation","worktree","dispatch","catalog"] },
];

/// Verbs every entity kind shares; subtracted to leave the distinctive set.
/// `status` is deliberately NOT in the spine — not universal, lifecycle-bearing,
/// so it surfaces as distinctive where present.
const SPINE: &[&str] = &["new", "list", "show", "paths"];

pub(crate) fn render_top_level_help(color: bool, term_width: Option<u16>) -> String; // rewritten — calls listing::render_grouped
pub(crate) fn render_boot_map() -> String;                                            // new, plain text
```

New in `src/listing.rs` (the shared comfy-table seam):

```rust
/// Render groups from ONE underlying table (columns size across all rows ⇒
/// shared-width alignment; color/wrap retained), with a full-width heading BAND
/// injected into the line stream before each group's rows (blank/key/blank,
/// painted edge-to-edge, color-gated). The flat `render_columns` is the
/// degenerate single-group case.
pub fn render_grouped<T>(groups: &[(&str, Vec<T>)], cols: &[&Column<T>], opts: RenderOpts) -> String;

/// Shared full-width band primitive — lifted from search's inline `--context`
/// painter (DRY). Pads `content` to `opts.term_width` and paints fg-on-bg when
/// `opts.color`; plain otherwise. Used by `render_grouped`'s heading band AND
/// `search`'s context snippet.
pub fn paint_full_width_band(content: &str, fg: DynColors, bg: DynColors, opts: RenderOpts) -> String;
```

- `render_boot_map()` takes no color/width: the boot snapshot is plain,
  width-independent text (it is `@`-imported, not terminal-rendered). Determinism
  by construction — no tty consult, no wrap.
- `main.rs`: extend the `--help` intercept — `--boot-map` → `render_boot_map()`
  (precedence: `--boot-map` before `--commands`; `--boot-map` wins if both
  given — see §5.5).

New in `src/boot.rs`:

```rust
enum SourceKind { …, CommandMap }                 // new variant

// boot_sequence(): insert right AFTER "Routing & Process"
("Commands", SourceKind::CommandMap),

// produce(): new arm — pure, infallible, build-stable
SourceKind::CommandMap => Section { heading, body: cli::render_boot_map() },
```

### 5.3 Data, State & Ownership

- **Owned, hand-maintained**: `FAMILIES` (incl. each family's `suppress_verbs`
  flag) and `SPINE` (cli.rs). Everything else is derived from the clap tree at
  render time.
- **Derived at render**: distinctive verbs per command (`verbs − SPINE`),
  whether a command is a leaf, command descriptions (`about` / first-sentence).
- No new persisted/runtime state. The boot-map body is regenerated each
  `doctrine boot`; it is derived content living in the gitignored snapshot.

### 5.4 Lifecycle, Operations & Dynamics

**Human `--help` (one table, full-width banded family headings, color + wrap):**
```
                                              ┃ blank band line (bg-coloured)
 change                                       ┃ family band — bg colour, full width
                                              ┃ blank band line (bg-coloured)
  slice       Create and list slices — the unit of intentional change
  revision    Create, show, and transition revisions
                                              ┃ blank band line
 governance                                   ┃ family band
                                              ┃ blank band line
  adr         Create and list architecture decision records
  …
```
(Bands paint edge-to-edge over the column structure; command rows keep the
shared table columns. Colour off ⇒ bands degrade to plain text + blank lines.)
- D8: rendered from a **single underlying comfy-table** of all command rows (so
  columns size across every row — shared-width alignment is automatic, and
  `comfy-table` color paint + `term_width` wrapping are retained), with
  **full-width family-heading bands injected into the rendered line stream** at
  each group boundary — NOT 8 separate tables, and NOT in-table heading *rows*
  (a band paints edge-to-edge, over where column separators would sit, so it
  cannot be a celled row).
- **Heading band styling** (per user steer — mirrors `doctrine search --context`,
  `src/search.rs:322` `write_context_snippet`): the family key on a **distinct
  background colour**, padded to the full terminal width so the band spans
  edge-to-edge, with a **blank band-coloured line above and below** (each heading
  = 3 painted full-width lines: blank, `  {family}`, blank). Colour gated on
  `render.color` — piped / `force_no_tty` output drops the paint to clean text
  (the byte-golden path). `term_width` unknown ⇒ no padding (plain), as search
  already handles (`_ => content`).
- **Injection mechanism**: post-process the table's output lines — walk them,
  detect logical row starts (`is_table_row_start`, `src/search.rs:317`), map the
  Nth row to its command's family, and emit the heading band when the family
  changes. This is exactly the `--context` snippet injection, reused.
- **DRY (house rule — no parallel implementation)**: the full-width-band
  primitive (collapse → pad-to-`term_width` → paint, colour-gated) and
  `is_table_row_start` already live inline in `src/search.rs`. Lift them into a
  shared `listing` helper (e.g. `paint_full_width_band(content, fg, bg, render)`
  + the row-start probe); `search` and the grouped-help heading band both call
  it. Heading-band colours are their own constants (distinct from search's
  sage/green); the *primitive* is shared, the palette is not.

**Boot map (`render_boot_map`, dense PUSH-tier):**
```
SPINE: new list show paths (+status where lifecycle) — entity kinds

change      slice revision rfc rec review reconcile coverage
  slice       design plan phases phase notes selector status
  review      raise dispose verify contest withdraw prime status
  revision    change approve apply status
  coverage    show record verify forget
governance  adr policy standard spec
  spec        req validate
knowledge   memory knowledge backlog
  memory      record find retrieve verify validate edit tag status resolve-links backlinks sync
  backlog     edit needs after tag
relations   link unlink needs after supersede
facets      estimate value risk tag
reports     status next blockers survey explain
explore     search inspect relation concept-map map
infra       install boot serve config validate reseat export reservation worktree dispatch catalog
```
Rules:
- Family header line: `{key}  {member member …}` — all members, bare.
- Sub-line per command **iff** it has distinctive verbs AND its family's
  `suppress_verbs` is false. Leaves and infra never sub-line (already named in
  header).
- Spine declared once at the top.

**Boot integration:** `doctrine boot` → `produce(CommandMap)` →
`render_boot_map()` → section body. Build-stable ⇒ idempotent ⇒ `boot --check`
clean.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 Total partition**: every visible (`!is_hide_set`, name ≠ `help`)
  top-level command belongs to exactly one family. Drift test enforces both
  directions.
- **INV-2 No phantom members**: every name in `FAMILIES` resolves to a real
  command in the clap tree. (Catches typos / removed commands.)
- **INV-3 Boot stability**: `render_boot_map()` is a pure function of the
  compiled clap tree — no clock/rng/disk/tty — so two runs are byte-identical;
  the snapshot only changes when commands/verbs change.
- **INV-4 Deterministic ordering**: families render in `FAMILIES`-declared
  order; commands within a family render in member-array order (NOT clap
  `get_subcommands()` order) — so goldens do not depend on derive order. Verbs
  within a command's sub-line render in clap order (stable derive order).
- **EDGE — `--boot-map` + `--commands` together**: `--boot-map` wins
  (documented); mutually-exclusive not enforced (matches the loose
  `--commands`/plain coexistence today).
- **EDGE — leaf command** (no subcommands, e.g. `search`, `reconcile`, `link`):
  appears in family header only; no sub-line in boot map; one `command |
  description` row in human help.
- **EDGE — command with only spine verbs**: distinctive set empty → no boot-map
  sub-line (correct: nothing distinctive to say).
- **EDGE — hidden / `help`**: excluded from classification and the drift test
  denominator (mirrors existing render filters).
- **ASM — clap tree enumerable**: `render_commands_table` already walks
  subcommands+grandchildren; the same seam serves classification and the test.

## 6. Open Questions & Unknowns

- **OQ-1 (D8 mechanism) — RESOLVED: option (b), single comfy-table + heading
  rows.** The "8 sub-tables can't share widths" problem only arises if you render
  8 tables. Don't: render **one** comfy-table with family-heading rows interleaved
  between groups. One table sizes columns across all rows ⇒ shared-width
  alignment is free, and color paint + `term_width` wrapping survive. Family
  heading = a styled row (`[key, ""]`). Adds a `listing::render_grouped` helper.
  Drops the earlier plain-text decision for human help (boot map stays plain
  text). Exact heading-row styling + optional group separators confirmed at
  execute (phase 1).
- *(Resolved: D1 static table, D2 auto-derive, D3 `--boot-map` flag + CommandMap
  section, D4 family-grouped help (one table + heading rows), D7 suppress infra
  verbs, D8 shared widths, OQ-1 single comfy-table via `render_grouped`
  (color/wrap retained), OQ-3 `--boot-map` name.)*

## 7. Decisions, Rationale & Alternatives

- **D1 — static `FAMILIES` table** (vs clap `help_heading` per subcommand). clap
  does not group *subcommands* under headings in derive cleanly; a static table
  gives full control of all three renderings and is made safe by the drift test.
- **D2 — auto-derive distinctive verbs** (vs hand-list per command). One fewer
  hand-maintained dataset; spine-factoring becomes a computed property. A
  differently-named create verb (`memory record`) correctly surfaces as
  distinctive.
- **D3 — `--boot-map` flag + `SourceKind::CommandMap`**, one `render_boot_map()`
  behind both. The flag is near-free, gives the golden a black-box target, and
  lets an agent pull the map without reading the snapshot file. Named
  `--boot-map` (not `--map`) to avoid overloading the `map` command (OQ-3).
- **D4 — family-grouped human `--help`** (vs single flat / single
  family-column table). Families are navigational headers. Rendered as ONE
  comfy-table with styled family-heading rows interleaved (not 8 tables) — keeps
  color + wrap + automatic shared-width alignment (D8, OQ-1 (b)). Human help is
  not token-budgeted, so it keeps the richer terminal rendering; the dense
  plain-text projection is the boot map's job, not the human help's.
- **D5 — CommandMap ordered after "Routing & Process"** — navigational, and
  build-stable so it does not disturb the `ExecPath`-last cache invariant.
- **D7 — suppress infra verb-expansion** — infra is operational / skill-driven,
  not boot-time authoring routing; header names every infra command, no verb
  dump (keeps `worktree`/`dispatch`'s 10+ verbs out of the snapshot).
- **D8 — shared column widths across sub-tables** — vertical divider alignment
  across the 8 family sections; a UX requirement from the design conversation.

## 8. Risks & Mitigations

- **R1 Boot snapshot churn / cache invalidation** — new section could move the
  cache boundary. Mitigation: place high (after Routing), build-stable content,
  `ExecPath` stays last; assert byte-stability + `boot --check` clean.
- **R2 Taxonomy drift** — a new command added without a family entry. Mitigation:
  INV-1/INV-2 drift test fails CI until classified (the core safety net).
- **R3 Golden brittleness** — comfy-table tty/wrap nondeterminism. Mitigation:
  `force_no_tty`; `render_boot_map` emits plain text (no comfy-table), so the
  boot-map golden is trivially stable.
- **R4 Scope bleed into IMP-135** (help copy consistency). Mitigation: non-goal;
  this slice only regroups + projects, does not rewrite descriptions.

## 9. Quality Engineering & Validation

- **Drift-guard unit test** (pure, over `Cli::command()`), three assertions —
  set equality alone is insufficient (a command in two families dedups in the
  union and passes):
  1. `FAMILIES` members contain **no duplicate** (build a name→family map; a
     second insert for a name is a collision failure) ⇒ single-family membership.
  2. every member resolves to a real visible command ⇒ INV-2 (no phantom).
  3. every visible (`!is_hide_set`, ≠ `help`) command appears in some family ⇒
     INV-1 (no orphan).
- **Golden: human `--help`** — black-box via `CARGO_BIN_EXE_doctrine`,
  `force_no_tty` (color off ⇒ bands are plain text + blank lines), byte-exact;
  asserts family order, banded grouping, and shared column alignment (D8). A
  separate color-on smoke check (not byte-golden — escape codes) asserts the
  band paint + full-width pad are present.
- **Search behaviour-preservation** — the `--context` snippet refactor onto the
  shared `paint_full_width_band` primitive keeps `src/search.rs`'s existing
  snippet tests green unchanged (the band-painting move is internal, output-stable).
- **Golden: `--help --boot-map`** — byte-exact boot-map text; asserts spine line,
  header+sub-line rule, infra suppression (D7), leaf handling.
- **Boot byte-stability** — `doctrine boot` twice ⇒ identical; `boot --check`
  clean with the CommandMap section present (extends IMP-123-style assertions).
- **SPINE guard (F-5)** — the family drift test does NOT guard SPINE validity;
  the **boot-map golden is the SPINE guard**: rename a spine verb (`list`→`ls`)
  and every kind's verb surfaces as distinctive, changing the golden output —
  the golden fails, forcing a deliberate look. Stated so the coupling is
  explicit, not accidental. (Cheaper belt-and-braces if wanted: assert each
  SPINE verb appears under ≥2 entity kinds.)
- **Behaviour preservation + boot blast radius (F-2)** — existing
  `--commands` golden / `render_commands_table` unchanged. Existing boot tests
  in `src/boot.rs` survive the CommandMap insertion *by construction*, but are
  named here as the accounted set:
  - `boot_sequence_orders_exec_path_last` — survives (ExecPath still last).
  - `boot_sequence_orders_active_policies_after_accepted_adrs` /
    `…active_standards_after_active_policies` — assert ADR→Policy→Standard
    *adjacency*; survive because CommandMap inserts after "Routing & Process"
    (before Governance), not between the governance sections.
  - `render_boot_is_byte_deterministic_and_structured` — synthetic sections,
    unaffected.
  No unit test embeds the full real snapshot byte-for-byte, so there is no
  hidden golden to regenerate. **New** tests this slice adds: a
  `produce(CommandMap)` arm test, and an ordering test pinning CommandMap
  immediately after "Routing & Process" and before "Governance".

## 10. Review Notes

Internal adversarial pass (pre-inquisition):

- **F1 (fixed)** — INV-1 drift test by set-union alone passes a command listed
  in two families (union dedups). Hardened to 3 assertions incl. a no-duplicate
  collision map (§9).
- **F2 (fixed)** — within-family ordering was unspecified; pinned to
  `FAMILIES`-declared order via INV-4 so goldens don't ride clap derive order.
- **F3 (fixed → OQ-1 upgraded)** — D8 shared-width sub-tables fight comfy-table's
  per-table autosize; underrated as "low risk". Now MEDIUM with two concrete
  options; plain-text grouped help is the lean.
- **F4 (resolved)** — `--map` flag overloaded the `map` command name. Renamed
  to `--boot-map` (OQ-3).
- **F5 (accepted)** — boot-map golden churns on any verb addition; intended
  (surface changes get an explicit review), documented as maintenance cost (R/9).
- **F6 (no change)** — mid-sequence CommandMap insertion shifts following
  sections' byte offsets once on first regen; one-time, content-stable
  thereafter; acceptable (R1).

Formal inquisition (RV-153, facet design, raiser inquisitor) — 5 charges, all
fix-now, verified: F-1 PUSH-tier defence (§3); F-2 boot blast radius enumerated +
defused (§9); F-3 slice-Context spine contradiction struck; F-4 `suppress_verbs`
folded into `Family` (§5.2); F-5 boot-map golden named as the SPINE guard (§9).

Post-inquisition revision (user steer): OQ-1 reopened — human `--help` keeps
**color + comfy-table** (not the plain-text fallback). Resolved (b): one
underlying comfy-table (shared widths free, color/wrap retained) via a new
`listing::render_grouped`, with full-width **family-heading bands** injected into
the line stream — distinct bg colour, edge-to-edge, blank band line above/below
(mirrors `search --context`). The full-width-band primitive + row-start probe are
**lifted from `search.rs` into shared `listing` helpers** (DRY); search refactors
onto them, behaviour-stable. Boot map stays plain text.
