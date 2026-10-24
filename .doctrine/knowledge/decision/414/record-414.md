# DEC-414: codex_mcp wiring: implementation deltas from the SL-286 design

Deltas adopted while implementing SL-286 PHASE-04 (wiring the codex user MCP
leg into `install` and `boot install`; commits ad689e094..224fb8fc1). Reconcile folds
each into `design.md`.

1. **One gather, in codex_mcp** — `read_plan(project_root, exec, env)` is the
   leg's shell half beside `execute`. The environment arrives through an
   injected lookup (`std::env::var_os` in production, a map in tests); PATH,
   both config files and the canonical paths come from disk. The design named
   no gather seam; install and boot install both call this one.
2. **exec resolves before the plan** — `install::run` resolves the running
   binary right after the project root (it used to resolve after the base
   plan), so the plan exists for the overview. `InstallPlan::build` takes the
   gather as a closure and calls it only for a project install with the codex
   harness (EX-2).
3. **Overview bullet suffix** — `render_overview(plan, yes)`; the bullet is
   labelled `codex`, beside the existing `codex` skills bullet.
4. **Consent seam** — `codex_mcp::ask_consent(plan, yes, ask)`: no proposal →
   NotAsked; `-y` → Granted; otherwise the whole `consent_block` (ending on its
   ask line) goes to `install::prompt_confirm`. No body/question split.
5. **Dry-run preview** — `codex_mcp::preview` = heading
   `Codex user MCP (not asked under --dry-run):` + the consent block. install
   prints it in `print_details` for `--dry-run` only (the `d` details view
   precedes the real question); boot install prints it after `wire`.
6. **Leg mapping** — `Leg::CodexUserMcp` runs after boot wiring; `Done(paths)`
   → touched paths with `Change::Changed` (the project file stays);
   `Refused` → `LegOutcome::Refused` (withholds the stamp).
7. **boot install** — consent is asked after its Wire prompt and before
   `wire`; the leg runs after `wire`. A `Refused` leg returns Ok: execute
   already printed it and boot install keeps no stamp.
8. **Relative or empty HOME degrades** (review F1) — `destination` treats a
   non-absolute `$HOME` as `NoHome`; a relative anchor resolves against the
   CWD, which inside a project is the project file read as the user entry.
   Cause text: "HOME is unset or not an absolute path".
