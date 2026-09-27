<!-- doctrine:section sec-1 -->
## 1. Design Problem

`boot install` registers the doctrine MCP server with **Claude** and with nobody
else. It merges `mcpServers.doctrine` into the project-root `.mcp.json`, and the
`Harness::Codex` arm of the same refresh carries `mcp: RefreshOutcome::None`. A
codex-driven session therefore reaches none of doctrine's MCP tools, and the only
way to close the gap today is for a human to run `codex mcp add doctrine --
doctrine serve --mcp` by hand — which has no scope flag and writes the **user**
layer (`~/.codex/config.toml`), not the project.

Codex reads its MCP servers from a TOML table, `mcp_servers.<id>`, in a
project-scoped `.codex/config.toml`. This design gives `boot install` a second MCP
leg that writes exactly that table, to the same standard the Claude leg already
meets: edit-preserving, idempotent, no-clobber, fail-soft, and disclosed.

The boundary is deliberately narrow. This leg **registers a server**. It does not
write codex's feature flags, does not touch the user layer, and does not add an
MCP tool. Three behaviours specific to codex shape it, all established by
evidence gathered before design:

1. codex does not interpolate `${VAR:-default}` in `command` — it execs the
   string literally, so the Claude leg's portable command cannot be copied.
2. codex hands an MCP child a *filtered* environment, so `DOCTRINE_BIN` survives
   only if the entry whitelists it.
3. codex silently ignores an untrusted project's config layer, so install cannot
   imply that what it wrote is live.

The result is a leg whose command form is portable and machine-path-free, whose
ownership predicate is explicit about which shapes are doctrine's, and whose
output tells the truth about what the harness will and will not do with it.

<!-- doctrine:section sec-2 -->
## 2. Current State

Line references are to `src/boot.rs` at `e569620cd`.

`install_refresh(harness, root, exec, dry_run)` runs one arm per harness. The
**Claude arm** calls `install_mcp` (`:2087`), which reads `.mcp.json`, plans
through `plan_mcp` (`:2034`) and writes through `fsutil::write_atomic` only on
change. `plan_mcp` fuses four steps over a `serde_json::Value`: parse, mutate at
the narrow path `mcpServers.doctrine`, classify, render. Its ownership predicate
`is_doctrine_mcp_entry` (`:2010`) owns two shapes — the current
`PORTABLE_EXEC` command (`${DOCTRINE_BIN:-doctrine}`, `:613`) and a legacy
absolute path whose file name is `doctrine` — and its no-op branch compares the
stored command against `PORTABLE_EXEC` (`:2056`). A foreign or customised
`doctrine` key, a non-object `mcpServers`, or malformed JSON yields the
`PrintedFallback` sentinel, rewritten by `install_mcp` into a manual snippet.

The **Codex arm** (`:1677-1705`) merges `.codex/hooks.json` through
`codex_hook_specs` (`:1308`), passing `CommandForm::Baked` as a literal justified
by this repo's own `.gitignore` (`:2331-2338`), runs the three pi-extension legs,
and sets `mcp: RefreshOutcome::None` (`:1702`). No function resolves a file's
tracking status; the form is a per-call argument.

`RefreshReport` (`:1774`) carries one `mcp: RefreshOutcome` field (`:1787`).
`wire()` (`:2851-2878`) matches it and prints a message naming `MCP_REL`, which is
a constant (`:594`), not a value carried by the outcome. `None` prints nothing.

`boot.rs` contains **no** `toml_edit` usage: every write in the module is
`serde_json` plus `fsutil::write_atomic`. The house edit-preserving TOML seam is
`dep_seq::set_authored_status` (`src/dep_seq.rs:433`, parsing to
`toml_edit::DocumentMut` at `:299`), used by `backlog.rs` and `knowledge.rs`.

`.codex/config.toml` is **not** written by doctrine today.
`write_codex_activation` (`:2684-2712`) only *tells the human* to set
`[features] hooks = true` in it (`:2696`) and to trust the hooks via `/hooks`.
The Claude leg's four pinned assertions on `out.mcp` live at `:5121`, `:5129`,
`:5157` and — the codex-arm line that must flip — `:5170`. `tests/` has no codex
install e2e and `tests/e2e_claude_install.rs` makes no MCP assertion at all.

<!-- doctrine:section sec-3 -->
## 3. Forces & Constraints

