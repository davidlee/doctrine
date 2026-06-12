# Design SL-051: retire `backlog order`; fold ordering into `list` as a default-on comparator

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-051, IMP-028, REQ-097, ADR-001); doc-local refs bare — OQ-1, D1, DD-1, R1. -->

## 1. Design Problem

IMP-028. `backlog order` and `backlog list` are divergent surfaces over one spine.
`list` carries the full grammar (`--kind/--filter/--regexp/--status/--tag/--all/
--format/--json/--columns` — the `listing.rs` shared column model); `order` carries
only `--path` and, to show *ordered* output, abandons every filter/format/column the
user already knows. The card rejects teaching `order` the list grammar (a second
verb for "same rows, different order" is duplication) in favour of the merge:
**retire `order`, fold ordering into `list`.**

Preflight refined this to **default-on** (locked by the user, do not relitigate):
composed `after`/`needs` order is the *default* row sequence for `list`, with an
opt-out to restore id-sort. Default-on forces a clean decomposition — ordering
becomes a **pure comparator** over the rows `retain` already kept; membership stays
the filter's job; `order`'s fail/hide behaviour is **not** inherited wholesale.

This slice introduces **no new graph mechanism**: the `cordage` `BacklogOrder`
adapter (`src/backlog_order.rs`) and its `project`/`render_overrides` shell helpers
are reused unchanged — only their *call site* moves from a standalone verb into the
`list` path.

## 2. Current State

- **`src/backlog.rs:900` `list_rows`** — the survey spine: `validate_statuses` →
  `args.columns.take()` → `listing::build` → `listing::retain(read_all)` (membership:
  substr/regex/status/tag + terminal hide-set) → `--kind` filter → `items.sort_by_key(|i|
  (i.kind.ordinal(), i.id))` → `select_columns`/`render_columns` | `json_envelope`.
  Returns `anyhow::Result<String>` (stdout only).
- **`src/backlog.rs:1560` `order_rows`** — the parallel surface: own `read_all` →
  `project(&items)` (the **non-terminal node set** + `AbsentDrop`s) → `BacklogOrder::build`
  → **hard-bail** on `dep_cycles().first()` → `corpus` map → `order.ordered()` rows
  (cordage order, default cols) + `render_overrides` footer. `run_order` (1595) is the
  thin shell; dispatch `src/main.rs:2222`; `Order` clap variant `src/main.rs:887`.
- **`src/backlog_order.rs`** — the `BacklogOrder` adapter. `build` (178) **succeeds
  even on a `needs` cycle** (cycle edges rejected on the `Reject` overlay; the graph
  still builds); `dep_cycles()` (288) reports them post-build; `ordered()` (278) is a
  **total** order over the node set; `overrides()` (303) the dropped soft edges. Pure.
