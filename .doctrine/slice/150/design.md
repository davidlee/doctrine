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
- `src/main.rs` ~185 — intercepts top-level `--help`; `--commands` switches
  between the two renderers.
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
  `--commands` are PULL-tier reference. The governing relation.
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

/// The 8-family taxonomy. The ONLY hand-maintained classification; the
/// drift-guard test asserts it partitions the visible clap subcommands exactly.
static FAMILIES: &[Family] = &[
    Family { key: "change",     members: &["slice","revision","rfc","rec","review","reconcile","coverage"] },
    Family { key: "governance", members: &["adr","policy","standard","spec"] },
    Family { key: "knowledge",  members: &["memory","knowledge","backlog"] },
    Family { key: "relations",  members: &["link","unlink","needs","after","supersede"] },
    Family { key: "facets",     members: &["estimate","value","risk","tag"] },
    Family { key: "reports",    members: &["status","next","blockers","survey","explain"] },
    Family { key: "explore",    members: &["search","inspect","relation","concept-map","map"] },
    Family { key: "infra",      members: &["install","boot","serve","config","validate","reseat",
                                           "export","reservation","worktree","dispatch","catalog"] },
];

/// Verbs every entity kind shares; subtracted to leave the distinctive set.
/// `status` is deliberately NOT in the spine — not universal, lifecycle-bearing,
/// so it surfaces as distinctive where present.
const SPINE: &[&str] = &["new", "list", "show", "paths"];

/// Infra is operational / skill-driven, not boot-time authoring routing — its
/// commands appear in the family header only, never verb-expanded (D7).
const SUPPRESS_VERBS: &[&str] = &["infra"];

pub(crate) fn render_top_level_help(color: bool, term_width: Option<u16>) -> String; // rewritten
pub(crate) fn render_boot_map() -> String;                                            // new, plain text
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

- **Owned, hand-maintained**: `FAMILIES`, `SPINE`, `SUPPRESS_VERBS` (cli.rs).
  Everything else is derived from the clap tree at render time.
- **Derived at render**: distinctive verbs per command (`verbs − SPINE`),
  whether a command is a leaf, command descriptions (`about` / first-sentence).
- No new persisted/runtime state. The boot-map body is regenerated each
  `doctrine boot`; it is derived content living in the gitignored snapshot.

### 5.4 Lifecycle, Operations & Dynamics

**Human `--help` (per-family sub-tables, D4a + D8 shared widths):**
```
change
  slice       Create and list slices — the unit of intentional change
  revision    Create, show, and transition revisions
  …
governance
  adr         Create and list architecture decision records
  …
```
- D8: all 8 sub-tables share **global column widths** (max command-name width
  across every family) so vertical dividers align across sections. Independent
  comfy-tables would autosize per-section → ragged. Mechanism (confirmed at
  execute): compute global widths once, render family bodies at fixed shared
  widths with plain family-heading lines interleaved — either a single table
  with header-rows or a shared-width parameter threaded into `listing`.

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
- Sub-line per command **iff** it has distinctive verbs AND its family ∉
  `SUPPRESS_VERBS`. Leaves and infra never sub-line (already named in header).
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

- **OQ-1 (D8 mechanism) — RESOLVED: plain-text grouped help (option a).**
  `listing::render_columns` (comfy-table) autosizes each table independently and
  has no section-header row, so 8 separate calls cannot share divider columns.
  Decision: render the grouped help as **plain text** with manually-computed
  shared padding (like `render_boot_map`) — clean alignment, full control;
  accepts dropping the alternating-row color paint and term_width wrapping the
  flat help has today (the grouped help's value is structure+alignment, not
  color; descriptions stay one short line). Exact padding mechanics confirmed at
  execute (phase 1).
- *(Resolved: D1 static table, D2 auto-derive, D3 `--boot-map` flag + CommandMap
  section, D4 sub-tables, D7 suppress infra verbs, D8 shared widths, OQ-1
  plain-text, OQ-3 `--boot-map` name.)*

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
- **D4 — per-family sub-tables** for human `--help` (vs single family-column
  table). Families are navigational headers; sub-tables scan fastest. Human
  help is not token-budgeted.
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
  `force_no_tty`, byte-exact; asserts family order, sub-table grouping, shared
  column alignment (D8).
- **Golden: `--help --boot-map`** — byte-exact boot-map text; asserts spine line,
  header+sub-line rule, infra suppression (D7), leaf handling.
- **Boot byte-stability** — `doctrine boot` twice ⇒ identical; `boot --check`
  clean with the CommandMap section present (extends IMP-123-style assertions).
- **Behaviour preservation** — existing `--commands` golden/`render_commands_table`
  unchanged; existing boot suites green.

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
