# SL-210 design — comparison ledger capture

RFC-019 Phase A. Pure capture: typed, lossless, append-only pairwise
judgements in merge-clean session files. No scoring change; nothing consumes
the ledger until Phase B.

## Current vs target behaviour

**Current.** Value enters the corpus only as an authored point magnitude
(`doctrine value set`), the weakest input the priority engine consumes
(RFC-019). No comparative evidence surface exists. `value set` on a
non-value-bearing kind silently writes a scoring-inert facet.

**Target.** A `compare` CLI group captures pairwise judgements into
append-only session files under `.doctrine/comparisons/`; `compare list`
reads the evidence back; `compare withdraw` appends tombstones. `value set`
on a non-value-bearing kind warns (REV-022 Q1). Scoring, `survey`, `next`,
`explain` are bit-identical to today.

## Adjudicated decisions

| # | Decision | Choice |
|---|---|---|
| D1 | Verb naming (OQ-A1) | Top-level `compare` group, domain-neutral; capture is the bare default action, `list`/`withdraw` subcommands |
| D2 | Implicit sessions (OQ-A2) | Session-of-one per invocation; no per-day file, no lock, ever — Phase C's elicit loop owns real multi-row sessions in-process |
| D3 | Tombstones (OQ-A3) | `compare withdraw` ships in Phase A; resolution semantics stay Phase B |
| D4 | Id form | `integrity::parse_canonical_ref` — full `SL-123` form only (cross-kind pairs make bare ids ambiguous; IMP-227) |
| D5 | Rater columns | `rater` kind enum (`human\|agent`, required, default `agent`) + optional `by` free-text identity — one string would smear enum and identity |
| D6 | Value-domain admissibility | `VALUE_BEARING` minus RSK (RFC-019 domain table; REV-022 Q2 keeps RSK's *scoring* participation — A4 gates comparisons only) |
| D7 | Frame vocab at ship | `equal-effort` (default) + `prefer-first`; closed set, named constants (STD-001) |
| D8 | Ratio form | Schema carries `form = "order"\|"ratio"`; the verb exposes order only — RFC-019 OQ-6 stays open, capture stays lossless |
| D9 | Warn scope | REV-022 Q1 warn lands in this slice (`commands/facet.rs`) — same governance driver, trivial delta |

## Session-file schema

Path: `.doctrine/comparisons/<date>-<session-uid>.toml` — authored tier
(committed, diffable). Filename uid = session uid (uuid v7, shell-minted).

```toml
schema  = "doctrine.comparison-session"
version = 1

[session]
uid      = "0197f3a2-5b1e-7c3d-9e4f-1a2b3c4d5e6f"
date     = "2026-07-10"
audience = "stakeholder"      # optional — OQ-1/T4 per-audience surfacing rides this

[[judgement]]
uid       = "0197f3a2-6c2f-7d4e-8f5a-2b3c4d5e6f7a"
seq       = 0                 # row_seq within file; ordering key (date, session_uid, seq)
a         = "SL-204"
b         = "IMP-118"
preferred = "SL-204"          # ∈ {a, b}
domain    = "value"
frame     = "equal-effort"    # closed vocab per domain
form      = "order"
lens      = "user-value"      # optional — IDE-035 seam
rater     = "agent"           # human | agent, required
by        = "david"           # optional identity
note      = "auth unblocks the pilot"   # optional
date      = "2026-07-10"

[[tombstone]]
uid    = "0197f3a4-…"
seq    = 1
target = "0197f3a2-6c2f-…"    # a judgement row uid (F-4: uid-referencing, file-order-independent)
date   = "2026-07-10"
note   = "wrong way round"    # optional
```

Invariants:

- **Append-only**: capture never rewrites an existing file; ad-hoc capture
  mints a fresh session file per invocation (D2). Withdraw appends; nothing
  edits or deletes rows.
- **Lossless**: every captured field round-trips verbatim through
  parse → serialize; unknown frames/domains in *future* files parse and are
  preserved (forward compatibility), but *this* verb only writes the closed
  vocab.
- **Total ordering**: `(date, session_uid, seq)` per RV-260 F-4; `seq` is
  dense per file starting at 0 across both row types.

## Module boundaries (ADR-001: leaf ← engine ← command)

### `src/comparison.rs` — new, pure engine tier

No clock, disk, rng, or git. Date and uids are inputs (the date/uid pattern).

The serde model is the wire model — it must emit the documented schema
exactly (RV-262 F-1): nested `[session]` table, singular array-of-table
names, lowercase enum tokens.

```rust
pub(crate) const COMPARISON_SCHEMA: &str = "doctrine.comparison-session";
pub(crate) const COMPARISONS_DIR: &str = "comparisons";   // under .doctrine/
pub(crate) const DOMAIN_VALUE: &str = "value";
pub(crate) const FRAME_EQUAL_EFFORT: &str = "equal-effort";
pub(crate) const FRAME_PREFER_FIRST: &str = "prefer-first";
pub(crate) const VALUE_FRAMES: &[&str] = &[FRAME_EQUAL_EFFORT, FRAME_PREFER_FIRST];

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RaterKind { Human, Agent }      // wire: "human" / "agent"

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RowForm { Order, Ratio }        // wire: "order" / "ratio"

#[derive(Serialize, Deserialize)]
pub(crate) struct SessionHeader {
  pub uid: String,
  pub date: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub audience: Option<String>,                 // OQ-1/T4 contract field
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Judgement {
  pub uid: String, pub seq: u32,
  pub a: String, pub b: String, pub preferred: String,
  pub domain: String, pub frame: String, pub form: RowForm,
  pub lens: Option<String>,
  pub rater: RaterKind, pub by: Option<String>,
  pub note: Option<String>, pub date: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Tombstone {
  pub uid: String, pub seq: u32, pub target: String,
  pub date: String, pub note: Option<String>,
}

/// The file model — serializes 1:1 to the documented schema.
#[derive(Serialize, Deserialize)]
pub(crate) struct ComparisonSession {
  pub schema: String,          // COMPARISON_SCHEMA, checked on parse
  pub version: u32,
  pub session: SessionHeader,
  #[serde(default, rename = "judgement")]
  pub judgements: Vec<Judgement>,
  #[serde(default, rename = "tombstone")]
  pub tombstones: Vec<Tombstone>,
}

pub(crate) fn parse(text: &str) -> anyhow::Result<ComparisonSession>;
pub(crate) fn to_toml(s: &ComparisonSession) -> String;   // mirrors plan/ledger house pattern;
                                                          // golden test pins byte shape

/// Value-domain admissibility over already-resolved kinds. Pure; the kind
/// lookup happens in the shell. Err carries the human-readable refusal.
pub(crate) fn admissible_value_pair(kind_a: &str, kind_b: &str) -> Result<(), String>;

/// Structural row validation: preferred ∈ {a,b}, a ≠ b, closed domain/frame
/// vocab, non-empty refs. Admissibility is separate (needs kinds).
pub(crate) fn validate_judgement(j: &Judgement) -> anyhow::Result<()>;
```

`admissible_value_pair` derives its admit set from `kinds::VALUE_BEARING`
minus `kinds::RSK` — expressed against the existing constants, not a parallel
list, with a unit test pinning the relationship (D6; no magic strings).

### `src/commands/compare.rs` — new, command shell

- Clap: `compare` group with bare capture args +
  `args_conflicts_with_subcommands = true`; subcommands `list`, `withdraw`.
  (Fallback if ergonomics fight clap: promote capture to `compare record` —
  cosmetic, not structural.)
- Full capture surface (RV-262 F-3/F-4 — every schema column the row needs
  is settable at capture; nothing reachable only by default):

  ```text
  doctrine compare <A> <B> --prefer <A|B|a|b>
      [--frame equal-effort|prefer-first]   # default equal-effort
      [--rater human|agent]                 # default agent
      [--by <NAME>] [--lens <L>] [--note <N>]
      [--audience <AUD>]                    # session-header field
  ```

  `domain` is fixed at `value` and `form` at `order` in Phase A (D8; no
  flag mints them).
- Capture flow: parse both refs (`parse_canonical_ref`) → resolve kinds via
  the corpus scan (refs must exist — dangling evidence refused) →
  `admissible_value_pair` → `validate_judgement` → mint session uid + row uid
  (uuid v7) + `clock::today()` → write
  `.doctrine/comparisons/<date>-<uid>.toml` (fresh file, `fsutil` create-new
  semantics — never truncate an existing path).
- `list [<ID>]`: scan `.doctrine/comparisons/*.toml`, parse all sessions,
  flatten rows, sort by `(date, session_uid, seq)`, optional participation
  filter (`a == ID || b == ID`), render: **full row uid**, pair with
  preferred marked, frame, rater(+by), date, note; rows targeted by any
  tombstone render struck with `withdrawn` — display-only interpretation
  (resolution is Phase B). Full uid, never a prefix: uuid-v7 prefixes share
  a timestamp bucket and collide, and this listing is the lookup surface
  that feeds `withdraw` (RV-262 F-6; house precedent in the memory
  subsystem).
- `withdraw <row-uid> [--note]`: scan to locate the target row uid (must
  exist, must be a judgement, must not already be tombstoned — double
  withdraw refuses with "already withdrawn"); append-only: writes a *new*
  session-of-one file containing only the tombstone (D2 uniformity — no
  in-place file edits, merge-clean, same code path as capture).
- `--prefer` accepts the full ref of one side, or the literals `a`/`b`
  (no lexical collision with canonical refs); refused otherwise.
- Terminal-status participants are **admitted**: comparing against shipped
  work is legitimate anchoring; row-effect semantics under entity lifecycle
  are Phase B's T6 event-effect table, not a capture gate.

### `src/commands/facet.rs` — REV-022 Q1 warn

`run_value_set`: after canonicalising, if the target kind ∉
`kinds::VALUE_BEARING`, print a warning (write still proceeds):
`warning: value on <ID> is scoring-inert — <kind> is not value-bearing;
scoring ignores this facet (ADR-015 § Value-source resolution)`.

### Wiring

`main.rs` + `commands/mod.rs`: `mod compare`, CLI enum arm, dispatch. No
config additions (no knobs in Phase A). `doctrine boot` regenerates the SPINE
line after landing.

## Data flow

```
capture:  argv ─▶ parse refs ─▶ corpus kinds ─▶ admissibility ─▶ validate
               ─▶ mint (uid, date)  [impure edge]
               ─▶ ComparisonSession { 1 row } ─▶ to_toml ─▶ create-new file
list:     dir scan ─▶ parse* ─▶ flatten ─▶ sort (date, session_uid, seq)
               ─▶ tombstone mark ─▶ render
withdraw: dir scan ─▶ find target uid ─▶ mint ─▶ session { 1 tombstone } ─▶ file
```

The pure core never sees a path or a clock; the shell never interprets rows.

## Verification alignment

Unit (`src/comparison.rs`):

- `round_trip_preserves_all_fields` — full row incl. optionals, golden TOML
- `golden_shape_matches_documented_schema` — byte-level: nested `[session]`,
  singular `[[judgement]]`/`[[tombstone]]`, lowercase enum tokens (F-1)
- `parse_preserves_unknown_frame_rows` — losslessness / forward compat
- `validate_rejects_preferred_outside_pair`, `_self_pair`, `_unknown_frame`,
  `_unknown_domain`
- `admissible_value_pair_admits_cross_kind_work` (SL×IMP), `_refuses_record`
  (QUE), `_refuses_rsk`, `admit_set_is_value_bearing_minus_rsk` (pins D6 to
  `kinds::` constants)

Integration (CLI):

- `compare_capture_writes_session_of_one` — file exists, shape golden
- `second_capture_never_touches_first_file` — append-only invariant
- `compare_refuses_missing_ref`, `_refuses_record_pair`, `_refuses_rsk`
- `compare_list_orders_by_total_key_and_filters_by_participant`
- `withdraw_appends_tombstone_and_list_marks_withdrawn`
- `withdraw_refuses_unknown_row_uid`, `_refuses_double_withdraw`
- `compare_admits_terminal_status_participant`
- `value_set_warns_on_non_value_bearing_kind` (write proceeds)

Behaviour-preservation gate: priority/`survey`/`next`/`explain` suites pass
unchanged — nothing consumes the ledger.

## Code impact summary

| Path | Change |
|---|---|
| `src/comparison.rs` | new — pure model, vocab constants, validation, admissibility |
| `src/commands/compare.rs` | new — capture / list / withdraw shell |
| `src/commands/mod.rs`, `src/main.rs` | wiring |
| `src/commands/facet.rs` | Q1 warn in `run_value_set` |
| `src/kinds.rs` | none expected (admit set derived in `comparison.rs`); touched only if constants need visibility widening |
| `.doctrine/comparisons/` | new authored directory (created lazily on first capture) |

## Out of scope (fenced)

Constraint propagation / bounds / projection / contradiction surfacing /
supersession *resolution* (Phase B); elicitation queue (C); estimate & risk
domains (capture schema admits the typing only); ratio-form capture (OQ-6);
JSON emit for `list` (arrives with the queue's agent-curator surface);
web/session surfaces (D+).

## Open questions

None blocking. OQ-6 (ratio admission) deliberately remains RFC-019's open
question; D8 keeps the schema ready without exposing it.