- **`src/listing.rs:123` `ListArgs`** — the clap-free arg mirror, **shared** by
  backlog / memory / spec / adr / … list surfaces. `build` (150) and `retain` (188);
  `retain` is FILTER-ONLY and preserves input order (ordering is the caller's job, §5.3).
- **`src/main.rs:67` `CommonListArgs`** — the flattened clap struct flowing into
  every `*::run_list`; `into_list_args()` (111) lowers it to `ListArgs`.
- **`tests/e2e_backlog_order_golden.rs`** (373 lines) — the `backlog order` /
  `backlog needs` black-box goldens, incl. the dep-cycle **hard-error** at line 167.

## 3. Forces & Constraints

- **Non-goals (scope):** no `listing.rs` redesign (`list` *adopts* a comparator slot);
  no data-model / relation-schema change; no RSK-005 fix (adapter bimap corruption,
  adjacent but separate); no PRD-009 amendment; no deprecation cadence (clean cut).
- **ADR-001 layering** (leaf ← engine ← command): the comparator is backlog-domain
  (needs `BacklogOrder`/`project`/`ItemId`) → lives in `backlog.rs`, never in the
  shared `listing.rs` leaf. The ordering opt-out is backlog-only → rides the backlog
  `List` clap variant, **not** the shared `CommonListArgs`/`ListArgs`.
- **Pure/imperative split:** the compute half (`list_rows`/`compose`) returns
  strings; only the thin shell (`run_list`) touches stdout/stderr. The cycle warning
  is *returned*, not `eprintln!`'d from the compute half.
- **A-2 (membership invariant):** today's `list` membership = the `retain` filter.
  Ordering must preserve **exactly** that set; `order`'s non-terminal projection is
  **not** re-imposed as a membership filter — it survives only to build the graph and
  feed the diagnostic.

## 4. Target Design

### 4.1 Data flow — compose-then-filter (OQ-1, DD-1)

```
let corpus = read_all(root)?;                          // read ONCE
let ordering = match by {
    OrderBy::Sequence => Some(compose(&corpus)?),       // borrows corpus
    OrderBy::Id       => None,                           // skip the graph entirely
};
let mut items = listing::retain(corpus, &filter, is_hidden, key);  // moves corpus; membership UNCHANGED
items.retain(|i| kind.is_none_or(|k| i.kind == k));
match &ordering {
    Some(o) if !o.degraded => items.sort_by_key(|i| {    // composed position, then classic tiebreak
        (o.pos.get(&ItemId::new(i.kind, i.id)).copied().unwrap_or(usize::MAX),
         i.kind.ordinal(), i.id)
    }),
    _ => items.sort_by_key(|i| (i.kind.ordinal(), i.id)),  // --by id OR cycle-degrade
}
```

The graph is built over the **full non-terminal corpus** (`project` filters
`!is_terminal` internally) → a global sequence. `retain`'s set is then ordered by
**global position**. `compose(&corpus)` borrows first, then `retain` moves `corpus`
(the `Ordering` owns its own data, no live borrow). Off-sequence rows — terminal
items shown via `--all`/`--status`, or rows the graph never placed — sort last via
the `usize::MAX` sentinel, then by the classic `(kind.ordinal, id)` tiebreak.

**Why compose-then-filter** (not filter-then-compose): the locked decomposition
fixes membership = `retain`'s set and ordering = a pure comparator. Building the
graph over the full corpus keeps the global order stable regardless of which rows
the filter later drops; filter-first would change which edges exist (edges to
filtered nodes would dangle), perturbing the order of the survivors — incompatible
with "membership unchanged."

### 4.2 New shape in `backlog.rs`

```rust
#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub(crate) enum OrderBy {
    #[default]
    Sequence,   // composed after/needs order (default)
    Id,         // classic (kind.ordinal, id) sort
}

/// The composed ordering over the live corpus + its honest-record diagnostic.
struct Ordering {
    pos: BTreeMap<ItemId, usize>,  // composed positions; chain members only (empty when degraded)
    footer: String,                // render_overrides output ("" when clean)
    degraded: bool,                // a needs cycle forced the id-sort fallback
    warning: Option<String>,       // the cycle advisory, for stderr
}

fn compose(corpus: &[BacklogItem]) -> anyhow::Result<Ordering> {
    let (inputs, absent) = project(corpus);
    let order = BacklogOrder::build(&inputs)?;   // err = internal adapter bug only (A2); propagate
    let cmap: BTreeMap<ItemId, &BacklogItem> =
        corpus.iter().map(|i| (ItemId::new(i.kind, i.id), i)).collect();
    let footer = render_overrides(&cmap, &absent, &order.overrides());
    if let Some(cycle) = order.dep_cycles().first() {
        return Ok(Ordering {
            pos: BTreeMap::new(),
            footer,
            degraded: true,
            warning: Some(format!(
                "backlog list: `needs` dependency cycle — {} — ordering by id (resolve, then re-run)",
                name_cycle(cycle)
            )),
        });
    }
    let pos = order.ordered().iter().enumerate().map(|(i, id)| (*id, i)).collect();
    Ok(Ordering { pos, footer, degraded: false, warning: None })
}
```

`project`, `render_overrides`, `classify_dangling`, `name_cycle`, `AbsentDrop`,
`exposure` all **survive** — re-homed under `compose`. `order_rows` and `run_order`
are **deleted**.

### 4.3 Output assembly per format (OQ-2, diagnostic decision)

`list_rows` return type changes to `anyhow::Result<(String, String)>` —
`(stdout, stderr)`:

| format | stdout | stderr |
|---|---|---|
| **Table** | composed rows + `footer` (conditional — `render_overrides` returns `""` when clean) | `warning` (cycle only) |
| **Json** | `json_envelope("backlog", rows)` — rows in **composed sequence**; envelope **unchanged** | `warning` + `footer` (advisory) |

JSON array order **is** the composed sequence (one surface, one order). The
`overrides` honest-record stays out of the JSON envelope — no `listing.rs`
`json_envelope` change, honouring the non-goal. In JSON mode the human-facing
footer routes to stderr as an advisory; in table mode it is the stdout footer
(matching the `order` precedent and the scope's "conditional footer" wording).

`run_list` (the shell) writes the stdout string to stdout and, when non-empty, the
stderr string to stderr:

```rust
pub(crate) fn run_list(path, kind, by: OrderBy, args: ListArgs) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (out, err) = list_rows(&root, kind, by, args)?;
    write!(io::stdout(), "{out}")?;
    if !err.is_empty() { write!(io::stderr(), "{err}")?; }
    Ok(())
}
```

### 4.4 Cycle-degrade (OQ-3, DD-3)

`build` succeeds on a `needs` cycle; `dep_cycles()` reports it. On a cycle, `compose`
returns `degraded: true` (empty `pos`) → `list_rows` falls back to the classic
`(kind.ordinal, id)` sort, emits the `warning` to stderr, and **exits 0**. Every
retained row is still present and id-ordered — never an empty table, never non-zero.
This replaces `order`'s old hard-error (the everyday survey verb must stay total on
a cyclic graph). The `--by id` opt-out skips `compose` entirely, so it never warns.

### 4.5 `main.rs`

- **Delete** the `Order` clap variant (887–894), its dispatch arm (2222), and
  `backlog::run_order`.
- The backlog `List` variant gains one field:
  `#[arg(long = "by", value_enum, default_value_t = backlog::OrderBy::Sequence)] by: backlog::OrderBy`
  (clap renders the values `sequence` | `id`).
- Dispatch (2204): `backlog::run_list(path, kind, by, list.into_list_args())` — 4
  positional args, under the clippy `too_many_arguments` ceiling.

## 5. Code Impact Summary

| path | change |
|---|---|
| `src/backlog.rs` | add `OrderBy` + `Ordering` + `compose`; rewrite `list_rows` (compose-then-filter, comparator, `(String, String)` return); rewrite `run_list` (stderr arm, `by` param); **delete** `order_rows` + `run_order`; re-home `project`/`render_overrides`/`name_cycle`/`AbsentDrop` (no logic change). |
| `src/main.rs` | **delete** `Order` variant + dispatch + help; add `--by` to `List`; thread `by` into `run_list`. |
| `src/backlog_order.rs` | **unchanged** (reused as-is). |
| `src/listing.rs` | **unchanged** (read-only — comparator and `OrderBy` are backlog-local). |
| `tests/e2e_backlog_order_golden.rs` | migrate onto `list`; replace the dep-cycle hard-error golden with the degrade golden; add default-on / `--by id` / filtered-compose / footer goldens. Rename → `e2e_backlog_list_order_golden.rs`. |

## 6. Verification Alignment

New / changed evidence:

- **VT — default-on order.** `backlog list` (no flag) emits the composed sequence
  (golden) over a fixture with `after`/`needs` edges.
- **VT — opt-out.** `backlog list --by id` restores the `(kind.ordinal, id)` sort
  (golden).
- **VT — membership invariant (A-2 proof).** The row *set* of `list` == the row set
  of `list --by id` over the same fixture — only order differs.
- **VT — filtered compose.** `backlog list --status open --kind improvement` orders
  the retained subset by global position (survivors keep relative order).
- **VT — footer conditionality.** The `overrides:` footer is absent on a clean
  survey, present (stdout, table) when an edge is dropped.
- **VT — cycle degrade.** A `needs` cycle ⇒ id-sorted table on stdout + warning on
  stderr + **exit 0**; never empty (replaces the line-167 hard-error golden).
- **VT — verb retired.** `backlog order` is an unknown-subcommand clap error.
- **VT — JSON sequence.** `backlog list --json` array order = composed sequence;
  envelope shape unchanged.
- `just check` green; `cargo clippy` (plain) zero warnings.

## 7. Design Decisions

- **DD-1 (OQ-1) — compose-then-filter.** Graph over the full non-terminal corpus →
  global sequence → membership = `retain`'s rows ordered by position. Forced by the
  locked decomposition; filter-first would dangle edges and perturb survivor order.
- **DD-2 (seam) — comparator in `backlog.rs`, not `listing.rs`.** It needs
  backlog-domain types (`BacklogOrder`/`project`/`ItemId`); it replaces the existing
  `sort_by_key` line in `list_rows`. `listing.rs` stays read-only (ADR-001 + non-goal).
- **DD-3 (OQ-3) — cycle-degrade: id-sort + stderr warn + exit 0.** The survey verb
  must not fail or empty on a cyclic graph.
- **DD-4 (OQ-2) — JSON emits the composed sequence; diagnostic out of the envelope.**
  Array order carries the order; `overrides`/cycle advisory route to stderr. No
  `json_envelope` change.
- **DD-5 (flag) — `--by id`** (values `sequence` | `id`, default `sequence`). Names
  the sort key, reads precisely, extensible (future `--by created`/`rank`); avoids
  the semantically-wrong `--no-order` ("not no order — id order").

## 8. Risks & Open Questions

- **A-1 — graph cost on every default `list`.** `compose` builds the `cordage` graph
  on each default invocation. Acceptable at backlog scale (dozens–hundreds); `--by id`
  is the zero-cost escape. Note, do not optimise.
- **R-1 — surface break (clean cut).** The only **live** `backlog order` consumers are
  the verb itself (`main.rs`/`backlog.rs`) and `tests/e2e_backlog_order_golden.rs`;
  all other hits are historical slice/spec/ADR prose (do not touch). PRD-009 /
  `REQ-097` bind the ordering *capability*, not the verb name — satisfied by `list`;
  a reconcile note at close, not a spec change.
- **No open questions remain** — OQ-1/2/3 + the two seam calls are settled (§7).

## 9. Follow-Ups

- RSK-005 (`backlog_order` adapter duplicate-`ItemId` bimap corruption) stays open;
  flag at close if the fold makes it cheaper to address (same `project` call site).
- IMP-033 (cross-kind `needs`/`after` sequencing) unaffected by this merge.