| authority | constraint it places on this design |
|---|---|
| ADR-001 (layering) | `boot` is command tier. A shared classification core carries no format and no caller's concept; each arm owns its parse and render. |
| ADR-013 | A change to SPEC-011's requirement text lands through a Revision. This slice does not *wait* on that change — the Revision retro-describes behaviour that ships here — so it carries no `needs` anchor; the reason is recorded in §9 rather than left implicit. |
| POL-002 facets 1–2 | No host absolute path may land in an artefact a client tracks. Install's `[gitignore].entries` add nothing for `.codex`, so a client's `.codex/config.toml` is tracked. |
| POL-002 facet 3 | A host-tool dependency must be **declared**, never silent. The portable form runs through a POSIX `sh`. |
| POL-003 facets 1–2 | Codex vocabulary (`mcp_servers`, `env_vars`, `command`, `args`) stays at the codex edge. The table's *shape* is a versioned seam: the design records the write as a documented version delta with a follow-up (facet 2's sanctioned outcome) rather than treating it as a stable contract. |
| POL-003 facet 3 | The supplement is opt-in and **disclosed**: report what was written, name any step that remains, and never claim an entry is active while codex still requires project trust or while the write's shape may have moved. |
| STD-001 | Every recurring literal gets one named constant, and where `const` cannot compose, a test pins the copies together. |
| SPEC-011 REQ-186 | The merge posture to match: an ownership predicate over the entry, refreshing a stale owned copy, preserving every foreign hook and key. (Its text is the Claude settings file; it governs here by the posture, with the Claude `.mcp.json` leg as in-repo precedent.) |
| SPEC-011 responsibilities (prose) | The pure-plan/imperative-apply split `boot install` rides — a responsibility, not a requirement member. |
| SPEC-011 REQ-479 | **Precedent, not authority**: the codex hook-registry leg shows a codex-specific surface getting its own member. Citing it as the *rule* would misread it. |
| SPEC-011 REQ-480 | The per-leg reporting pattern: each generated file is reported as its own outcome rather than folded into another leg's line. |
| PRD-006 / SPEC-009 | The manifest decides trackedness; install never overwrites a file it cannot interpret. PRD-006's never-overwrite is refined, not carved out, by the ownership-aware merge. |
| STD-003 | Not textually binding here (its scope fence excludes client-owned harness config); its disclosure *principle* is honoured through POL-003 facet 3. |

**Motivating evidence** (not authority): IMP-249 — in the jail and in dispatch,
PATH `doctrine` is a read-only, possibly stale binary, and `DOCTRINE_BIN` is how
agents point at the right one. And the pre-design probe of codex 0.155.1, which
established the three behavioural facts the design turns on (see IMP-111).

**Analogy, not authority**: ADR-019 decides how doctrine projects the assets it
ships, minimally; it says nothing about mutating a file doctrine does not own.
The merge posture is governed by PRD-006 refined by SPEC-011's ownership-aware
merge, with the Claude `.mcp.json` leg (`src/boot.rs:2034`) as the working
precedent.

Gap the design inherits rather than closes: **no requirement covers MCP
registration for either harness.** The Claude arm shipped under a chore. A
SPEC-011 Revision is owed at close (see §6, §9).

<!-- doctrine:section sec-4 -->
## 4. Guiding Principles

1. **Portable, not baked.** The entry names no machine path. The override a host
   needs rides `DOCTRINE_BIN`, and where a shell is required to read it, the
   dependency is declared rather than assumed.
2. **The comparator tracks what is written.** A no-op branch compares the stored
   entry against the constants the installer emits, never against a
   hand-spelled variant — the failure mode this codebase has already paid for.
3. **Judgement shared, formats bespoke.** Which classes exist (absent /
   ours-current / ours-stale / foreign) and what each one *does* (wire, refresh,
   no-op, disclose) is one shared enum and one shared table, and **both arms call
   it**: `plan_codex_mcp` from the start, and `plan_mcp` refactored onto it, with
   the existing `plan_mcp_*` suite as the behaviour-preservation proof. Each arm
   derives its entry's class from its own parse with a total match — no boolean
   triple, so no unreachable combination is representable. Parsing, mutating and
   rendering stay per-arm.
4. **Strict ownership.** An entry is ours only in a shape doctrine has emitted:
   the exact key set, the exact arity, a wrapper line from the emitted-forms set,
   and either no `env_vars` or exactly the emitted whitelist. Anything else — an
   extra key, an extra argument, a different program, a user's own `env_vars`
   value — is foreign and left alone; doctrine heals its own output, never a
   user's edit.
5. **Never clobber, never pretend.** A file doctrine cannot interpret is left
   alone and a snippet is printed; a read that fails for any reason other than
   the file being absent is a degraded read, never an empty file; a soft failure
   is disclosed; no line claims activation.
6. **Impurity at the edge.** The planner is pure over text; the file read, the
   atomic write and the harness probe are shell seams, injectable so tests need
   neither a real codex nor a real config file.

<!-- doctrine:section sec-5 -->
## 5. Proposed Design

### 5.1 System Model

The leg hangs off the existing per-harness refresh, beside the codex hook merge,
and reports through the existing report seam. `wire()` is the caller: it runs the
refresh and receives the resulting report.

```mermaid
flowchart TD
  install["boot install"] --> wire["wire(runner)"]
  wire --> refresh["install_refresh(Codex)"]
  refresh --> hooks["codex hook merge<br/>.codex/hooks.json"]
  refresh --> mcp["install_codex_mcp<br/>.codex/config.toml"]
  refresh --> pi["pi extension legs"]
  mcp --> plan["plan_codex_mcp (pure)"]
  plan --> action["mcp_action (pure, shared)"]
  monkey["plan_mcp (Claude, refactored)"] --> action
  mcp --> write["toml_edit + write_atomic"]
  refresh --> report["RefreshReport.mcp"]
  report --> wire
  hooks --> wire
  wire --> probe["codex features list, cwd=root (probe, fail-soft)"]
  wire --> out["stdout: wrote-line, trust caveat, hooks warning"]
```

The decision table is the one shared element between the arms: `mcp_action` maps
an entry's class to what to do, and knows nothing about JSON or TOML. Each arm
derives its class from its own parse and renders its own payload. The probe hangs
off the hook write, not the MCP write — it answers a question about hooks.

