# Notes SL-030: Policy entity kind (POL)

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-02 — governance.rs spine extraction; ADR migrated onto it

Done: `src/governance.rs` (NEW) is the command-tier shared spine — `GovKind
{kind, stem, statuses, hidden}` + compute/io (`list_rows`, `key`, `render_table`,
`json_rows`/`GovRow`, `read_doc`/`Doc`/`Relationships`, `parse_ref`, `format_show`,
`show_json`, `set_status`) + the `run_new`/`run_list`/`run_show` shell wrappers,
all `&GovKind`-parameterized. `src/adr.rs` reduced to a thin kind (descriptor +
`AdrStatus` enum/known-set + `is_hidden` + render/scaffold + 7 forwarders).
`boot.rs` rebinds `adr::list_rows` → `governance::list_rows(&adr::ADR_KIND, …)`.

**Behaviour-preservation proof (the point of the phase):**
- The 10 black-box goldens pass UNCHANGED — `tests/e2e_adr_cli_golden.rs` is
  byte-untouched (empty `git diff`). Lib test count held 698→698 (no test lost in
  the relocation). boot `regenerate_projects_accepted_adrs` green (EX-4).
- Mutate-to-red RE-proven against the *relocated* code: em-dash→hyphen in
  `governance::format_show` reddens `adr_show_table_is_byte_exact`. The net wires
  to the new location.

**Contract details that mattered (carry into PHASE-03 policy.rs):**
- The spine derives every user-facing literal from `g.stem` / `g.kind.prefix`:
  filenames `{stem}-NNN.{toml,md}`, "{stem} NNN not found at …", "malformed
  {stem} NNN … (regenerate via `{stem} new`)", JSON `{kind}`/object-key = `stem`,
  "Created {PREFIX} NNN", canonical ids `{PREFIX}-NNN`. POL gets all of these free
  by setting `stem="policy"`, `prefix="POL"`.
- `parse_ref` strips TWO literal cases (`{PREFIX}-` | lowercased), NOT
  case-insensitive — pinned executably now (`parse_ref("AdR-7").is_err()`).
- `show_json` hand-builds a `serde_json::Map` (dynamic stem key; `json!` can't take
  a runtime key). Output is pretty + BTreeMap-sorted + NO trailing newline. Repo
  serde_json has no `preserve_order` (confirmed: struct fields serialize sorted).
- a/an: message is "not an {prefix} reference" — byte-correct for ADR, renders
  "not an POL" for POL (cosmetic; deferred per design D1, not pinned in P2).
- The `run_status` enum→`&str` binding stays per-kind (binds `AdrStatus`); the
  spine `set_status` takes `&str` + an injected `today`. Policy mirrors this:
  `run_status(path,id,PolicyStatus)` → `governance::set_status(&POLICY_KIND, …)`.
- Layering held: `governance` (command tier) → `entity`/`meta`/`listing`/`root`/
  `clock`/`input`; `adr`→`governance`; `boot`→`governance`+`adr`. No engine/leaf
  depends on `governance`. The relocated tests import `crate::adr::ADR_KIND` —
  a cfg(test)-ONLY edge; production stays `adr`→`governance` one-way.

Gotcha: `git checkout <untracked-file>` is a silent no-op — a mutate-to-red probe
on the new (untracked) `governance.rs` did NOT revert; caught by re-grep. Revert
probes on new files by hand.

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
