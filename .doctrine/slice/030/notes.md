# Notes SL-030: Policy entity kind (POL)

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — adr CLI golden net (commit `8607e12`)

Done: `tests/e2e_adr_cli_golden.rs`, 10 black-box goldens over the built binary
pinning `adr show`/`status`/`list` byte-exact (stdout + JSON + error text). This
is the behaviour-preservation gate PHASE-02 holds green UNCHANGED. `src/`
untouched (EX-3). `just check` green. Plan was amended first (commit `80f03fa`)
to add the `adr list` golden — see below.

**The PHASE-02 contract these goldens lock (read before extracting):**
- `parse_ref` strips `ADR-` OR `adr-` — exactly two literal cases, NOT
  case-insensitive (the doc-comment at `src/adr.rs:307` lies). `adr_show_garbage_
  ref` + the migrated descriptor must keep the two-case strip; a `to_lowercase`
  "fix" reddens the gate (Codex MAJOR-3, now executable).
- `show --json` is **pretty, BTreeMap key order** (serde_json, no `preserve_order`
  → keys alphabetical), **no trailing newline** (`write!` not `writeln!`). The
  dynamic stem key (`"adr"`) is what PHASE-02's hand-built `serde_json::Map` must
  reproduce — the `json!` macro can't take a runtime key (design R2).
- error stderr shape = anyhow `Debug`: `Error: <ctx>\n\nCaused by:\n    <source>\n`
  for sourced errors; a bare `bail!` (malformed-refuse) prints `Error: <msg>\n`
  with NO `Caused by`. Both pinned.
- `adr status` CLI surface = `adr status <ID> --status <S>` — `--status` is a
  FLAG, not a 2nd positional (the plan/sheet first assumed 2 positionals; probed
  and corrected). Enum: `proposed|accepted|rejected|superseded|deprecated`;
  hide-set = `rejected|superseded|deprecated`.

**Determinism rules for any future CLI golden here:**
- NEVER `adr new`/`adr status` to build a fixture — they stamp `clock::today()`
  into `created`/`updated`. Hand-seed the `adr-NNN.{toml,md}` tree with fixed
  dates instead (the `seed()` helper).
- Two carve-outs asserted structurally, not byte-exact: (1) the absolute tempdir
  path in `… not found at <path>` (match stable prefix + relative suffix); (2) a
  real `status` transition bumps `updated`→today (assert it MOVED off the seeded
  value, don't pin it).

Surprise/adaptation: none beyond the `--status`-flag correction. All 10 goldens
green on first run (strings transcribed from a probed fixture).

Mutate-to-red evidence (VT-2/VT-3): em-dash→hyphen in `format_show` reddens the
show table golden; `render_table` header `"id"`→`"ID"` reddens both list table
goldens (list JSON unaffected — correct, header is table-only). Reverted.

Follow-up for PHASE-02: cite `boot::tests::regenerate_projects_accepted_adrs…`
(`src/boot.rs:1017`) — it already drives adr output through boot end-to-end, a
second preservation signal beyond these CLI goldens. And `boot.rs` call site
`adr::list_rows` must rebind to `governance::list_rows(&ADR_KIND, …)` once
`list_rows` leaves `adr.rs` (plan PHASE-02 EX-4).

Candidate memory (reusable beyond this slice): "doctrine black-box CLI goldens —
hand-seed fixed dates, match anyhow's `Error:/Caused by` shape, carve out abs
paths." Adjacent to `mem.pattern.testing.conformance-asserts-surface-not-just-
envelope`. Recording next.