### 5.2 Interfaces & Contracts

New constants beside `MCP_REL` (STD-001). `const` cannot compose strings, so the
constants each hold their own literal and **the test pins them together**:

```rust
const CODEX_CONFIG_REL: &str = ".codex/config.toml";
const CODEX_MCP_TABLE: &str = "mcp_servers";
const CODEX_MCP_SERVER_KEY: &str = "doctrine";   // pinned == MCP_SERVER_KEY
const CODEX_MCP_SHELL: &str = "sh";
const CODEX_MCP_SHELL_FLAG: &str = "-c";
const CODEX_MCP_SERVE_ARGS: &str = "serve --mcp";
const CODEX_MCP_WRAPPER: &str = "exec \"${DOCTRINE_BIN:-doctrine}\" serve --mcp";
/// Every wrapper line doctrine has ever emitted, newest first (today: one). A
/// wording change APPENDS its predecessor, so a previously-emitted wrapper stays
/// ours and refreshes instead of becoming foreign and nagging forever.
const CODEX_MCP_WRAPPER_FORMS: &[&str] = &[CODEX_MCP_WRAPPER];
const CODEX_MCP_ENV: &str = "DOCTRINE_BIN";
const CODEX_HOOKS_FEATURE: &str = "hooks";
```

Shared judgement — **total over parsed input**, with no `Malformed` arm and no
refusal channel. Each arm derives the class from its own parsed value with a
total match; the shared table decides what the class *does*:

```rust
pub(crate) enum McpEntryClass { Absent, OwnedCurrent, OwnedStale, Foreign }

/// The shared decision table — the judgement that can drift between the arms:
/// class -> (outcome, write?). `payload` is the arm's own rendering (the
/// invocation for Wire/Refresh, the pass-through snippet for Fallback) and
/// `file` the file that snippet concerns. Malformed is NOT here: it is a parse
/// outcome, decided by each planner before classification (the `plan_mcp`
/// sentinel shape).
fn mcp_action(class: McpEntryClass, payload: String, file: &'static str)
    -> (RefreshOutcome, bool);
```

Codex planner and shell. Malformed is planned, not classified:

```rust
/// Malformed covers: unparseable TOML; `mcp_servers` present but neither a table
/// nor an inline table; the `doctrine` entry present but neither. In that case
/// `new_toml` is None and the file is never opened for writing. No path indexes
/// `args` without a length check first.
struct CodexMcpPlan { class: Result<McpEntryClass, Malformed>, new_toml: Option<String> }

fn plan_codex_mcp(existing_toml: Option<&str>) -> CodexMcpPlan;
fn install_codex_mcp(root: &Path, dry_run: bool) -> anyhow::Result<RefreshOutcome>;

/// The entry, built ONCE: the writer inserts this item and the snippet renders
/// the same item standalone, so the two cannot drift.
fn codex_mcp_entry() -> toml_edit::Item;

/// The manual snippet for the fallback path — the `[mcp_servers.doctrine]`
/// table in TOML, never the JSON block `mcp_fallback_snippet` emits.
fn codex_mcp_fallback_snippet() -> String;
```

The file read is part of the shell, not the planner, and it distinguishes absence
from failure: `ErrorKind::NotFound` is `None`; every other read error
(permissions, a non-UTF-8 byte — i.e. not valid TOML) is malformed, never absent.
`install_mcp` currently reads with a blanket `.ok()` (`boot.rs:2089`), which turns
a non-UTF-8 or unreadable `.mcp.json` into "absent" and replaces the whole file;
applying the same distinction there is a recorded follow-up (backlog), not done
silently here.

The snippet and the write share one builder (above): `toml_edit` renders a new
string by preferring the basic form and falling back to a LITERAL string when it
contains a `"`, so the file carries `'exec "…" serve --mcp'` (single-quoted)
where §5.3 shows the equivalent `"exec \"…\" serve --mcp"`. Assertions are on
PARSED values, never on the bytes §5.3 displays.

Ownership is strict and stated as a formula. With `t` the entry (read through
`as_table_like()`, so an inline table classifies on its content, not its
spelling):

```text
normalise(l) = l.trim() with internal whitespace runs collapsed to one space

owned = t.keys ⊆ {command, args, env_vars}
     && t.command == CODEX_MCP_SHELL
     && t.args.len() == 2 && t.args[0] == CODEX_MCP_SHELL_FLAG
     && CODEX_MCP_WRAPPER_FORMS.contains(&normalise(t.args[1]))
     && (t.env_vars is absent || t.env_vars == [CODEX_MCP_ENV])
current = t.args[1] == CODEX_MCP_WRAPPER && t.env_vars == [CODEX_MCP_ENV]
```

The length check precedes every index: `args = []` and `args = ["-c"]` are
`Foreign`, never a panic. Extra keys or a third argument make the entry foreign —
doctrine heals its own output, never a user's edit.

