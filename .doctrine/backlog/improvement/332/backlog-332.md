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
