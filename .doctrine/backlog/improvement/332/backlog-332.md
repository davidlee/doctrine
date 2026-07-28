# IMP-332: CLI record verb accepts typed facet fields and a stdin/file request

Owner for RV-318 F-1, the SL-231 audit's blocker. **Decision: deliver, not
narrow** — `design.md` §3.1 stands as written and needs no edit; this item is
what makes it true.

## What is missing

`design.md` §3.1:169-175 lists what `doctrine observation record friction` "also
accepts". Two of the five never shipped:

- **repeatable typed facet fields** — `FrictionRecordArgs`
  (`src/commands/observation.rs:50-69`) carries only `summary`, `--detail`,
  `--uid`, `--no-enrich`, `--path`.
- **a complete friction request from standard input or a file** — no stdin or
  `--input` path exists anywhere in the observation adapter (confirmed with a
  positive control).

Consequence today: `run_record` calls `merge_explicit_facets(auto_facets, None)`
with a hard-coded `None`, so design §3.1:197 "explicit caller values win" is
structurally unreachable from the CLI. The MCP adapter already passes explicit
facets through (`src/mcp_server/tools.rs:1283`), so the two adapters are not at
parity and only MCP satisfies PHASE-03 EX-3 end to end.

## What is already decided, and what is not

**Decided — do not re-litigate.** §3.3 pins the request shape
(`uid? summary detail? facets? enrich?`), and `merge_explicit_facets` is the
shared merge policy in the `observation` leaf, reachable from both adapters by
construction (ADR-001 severs the `mcp_server → commands` back edge, which is why
the policy lives below both). The service seam, receipt shape, validation, and
idempotency are all unchanged — this is an adapter-input change only.

**Open — needs a sketch and acceptance before code.**

1. **Facet flag grammar.** Five facet groups with heterogeneous field types
   (`Option<String>`, numeric counters, `Option<Vec<String>>`). Dotted repeatable
   key=value (`--facet execution.interface=pi`) is the obvious candidate; the
   typing and error surface for non-string fields is the real question.
2. **Whether stdin/file takes the §3.3 request object verbatim.** Strongly
   implied and probably right — one shape for both adapters — but §3.1 only says
   "a complete friction request", so it is inference, not text.
3. **Precedence when both are supplied** (a stdin request plus flags), and how
   either interacts with `--no-enrich`.
4. Note `-` as a stdin sentinel already has a meaning elsewhere in this codebase
   and is explicitly refused over MCP (`reject_stdin_sentinel`,
   `MCP_BODY_STDIN_SENTINEL`, SL-230 D-P5-1). Follow that precedent rather than
   inventing a second convention.

## Also in scope: the criterion that let this through

SL-231 PHASE-03 VT-1's `expects` names "stdin/file input" and
"explicit-over-automatic enrichment"; its four keywords cover neither, so the
criterion reported PASS while both clauses were undelivered. That is the RV-317
F-4 class recurring. When this work lands, **append** keywords to that criterion
so it covers its own text — ids are immutable, append never renumber. Do not
leave the plan asserting a contract nothing checks.

## Route

Per project governance, the small-change route (design sketch → acceptance →
implement → close) rather than a full slice: the request shape is already fixed
by §3.3 and only the CLI flag surface is genuinely open. Promote to a slice if
the facet grammar turns out to carry more than it looks.

Sequenced **after SL-231 closes**, executed **inline** (not dispatched) — the
delta is one adapter file plus tests, and a funnel cycle would cost more than it
buys.

Source: RV-318 F-1 (`.doctrine/review/318/review-318.md`, Synthesis and
Reconciliation Brief).

## Accepted sketch (2026-07-28)

Settled with the user on the small-change route; implementation follows this.
The four open questions above are answered here — do not re-open them.

### 1. `--facet <group>.<field>=<value>`, repeatable — string values only

Dotted keys assemble into a JSON object, then `default_facet_schema_versions` →
`serde_json::from_value::<Facets>`. **No field table and no type table**: all six
facet types carry `#[serde(deny_unknown_fields)]`, so unknown groups and unknown
fields are refused by serde against the real struct definitions. Key validation
is free and cannot drift.

Answers open question 1. The field census is what settles the typing half: of 30
caller-settable fields exactly **four** are not `Option<String>` —
`usage.total_tokens`, `usage.prompt_tokens`, `usage.completion_tokens`, and
`correlation.related_observations`. Those four are precisely the fields a harness
emits programmatically; the 26 an agent types by hand are all strings. So typed
fields are reached through `--input`, and a `--facet` attempt at one earns an
accurate serde type error naming the field and the expected type.

§3.1's "repeatable **typed** facet fields" is read as *addressed against the
typed schema and validated* — as opposed to free-form tags — not "assorted scalar
types from the flag". Recorded because it is a judgement call, not text. The
other reading is a purely additive upgrade later (JSON-parse the RHS when it
parses as a non-string scalar); nothing here forecloses it.

### 2. `--input <PATH>`, `-` means stdin

Carries the §3.3 request **verbatim** — `{uid?, summary, detail?, facets?,
enrich?}` — so both adapters take one shape. Answers open question 2 as *yes*.
The `-` sentinel follows the established precedent (`resolve_body`,
`src/memory.rs`), already refused MCP-side as `MCP_BODY_STDIN_SENTINEL`
(SL-230 D-P5-1) — open question 4.

### 3. `--input` is exclusive

Refused alongside the positional summary, `--detail`, `--uid`, `--no-enrich` and
`--facet`; `summary` becomes `required_unless_present("input")`. Answers open
question 3. Rationale: two sources of truth for one field re-opens "explicit
caller values win" (§3.1:197) at a layer that has no origin field to record the
answer in. `--path` stays compatible — it says where the repository is, not what
the record says.

### 4. `input` joins `CAPTURE_REFUSED_KEYS` (MCP side)

Beyond the item as filed, but it follows: that denylist already refuses `path`
and `root` because the confined capture surface must not let a caller name a
filesystem path, and `input` is exactly such a key. One const entry plus a case
in the existing refusal test loop keeps the two adapters' contracts coherent.

### 5. `parse_explicit_facets` + `default_facet_schema_versions` move to the leaf

Both are stranded in `src/mcp_server/tools.rs`; both adapters now need them and
ADR-001 severs the `mcp_server → commands` back edge, so they belong in
`src/observation/wire.rs` beside `merge_explicit_facets` for the same reason that
policy does. The existing MCP tests are the behaviour-preservation gate.

### Not a concern, verified

`store::create` validates via `wire::validate`, so the CLI's hand-built envelope
is not skipping validation — there is no second parity gap here. And RV-318 F-2's
`adopt` derives origin and ignores any caller-supplied `_origin`, so exposing
facets on the CLI opens no forgery surface. Pinned by a test at the new entry
point rather than assumed.