`owned && !current` is `OwnedStale`, and it is reachable in exactly three ways,
each deliberate: the whitelist is *absent* (the dead-override mitigation — a
wrapper with no `env_vars` silently falls back to the PATH binary); the wrapper
line is a spacing/reformatting variant that normalises into the emitted set (a
canonicalising refresh); or it is a wording doctrine emitted earlier and now
retained in `CODEX_MCP_WRAPPER_FORMS`. Anything doctrine has *not* emitted is
`Foreign`: a user's own `env_vars` list (any value but absent or exactly the
emitted one, including a wrong type or an extra element), a different program, an
extra key, an extra argument. A plain-literal `command` and a baked abspath are
foreign for the same reason.

The probe, split pure/imperative, over a capture-capable seam that the existing
installer runner cannot provide (it uses `.status()` and inherits stdio):

```rust
struct Capture { success: bool, stdout: String, stderr: String }

/// The seam `wire()` takes as a parameter. `install::CaptureRunner` is the
/// default implementation (Command::output()); tests inject a fake. `cwd` is
/// the install root, so the probe answers for the project being installed —
/// `--path`, or install run from a subdirectory, must not probe another tree.
trait CommandRunner {
    fn run_capture(&self, program: &str, args: &[&str], cwd: &Path) -> anyhow::Result<Capture>;
}

enum HooksState { Enabled, Disabled, Unknown(String) }

/// Pure. The row whose first token is `hooks` contributes its final token:
/// "true" -> Some(true), "false" -> Some(false); no row or any other shape -> None.
/// `stdout` is read regardless of `success`: a failed command that still printed
/// a parseable row is a usable answer, and `stderr` is available for the reason.
fn parse_codex_features(stdout: &str) -> Option<bool>;
fn codex_hooks_state(run: &dyn CommandRunner, cwd: &Path) -> HooksState;
```

`install_refresh`'s Codex arm replaces `mcp: RefreshOutcome::None` with
`mcp: install_codex_mcp(root, dry_run).with_context(…)?` — the context names the
leg and what preceded it, mirroring the Claude hook loop's boundary context
(`boot.rs:1732`), so an aborted leg does not silently lose the report of hooks
already written. `wire()` gains the runner parameter and selects the reported
file from the harness it holds (`Harness::Codex => CODEX_CONFIG_REL`, otherwise
`MCP_REL`), carrying the full rendered invocation so neither arm re-appends
arguments. The codex messages state what was **written**, not that the harness
has activated it. The fallback line is ONE wording for both arms and both causes
(a foreign entry and an uninterpretable file): the shared table cannot tell them
apart, and the operator's next step is the same either way.

```text
registered MCP server in .mcp.json: <invocation>                    (Claude)
wrote MCP server registration in .codex/config.toml: <wrapper line>
would write MCP server registration in .codex/config.toml: <wrapper line>   (dry_run)
<file> has a doctrine entry doctrine did not write, or could not be read —
left untouched. To register it manually:
```

### 5.3 Data, State & Ownership

The emitted table, semantically (on the string spelling, see §5.2):

```toml
[mcp_servers.doctrine]
command = "sh"
args = ["-c", "exec \"${DOCTRINE_BIN:-doctrine}\" serve --mcp"]
env_vars = ["DOCTRINE_BIN"]
```

- **Placement.** The mutation is a `toml_edit` narrow-path insert at
  `["mcp_servers"]["doctrine"]`, creating `mcp_servers` when absent. When the
  parent table is absent the new table is appended at the document's end;
  otherwise it lands adjacent to `mcp_servers`, not at the end of the file.
- **Spelling.** Both `mcp_servers` and the entry are reached through
  `as_table_like`/`as_table_like_mut`, so a table and an inline table are the
  same thing to this leg. The entry is built once and inserted in the
  *container's* spelling — an `Item::Table` under a `[mcp_servers]` table
  (rendering as the block above), an inline value under an inline parent. The
  adapter is not cosmetic: `InlineTable`'s `TableLike::insert` unwraps its
  argument to a `Value`, so handing a `Table` to an inline parent PANICS. An
  owned-but-stale entry is replaced wholesale, the Claude leg's precedent.
- **Preservation.** Unrelated keys, tables and comments are preserved: the edit
  is narrow-path, not a typed round-trip. The claim is scoped to preservation,
  not to byte-for-byte identity of the whole file.
- **Ownership.** Strict (§5.2): only a shape doctrine has emitted. Other keys,
  other tables, a user's own `env_vars` value, and any `doctrine` entry that is
  not our wrapper — the naive plain-command form, `/bin/sh`, a baked abspath, our
  shape plus an extra key — are user-owned and never modified.
- **No `Baked` variant.** The entry is portable unconditionally, mirroring
  `plan_mcp`, which is likewise form-blind. This departs deliberately from the
  codex *hook* leg's hardcoded `Baked`: there is no tracking-status resolver to
  reuse, and the portable shape is safe under both trackedness outcomes.
- **State.** The only state is the file. No new runtime state, no cache, no
  watermark; the planner is a function of the file's bytes.

### 5.4 Lifecycle, Operations & Dynamics

```mermaid
sequenceDiagram
  participant W as wire(runner)
  participant I as install_refresh (Codex arm)
  participant P as plan_codex_mcp (pure)
  participant F as .codex/config.toml
  participant C as CommandRunner (install::CaptureRunner)
  W->>I: refresh(harness, root, exec, dry_run)
  I->>F: read (NotFound -> None; any other error -> malformed)
  I->>P: plan_codex_mcp(existing)
  P->>P: parse -> derive class -> mcp_action -> render
  alt class is OwnedCurrent
    Note over I: RefreshOutcome::None, no write, no output
  else class is Absent or OwnedStale
    I->>F: write_atomic (skipped when dry_run)
  else Err(Malformed) or Foreign
    Note over I: PrintedFallback + TOML snippet, file untouched
  end
  I-->>W: RefreshReport { mcp }
  alt h is Codex AND a hook was written AND not dry_run
    W->>C: codex features list, cwd=root (fail-soft)
    C-->>W: Capture { success, stdout, stderr }
    W->>W: activation notice; hooks step only unless Enabled
  end
  alt h is Codex AND the MCP entry was written AND the notice did not print
    W->>W: trust caveat
  end
```

Ordering and disclosure rules:

- The MCP leg is **independent** of the hook merge: it writes a different file,
  so no ordering guard is needed between them. The disclosure of a soft failure
  is carried by `PrintedFallback` (nothing was written) as distinct from `None`
  (already current). The outcome type is unchanged; the design adds no
  "did not attempt" variant, because the existing pair already carries the
  distinction.
- The MCP leg is written **last** in the codex arm, after the hook merge and the
  pi extensions, and its error carries boundary context naming how far the arm
  got. A `Result` still aborts the arm and the accumulated report is dropped —
  the Claude arm accepts the same cost and names the boundary instead.
- A malformed file yields `PrintedFallback` with a TOML snippet and no write.
  Install continues and the run ends green: the leg's failure mode is disclosure,
  not error (SPEC-011 REQ-186). A foreign entry yields the same line, and a
  repeat install produces the same stable line rather than a fresh instruction.
- **Exact disclosure predicate.** Three separate conditions, because they answer
  three different questions:
  - the MCP **wrote-line** prints iff `h == Codex && matches!(report.mcp, Wired |
    Refreshed) && !dry_run`;
  - the **hooks probe** fires iff `h == Codex && codex_hook_written && !dry_run` —
    the same signal that gates the activation notice (`boot.rs:2788`). `[features]
    hooks` gates codex HOOKS, not MCP servers, and the warning's own text is about
    "the hooks this install just wired", so keying it to the MCP outcome would
    both miss a dead hook write and warn about hooks nobody just wired;
  - the **trust caveat** prints iff `h == Codex && the MCP entry was written &&
    !dry_run && !codex_hook_written` — i.e. only when the activation notice (which
    already carries the trust step) did not print it. Once per arm, never twice.
- The probe's result folds into the activation notice: its "ensure `[features]
  hooks = true`" step is printed only when the state is not `Enabled` — `Disabled`
  names the key, `Unknown(reason)` names the reason. On `Enabled` nothing is said
  about hooks, per DEC-329.
- The trust caveat states that codex loads project-scoped config only for trusted
  projects and that an untrusted project's layer is skipped silently, so a
  `Wired` report is a statement about the file, never about activation.
- Under `dry_run` both this line and the pre-existing hook activation notice say
  **would write**; no output under `dry_run` contains "wrote".
- A second install over an unchanged file produces `RefreshOutcome::None`, no
  write, and no output.

### 5.5 Invariants, Assumptions & Edge Cases

**Invariants**

1. The written entry contains no host absolute path (POL-002 facets 1–2).
2. The no-op comparator compares against the same constants the renderer uses.
3. An entry that is not a shape doctrine has emitted is never modified.
4. A file that does not parse, whose `mcp_servers` shape is neither a table nor
   an inline table, or whose read failed for any reason other than absence, is
   never rewritten.
5. No planner path indexes `args` without checking its length, and no input
   aborts the install.
6. The leg's outcome is reported, and a soft failure is disclosed as such.
7. No install path writes the user layer (`~/.codex/config.toml`).
8. No output line claims activation, and none says "wrote" under `dry_run`.

**Assumptions** (probe-verified on codex 0.155.1 unless noted)

- The project layer may override `mcp_servers`, and `.codex/config.toml` is that
  layer. Re-verification against the live reference is a follow-up (POL-003
  facet 2 records the seam as a version delta).
- `sh` resolves through the PATH codex gives the server (its whitelisted
  environment) — **decided**, not assumed away: the only shadowing vector is a
  PATH entry containing the project directory, which the install caveat names as
  a residual.
- codex executes `command` literally and forwards only whitelisted env. The
  installer asserts only the emitted string; the runtime behaviour of
  `${DOCTRINE_BIN:-doctrine}` is codex's, and is not testable from doctrine.

**Edge cases** the suite must pin

| case | expected |
|---|---|
| file absent | file created (parent dir ensured), entry written, `Wired` |
| file present, no `mcp_servers` | table added, siblings and comments intact |
| entry present and current | `None`, no write, no output |
| wrapper, right command, `env_vars` absent | `Refreshed` (the dead-override refresh) |
| wrapper line a spacing variant, or a wording doctrine emitted earlier | `Refreshed` (canonicalised) |
| wrapper with a user `env_vars` value (extra element, wrong type, other value) | `Foreign`, file untouched |
| `command = "sh"`, `args = []` or `["-c"]` | `Foreign`, no panic, file untouched |
| plain `command = "doctrine"` | `Foreign`, "left untouched" line, file untouched |
| `/bin/sh`, or a wrapped baked abspath | `Foreign`, file untouched |
| our shape plus an extra key or a third argument | `Foreign`, file untouched |
| `mcp_servers` present but neither a table nor an inline table | `PrintedFallback`, file untouched |
| `mcp_servers` an inline table, `doctrine` absent | entry inserted inline, siblings intact |
| `doctrine` an inline table in the emitted shape | `None` — content, not spelling, decides |
| TOML does not parse, or the file is unreadable / not UTF-8 | `PrintedFallback`, bytes untouched |
| file carries `[features] hooks = true` and comments | both preserved |
| `codex` absent, `features list` fails, or the row is unrecognised | `Unknown(reason)` warning; install still succeeds |
| `DOCTRINE_BIN` unset at run time | emitted string unchanged; runtime fallback is codex's behaviour |

<!-- doctrine:section sec-6 -->
## 6. Open Questions & Unknowns

None blocking. The five design questions IMP-111 carried are settled (§7). What
remains is placement, observation, and recorded follow-ups:

- **Where the shell declaration lands.** POL-002 facet 3 requires the `sh`
  dependency be named in README/install documentation. The site is a plan-level
  choice (which document, which wording), not a design question.
- **The Revision's shape and timing.** One Specification Revision introducing
  **two** members — one retro-covering the shipped Claude `.mcp.json` arm, one for
  the codex `mcp_servers` leg — with their durable `REQ` ids minted by the
  Revision. The Revision is **created and applied at reconcile**, after the
  phases; no phase cites the new ids (the plan traces to IMP-111 and this design),
  so nothing cites an id that does not yet exist. Phases do **not** wait on it;
  close requires it landed or a recorded waiver (the ADR-013 obligation is
  discharged at the boundary that owns it).
- **Post-write verification.** Whether a later phase adds a harness-side check
  that the written table is the one codex reads (`codex mcp get doctrine`) is a
  follow-up, not part of this leg: the shape is a versioned seam (POL-003
  facet 2), and the report claims only what doctrine wrote.
- **`.mcp.json` read errors.** `install_mcp` reads with a blanket `.ok()`
  (`boot.rs:2089`), so a non-UTF-8 or unreadable `.mcp.json` is treated as absent
  and replaced. The codex leg's `NotFound`-vs-error split is the pattern;
  applying it to the Claude leg is a recorded follow-up (backlog item), not
  silently done here.
- **`codex features list` scope.** Resolved by construction: the probe runs with
  `root` as its working directory, so it answers for the project being installed
  rather than for whatever tree the shell happens to be in.
- **A third harness.** Cursor (IMP-245) inherits the same question; the shared
  decision table is the seam that makes its planner cheap.

<!-- doctrine:section sec-7 -->
## 7. Decisions, Rationale & Alternatives

Each decision is an accepted record; this section carries the current meaning and
the alternatives considered, not the chronology.

| decision | chosen | record |
|---|---|---|
| Command form | Portable `${DOCTRINE_BIN:-doctrine}` executed through `sh -c`, with `env_vars = ["DOCTRINE_BIN"]`; the POSIX-shell dependency is declared (POL-002 facet 3). | DEC-323 |
| Ownership & emitted-form set | Own the wrapper shape only (`command = "sh"` + a line in the emitted-forms set); `env_vars` absent or exactly the emitted whitelist; a stale wrapper refreshes to canonical. A plain `doctrine` literal, `/bin/sh`, baked abspaths and any user `env_vars` value are FOREIGN and left untouched. Comparator tests the emitted constants. | DEC-332 (supersedes the second half of DEC-324) |
| Sharing boundary | Separate pure planner and shell per arm; one shared `McpEntryClass` enum plus `mcp_action` (class → what to do), each arm deriving its class from its own parse. | DEC-328 |
| Report seam | One `mcp` field; `wire()` names the file from the harness; `PrintedFallback` keeps carrying its own file. | DEC-325 |
| Disclosure & file ownership | Register MCP only — never `[features] hooks = true`; probe the harness (`codex features list`) and warn on `false` or unknown; disclose the trust-gated skip. The probe's *trigger* is a design choice keyed to the hook write, not the MCP write — DEC-329 itself rejects coupling the hook leg's activation to the MCP leg. | DEC-329 |

The ownership set was narrowed during the adversarial pass (RV-399 F-7): the plain literal is foreign-by-design, not a migration input. The third pass (RV-399 F-32/F-33) named the emitted-forms set as the mechanism behind "an earlier wording refreshes", and tightened `env_vars` to absent-or-exact so a refresh can never clobber a user's own list; DEC-332 was amended to match.

Rejected alternative (F-37): refusing to write into an inline `mcp_servers` or an inline `doctrine` entry. It is safe but fails to register in a config codex reads identically. The chosen path mutates in place, because the alternative is to report a legal config as unregisterable.

Rejected alternatives, kept because they will be proposed again: baking an
absolute path (POL-002, and untracked-vs-tracked is unresolvable without a
resolver that does not exist); the literal `doctrine` command alone (loses the
override that the jail and dispatch depend on, IMP-249); reading
`[features] hooks` out of the project file (wrong by construction — the effective
value may come from the user layer); and threading the MCP leg through the
owner-locked hook merge core rather than sitting beside it.


<!-- doctrine:section sec-8 -->
## 8. Risks & Mitigations

| risk | why it bites | mitigation |
|---|---|---|
| Comparator/migration thrash | A no-op branch testing a form the renderer does not emit rewrites the file on every install — the SL-195 `F-1` failure. | The comparator reads the emitted constants; ownership is one strict shape; the emitted-form matrix and the constants-agreement test make drift fail loudly. |
| Stranding our own older wording | An ownership test that is exact string equality retires every installed entry the moment the wrapper wording changes: each becomes "foreign" and nags on every install — the false negative the first pass's probe list warned of. | Ownership is membership in `CODEX_MCP_WRAPPER_FORMS`; a wording change APPENDS its predecessor, so a previously-emitted wrapper refreshes instead of stranding. The matrix test enumerates the set. |
| Clobbering a user's `env_vars` | An ownership rule that admits any `env_vars` reaches owned-but-stale from a *user's* own list and rewrites it, dropping the variables they forwarded — while an extra element is tolerated as current. | `owned` requires `env_vars` absent or exactly the emitted array; any other value is `Foreign`. Strict for elements as well as keys. |
| A trust claim keyed to the wrong write | A probe hung off the MCP outcome leaves a dead hook write unwarned, and warns about hooks nobody just wired; the unchanged activation notice also tells every operator to set `hooks = true` even when the probe reports them on. | The probe hangs off `codex_hook_written`, its result folds into the activation notice (step 1 printed only when not `Enabled`), and the trust caveat prints exactly once per arm. |
| Parallel planner divergence | Two hand-maintained copies of "what is stale vs foreign" drift silently. | One shared `mcp_action` table both arms call (plus the class enum), with the existing `plan_mcp_*` suite as the unchanged behaviour-preservation proof. |
| Panic instead of fail-soft | Indexing `args` on a hand-written short entry aborts `install_refresh` — the opposite of the never-clobber posture. | Every index is length-guarded; short arities are `Foreign`; unit cases pin `args = []` and `["-c"]`. |
| Clobbering a user-owned file | `.codex/config.toml` holds `[features]`, comments and user keys. | Narrow-path `toml_edit` mutation; only a shape doctrine has emitted is owned; an e2e preservation assertion. |
| False or stale claim in output | A written table that codex ignores, a dry-run that says "wrote", or a repeat "register manually" line all misdescribe the state. | The report states what was written; `dry_run` renders "would write"; a foreign entry's repeat output is stable; the trust caveat names the untrusted-project skip. |
| Resting on an incidental seam | `.codex/config.toml`'s shape and `features list`'s output are both version-varying. | Unrecognised probe output degrades to a named `Unknown`; nothing is gated on the probe; the write seam is a recorded version delta with a post-write verification follow-up. |
| Dead override | Without `env_vars` the wrapper silently falls back to the stale PATH binary (IMP-249). | A wrapper with `env_vars` absent is `OwnedStale`, so it refreshes; `current` requires the emitted array exactly. |
| Undeclared host dependency | The `sh` wrapper acquires a POSIX shell on the default path. | Declared per POL-002 facet 3 (§6). |
| Requirements gap treated as closed | The Revision is raised at reconcile; a reconcile that skips it would close the slice with the surface permanently undelivered. | Close requires the two-member Revision landed or a recorded waiver; phases are allowed to proceed first (SL-250 / RV-350 precedent). |
| Test fixtures that cannot fail | An absence assertion over an unwritten file, a tautological agreement test, or an ownership fixture seeded by a real installer passes for the wrong reason. | Positive controls in the same test; the agreement test compares against a `format!` expectation built from the inputs; ownership fixtures seeded literally; the probe injected. |

<!-- doctrine:section sec-9 -->
## 9. Quality Engineering & Validation

**Unit — codex planner**, mirroring the `plan_mcp_*` cases plus the emitted-form
matrix: absent → `Wired` (file created, parent dir ensured); current → `None`;
wrapper with `env_vars` absent → `Refreshed`; wrapper with a spacing variant or an
earlier emitted wording → `Refreshed`; wrapper with a user `env_vars` value (an
extra element, a wrong type) → `Foreign`; `command = "sh"` with `args = []` or
`["-c"]` → `Foreign` (no panic); plain `doctrine` command → `Foreign`; `/bin/sh`
→ `Foreign`; wrapped baked abspath → `Foreign`; our shape plus an extra key or a
third argument → `Foreign`; `mcp_servers` neither a table nor an inline table →
`Malformed`; unparseable
TOML → `Malformed`; an inline `mcp_servers` parent (entry inserted inline,
siblings intact) and an inline `doctrine` entry in the emitted shape (`None` —
content, not spelling, decides); sibling servers and `[features]` preserved.
Assertions are on PARSED values, never on the bytes §5.3 shows — `toml_edit`
writes the wrapper as a single-quoted literal. The fallback unit case asserts the
snippet is the TOML table (§5.2's `codex_mcp_fallback_snippet`) and is rendered by
the SAME builder the writer inserts, never the Claude JSON block.

**Unit — shared decision table**, table-driven over the four classes, asserting
each maps to exactly one outcome and write/no-write decision. The class has no
`Malformed` arm and no impossible combination (each arm derives it from a total
match on its own parse), so the table needs no refusal channel. `Malformed` is
asserted at the planner, where it originates.

**Unit — Claude behaviour preservation.** The refactored `plan_mcp` keeps the
existing `plan_mcp_*` suite green **unchanged** (boot.rs:5292-5416); that suite,
not a new one, is the proof that moving the decision table into `mcp_action`
changed no planner outcome. The `Wired`/`Refreshed` carried string becomes the full
invocation (the wire message no longer re-appends ` serve --mcp`), which the suite
matches by wildcard — a report-seam change, not a classification one.

**Unit — constants agreement.** `CODEX_MCP_WRAPPER` equals
`format!("exec \"{}\" {}", PORTABLE_EXEC, CODEX_MCP_SERVE_ARGS)`; the args suffix
is built from `CODEX_MCP_SERVE_ARGS`; `CODEX_MCP_ENV` occurs inside
`PORTABLE_EXEC`; and `CODEX_MCP_SERVER_KEY == MCP_SERVER_KEY`, pinned with the
reason (one server, two harnesses) rather than shared by construction.

**Unit — probe.** `parse_codex_features` against the verified shape
(`hooks  stable  true` / `false`), an absent row, unexpected columns and garbage.
`codex_hooks_state` through an injected runner over **five** cases: success
carrying `hooks ... true` → `Enabled`; success carrying `hooks ... false` →
`Disabled`; success with empty stdout → `Unknown`; **non-zero exit carrying a
parseable row** → the parsed answer; runner error with useful `stderr` →
`Unknown` naming that reason. One case asserts the runner received `root` as its
working directory — a `--path` install must not probe another tree. No test
requires a real codex on `PATH`.

**Integration — codex install.** Four cases. (i) A project whose
`.codex/config.toml` carries `[features] hooks = true` and a comment: assert the
emitted entry equals §5.3 by PARSED value, the pre-existing keys and comment
survive, and a second run reports nothing to do; (ii) a project with no
`.codex/config.toml`: assert the file and parent directory are created; (iii) a
project whose `.codex/config.toml` holds a non-UTF-8 byte: assert the bytes are
unchanged and the fallback line prints (no clobber); (iv) `PATH` emptied: this
case uses the **real** `CaptureRunner` (no injection), so the empty `PATH` is what
makes the probe fail, and it asserts the warning carries a non-empty reason.
Cases (i)–(iii) inject the runner. Additional assertions: the probe fires on a
hook write even when the MCP entry was already current, and does not fire when
only the MCP entry was refreshed; a foreign entry's second-run output is stable
(not a fresh instruction); and no `dry_run` output contains the word "wrote" —
for the MCP line **and** the hook activation notice. Ownership fixtures are
seeded literally, and every absence assertion carries a positive control in the
same test.

**Verification alignment.**

- Existing Claude assertions (`boot.rs:5121`, `:5129`, `:5157`) keep their
  meaning; the codex-arm expectation at `:5170` flips, and a new assertion pins
  that a Claude-only run emits neither the codex caveat nor the hooks warning.
- A no-host-abspath assertion extends the existing portable-command discipline to
  the codex file.
- `doctrine check gate` at close (a fresh binary against the real corpus).
- The two-member Specification Revision is raised at reconcile; **close requires
  it landed or a recorded waiver**, while phases proceed without waiting on it
  (SL-250 / RV-350 precedent). No phase cites its `REQ` ids — the plan traces to
  IMP-111 and this design — so nothing cites an id that does not yet exist.

**Impact.**

| path | change |
|---|---|
| `src/boot.rs` | the codex constants (incl. `CODEX_MCP_WRAPPER_FORMS`); `McpEntryClass` + `mcp_action`; the `plan_mcp` refactor onto them (behaviour-preservation: existing `plan_mcp_*` suite unchanged); `plan_codex_mcp` / `install_codex_mcp` / `codex_mcp_entry` / `codex_mcp_fallback_snippet`; the read's `NotFound`-vs-error split; the spelling-aware write; `parse_codex_features` / `codex_hooks_state`; the Codex arm's `mcp` outcome and its error boundary; `wire()`'s runner parameter, per-harness file name, wrote/would-write wording, one fallback wording, the three disclosure conditions and dry-run gating; `write_codex_activation`'s `HooksState` parameter and conditional step 1; `run_install:2645`; three stale doc comments (`RefreshOutcome:940`, `RefreshReport.mcp:1787`, the `wire` MCP block `:2851`) |
| `src/install.rs` | `CaptureRunner` (`Command::output()` with `current_dir(cwd)`) and the default-injection point; `wire`'s production call site `install::run:414` |
| `src/boot.rs` (tests) | the codex planner matrix, the decision-table test, the constants-agreement test, the five probe cases (one asserting the probe's `cwd`), the wire-level Claude-only assertion, the three disclosure-condition assertions, the dry-run wording assertion, the four existing `wire` call sites (`:6647`, `:6663`, `:6683`, `:6702`) and the `:5170` flip |
| `tests/e2e_codex_install.rs` (new) | preservation + idempotence, create-from-absent, non-UTF-8 no-clobber, empty-`PATH` `Unknown(reason)` with the real runner |
| `README.md`, `install/` docs | the POL-002 facet 3 declaration of the `sh` dependency |
| `.doctrine/spec/tech/011/**` | touched only at reconcile, by the two-member Revision — never by a phase |

