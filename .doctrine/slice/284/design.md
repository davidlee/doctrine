<!-- doctrine:section sec-1 -->
## Overview

Two install surfaces stop reaching outside the binary or across harnesses:

1. **Skills.** codex, pi and `universal` receive skills from the embed through the
   existing canon-plus-link convergence (SL-281), into `.agents/skills/`. The npx
   delegation is deleted (DEC-386, DEC-388).
2. **Pi boot.** Pi becomes its own boot adapter, separated from codex (ISS-479,
   DEC-389). Its generated extensions move out of `.pi/extensions/doctrine/` into
   the gitignored runtime tier, registered through `.pi/settings.json`
   (ISS-525, DEC-390).

```
             before                                   after
claude   embed→.doctrine/skills ← .claude/skills    (unchanged)
codex    npx (GitHub) → .agents/skills              embed→.doctrine/skills ← .agents/skills
pi       npx (GitHub) → .agents/skills              embed→.doctrine/skills ← .agents/skills
         boot: codex arm (hooks+MCP+pi ext)         boot: pi arm (ext in .doctrine/state/pi/
                                                     extensions + .pi/settings.json + AGENTS.md)
codex    boot: codex arm also writes pi ext         boot: codex arm = hooks + MCP + AGENTS.md
```

No migration code ships (DEC-387): an existing npx copy at a link site is kept and
disclosed by the existing `Shadows` notice; the maintainer's checkouts get a
one-off cleanup at close (sec-7).

Evidence: `research/research.md` (threads 1-3).
<!-- doctrine:section sec-2 -->
## Skills routing

**Registry** (`install/manifest.toml`): `[harness.codex]`, `[harness.pi]` and
`[harness.universal]` each replace `skills_via = "universal"` with
`skills = ".agents/skills"`. Doctrine never routes into `.pi/skills` (pi
discovers `.agents/skills` itself, pi 0.99.1 `docs/skills.md:61`, and warns on a
duplicate).

**Mechanism: unchanged.** `desired_set` (`src/install.rs:3026`) already emits one
link site per harness carrying a `skills` route. Three harnesses sharing
`.agents/skills` yield the same site three times; `Member.links` is a `BTreeSet`
(`src/install_ownership.rs:38`), so the duplicate collapses. Canon writes,
relink and prune follow from SL-281's convergence with no code change. A test
pins the shared-dir collapse.

**Gitignore: per link, already.** The managed block lists one anchored line per
doctrine link (`render_block`, `src/install_ownership.rs:366`), as it does for
`.claude/skills/<id>` today. So `.agents/skills/<id>` links are ignored with no
new code, and install leaves no tracked noise. Doctrine does not ignore the whole
directory: a user's own skills under `.agents/skills/` stay visible to git.

**Customising hint.** `render_block` emits a fixed comment directly under the
opening marker (a named constant, STD-001):

```
# >>> doctrine managed — rewritten by `doctrine install`; edit outside this block
# Customised a skill? Replace its link with a real dir; the next install drops
# it from this block. Until then, un-ignore it BELOW the block, e.g.
#   !/.agents/skills/<id>
# (git cannot re-include a path whose parent dir is itself ignored.)
```

The parenthetical matters (RV-423 F-7): a host rule ignoring a parent, such as
this repo's own `/.agents`, defeats the leaf negation. That rule is the host's,
so doctrine only names the limit. The `!` line must sit after the block, because gitignore's last match wins and
the block is rewritten on every install. Block detection keys on the marker
prefix (`BLOCK_START_PREFIX`, `src/install_ownership.rs:325`), so the extra
comment lines don't affect splicing; a splice test pins this. Existing block
snapshot tests change by exactly these lines.

**`skills_via` is deleted** from `HarnessSection` (`src/install.rs:92`) along
with the both-routes validation (`:2237`). `deny_unknown_fields` then refuses a
manifest that still carries it, which is correct for an embedded asset.

**`--only-memory` actually subsets (RV-423 F-1).** A pre-existing bug: both
`select_for_install` call sites (`src/install.rs:952`, `:1048`) pass only
`--skill`/`--domain`, so `--only-memory` links the whole catalog (Claude today).
Every harness now rides this seam, so it is fixed here: `--only-memory` resolves to
the memory pair (`record-memory`, `retrieve-memory`, one named constant) through
the same selection for preview and live convergence.

**Unknown `--agent` is refused** (inq-7). `resolve_install_harnesses`
(`src/install.rs:660`) becomes fallible: any explicit name not in the registry
fails before any write, naming the registered harnesses and suggesting
`universal` for any other harness that reads `.agents/skills`. The interactive
answer path (`parse_harness_answer`, `:725`) re-prompts or refuses on the same
rule instead of collecting `unregistered`.

**Global scope.** `--global` links under `$HOME/.agents/skills`, which is what
codex and pi read. npx's `universal` global dir is `~/.config/agents/skills`;
doctrine does not model per-scope dirs. This is accepted and noted in the spec
REV.
<!-- doctrine:section sec-3 -->
## Removing the npx delegation

Deleted outright (DEC-388). Nothing else consumes these:

| Item | Site |
|---|---|
| `RUNNER_NPX` / `RUNNER_BUNX`, `program_available`, `resolve_runner(_with)` | `src/install.rs:37`, `:2529-2556` |
| `Runner` trait, `ProcessRunner`, `delegate_argv`, `InstallOtherArgs`, `install_for_other` | `src/install.rs:2804-2893` |
| `DelegateTargets`, `delegate_targets`, `report_unregistered` | `src/install.rs:755-845` |
| `OFFER_SKILLS_VIA_NPX` and its branch in `install_offer` | `src/install.rs:785-810` |
| `NPX_SKILLS_NOTE`, `NPX_REVIEW` disclosures | `src/install.rs:2323`, `:1311` |
| `Leg::Skills` and its forward step | `src/install.rs:1066-1094`, `:1255` |
| `[install] repo` key + `DEFAULT_REPO` | `src/install_config.rs:13-17`, `:73-75` |

Outside `src/`: the `justfile` `reinstall` recipe (`justfile:261-264`) drops its
`npx skills add . --agent universal` line.

`run_forward_steps` then ends at `Leg::Convergence`. Leg-dependent logic
(`incomplete_legs`, `touched`, `stamp_decision`, commit hint) is keyed on the
enum, so dropping a variant is a compile-checked removal for the remaining legs.

**Behaviour tests that assert removed behaviour are rewritten, not kept green
unchanged (RV-423 F-2):**

| Test | Asserted | Becomes |
|---|---|---|
| `tests/e2e_install_flow.rs:368-400` | `NPX_REVIEW` disclosure | no npx disclosure; `.agents/skills` links listed |
| `tests/e2e_install_executor.rs:242-269` | `agents universal (delegate)` | direct links for universal |
| `tests/e2e_claude_install.rs:350-375` | npx offer text | offer reads `skills` for every registered harness |
| `tests/e2e_install_ownership.rs` (`pi_without_npx_*`) | pi skills need npx | pi links `.agents/skills` from the embed (plan PHASE-01 EX-4) |
| `tests/e2e_surface_neutral_roundtrip.rs:111-129` | codex writes pi extension at old path | pi writes it under the runtime dir |
| `src/boot.rs:8234-8265`, `:9223-9242` | `Harness::Codex` writes pi files | split: codex writes none, pi writes them |

Every other suite stays green unchanged (the behaviour-preservation gate).

A leftover `repo` key in a client `doctrine.toml` is silently ignored, because
`InstallConfig` does not deny unknown fields (inq-8, accepted).

Docs and comments teaching npx are rewritten: `README.md:128,157,268`, the
`install/manifest.toml` registry comment, module docs in `src/install_config.rs`,
the `skills::parse_agent` mention in `src/boot.rs:736`, and the local memory
`mem.fact.doctrine.agents-skill-mirror-is-published-source` is retired at close
(`doctrine memory status`), since its trap no longer exists.
<!-- doctrine:section sec-4 -->
## Pi boot adapter

**Vocabulary** (`src/boot.rs`):
- `enum Harness { Claude, Codex, Pi }`, plus the matching arms in
  `parse_harness`, `harness_label` and the `parse_harness` error text.
- `import_targets(Pi)` = `AGENTS.md`, which is shared with codex. The existing
  same-inode dedup means selecting both prepends the ref once (inq-6).
- `resolve_harnesses` auto-detects `Pi` on `.pi/` only. `AGENTS.md` continues to
  imply codex alone.

**Arms of `install_refresh`** (`src/boot.rs:1781`):

| Arm | Writes |
|---|---|
| Codex | codex hooks (`.codex/hooks.json`), spike check, codex MCP (`.codex/config.toml`). No pi files. |
| Pi | the three generated extensions (`index.ts`, `mcp.ts`, `surface.ts`) under `PI_EXT_DIR_REL`, plus the `.pi/settings.json` `extensions` entry (sec-5). No codex hooks or MCP. |

**Report.** The codex arm reports the extension legs as not applicable, and the
pi arm reports hooks and MCP as not applicable. This uses the existing
`NotApplicable` idiom (`AppendSystemOutcome`, `BaseRefOutcome`), extended to the
extension and MCP outcome types where they lack it.

`RefreshReport` (`src/boot.rs:1892`) gains a **pi-settings outcome** (RV-423
F-4). The settings write joins `refresh_touched` (`:3525`), so it reaches the
install commit hint. Its malformed fallback joins `refused_wiring` (`:3199`),
so the boot leg is not stamped complete while the extensions are unregistered.

**MCP report sites (RV-423 F-6).** `mcp_rel` (`:3514`) and the MCP write-verb
table (`:3386`) are total over `Harness`. Both become conditional on an
applicable MCP outcome, so pi never names a Claude or codex MCP file.

**Activation disclosure (RV-423 F-5, POL-003).** Pi loads project `.pi`
settings only after project trust, and a running session needs `/reload` (pi
0.99.1 `docs/configuration.md:3-5`). The pi arm prints one activation notice on
any extension or settings write, mirroring `write_codex_activation` (`:3312`).
It states what was written, never that it is active. Its closing line promises
that `/reload` loads the changes only when the settings registration was
written; otherwise it says the files stay unloaded until the registration is
added (RV-424 F-5).

**Install registry:** `[harness.pi] boot = "pi"`. `boot_names`
(`src/install.rs:637`) needs no change: it maps the registry value through
`boot::resolve_harnesses`, which now parses `pi`.

**Out of scope:** `install_append_system` (`src/boot.rs:2772`) has no
production caller; it is noted as a follow-up, not revived here.
<!-- doctrine:section sec-5 -->
## Pi extension location

DEC-390: generated extensions leave the client's source tree.

- `PI_EXT_DIR_REL` (`src/boot.rs:52`) becomes `.doctrine/state/pi/extensions`.
  `.doctrine/state/` is already gitignored by the managed block. The ownership
  header, plan and foreign-skip logic (`plan_extension` /
  `install_extension`) are unchanged, because they key on the directory
  constant.
- **New: `.pi/settings.json` merge.** Pi loads extension files or directories
  listed under `extensions`, with relative paths resolved from the settings
  file (pi 0.99.1 `docs/settings.md:150`, `docs/packages.md:40`). The pi arm
  ensures the array contains `"../.doctrine/state/pi/extensions"`.
  - It is a pure planner beside `plan_baseref` (`src/boot.rs:1964`), the
    narrow-key `serde_json::Value` precedent, reusing `parse_settings`.
  - Absent file or key: write. Entry already present: no-op. Other entries are
    preserved in order.
  - Malformed file, `extensions` that is not an array, or a settings file that
    is a symlink (a rename would sever a dotfiles-managed link): leave the file
    untouched and print what it must list — the entry, under its `extensions`
    array — rather than a mechanical edit a non-array `extensions` makes
    impossible (STD-003; RV-424 F-5). The outcome is reported per sec-4.
  - An empty or whitespace-only file counts as an empty object, as
    `parse_settings` (`src/boot.rs:1648`) already does for Claude and codex
    settings. It is written, since an empty file holds nothing to lose. This is
    deliberate, not the malformed case (RV-423 F-8).
  - The literal is a named constant (STD-001).
  - **Fidelity caveat, accepted (RV-424 F-6).** The planner re-serialises the
    whole document through `serde_json::Value` (no `arbitrary_precision`), so
    "preserved in order" means order and structure: an integer outside the
    i64/u64 range is not preserved bit-exactly. A textual splice of the array
    would be more complex and more brittle for a vanishingly rare input.

**Shared-writer change (RV-424 F-3).** The settings write required hardening
`fsutil::write_atomic`, which every atomic write in the binary uses: it now
preserves an existing target's mode. Without it, rewriting a private `0600`
settings file republished it at the umask default (found by PHASE-04's review).
The residual gaps — the temp is written before the mode is applied, and special
bits are copied — are IMP-520, not this slice.

**Risk, accepted.** `.doctrine/state/` is disposable. Wiping it removes the
extensions (and `boot.md`) until `doctrine boot install` runs again, and pi
reports the listed directory missing in the meantime. This is the same class
as `boot.md` today.

**Residual.** A client lint config that honours neither `.gitignore` nor any
`.doctrine/` ignore could still see the files. ISS-525 option 3 (user-level
extensions) remains the complete fix if that bites.
<!-- doctrine:section sec-6 -->
## Spec revision

One REV, raised during execution and applied at reconcile. It covers structured
and prose occurrences (RV-423 F-3). `doctrine spec validate` runs on all three
specs after it is applied.

| Spec | Clause | Change |
|---|---|---|
| SPEC-010 | Overview (non-Claude "delegated") | Every registered harness links from the embed |
| SPEC-010 | `D2`, `REQ-175` (FR-003), responsibilities line 3 | `skills` link route for every registered harness; npx removed; unknown agents refused |
| SPEC-010 | Concern "Delegate version skew (accepted)" | Retired |
| SPEC-010 | Concern "Multi-agent status fidelity" | Retired: every harness is a dir doctrine owns |
| SPEC-010 | Hypothesis "Delegate the long tail" | Replaced: a new harness is a registry entry |
| SPEC-010 | Concerns (new) | `universal --global` lands at `~/.agents/skills` |
| SPEC-009 | Overview, `D4` (registry: `skills_via` / npx) | Registry carries one skills route, `skills` |
| SPEC-009 | `D5`, `REQ-172` (NF-003) | Drop "delegates skills … through npx"; the memory-sync clause stands |
| SPEC-011 | `REQ-185` + its responsibility (import targets, auto-detect) | Add pi: `AGENTS.md` import, `.pi/` detection |
| SPEC-011 | `REQ-480` + its responsibility (generated pi extensions) | Owned by the pi adapter; runtime-tier location; `.pi/settings.json` registration with its outcome and activation notice; codex arm writes no pi files |

**DEC-377 is superseded, not revised (RV-424 F-1).** DEC-377 still described
the npx delegation and pi→codex boot routing. A decision record is not a legal
`revises` target, so it has no row here: reconcile minted DEC-391
and superseded DEC-377 with it. The spec half of the same claims is this REV.

The REV's own scan repeats `doctrine search npx -k all` and
`doctrine search skills_via -k all`, so no hit is left unaddressed.
<!-- doctrine:section sec-7 -->
## Code impact and verification

**Design-target selectors:** `install/manifest.toml`, `src/install.rs`,
`src/install_config.rs`, `src/boot.rs`, `src/install_ownership.rs` (block
hint), `src/fsutil.rs` (sec-5), `install/doctrine.toml`,
`install/doctrine.toml.example`, `src/dtoml.rs` (the `[install] repo` key),
`README.md`, `justfile`, and the `tests/` suites in sec-3. `src/commands/cli.rs`
was conditional on `--only-memory` plumbing needing it; it did not, and its
selector is removed (RV-424 F-7).

**Tests (VT):**
- An install selecting pi, codex or universal, run with a runner seam absent
  (no npx code exists), produces `.agents/skills/<id>` relative links into
  `.doctrine/skills/<id>` matching the embed, and writes no `skills-lock.json`.
- The shared dir collapses: pi, codex and universal together produce one link
  per skill.
- The managed gitignore block lists each `.agents/skills/<id>` link, and a
  foreign entry in that dir is not listed.
- The block carries the customising hint under its opening marker, and
  re-splicing a block that has the hint is byte-stable.
- A real directory at `.agents/skills/<id>` is kept and reported `Shadows`.
  This is the existing behaviour, pinned for the new target.
- An unknown `--agent` is refused before any write, and the message names the
  registered harnesses.
- A codex-only `boot install` writes no `.pi/` or `.doctrine/state/pi/` file.
  A pi-only one writes no `.codex/` file.
- `.pi/settings.json` planner cases: absent, present without the key, already
  present, foreign entries preserved, and malformed left untouched.
- Pi auto-detects on `.pi/`. `AGENTS.md` alone implies codex, not pi.

- `--only-memory` with any harness links exactly the memory pair (F-1).
- The pi-settings outcome: a successful write appears in the commit hint; a
  malformed fallback marks the boot leg incomplete (F-4).
- A pi install prints the activation notice on write, not on a no-op (F-5).
- A pi-only boot install report names no MCP file (F-6).
- `.pi/settings.json` empty or whitespace-only is written; malformed is left
  untouched (F-8).
- The rewritten suites in sec-3's table.

**Manual (VH, at close):**
1. One-off cleanup of the maintainer's checkouts: remove the npx copies in
   `.agents/skills/` and `.pi/skills/`, `skills-lock.json` (tracked: delete it
   in a commit), and the old `.pi/extensions/doctrine/`. Then run
   `doctrine install`.
2. A1 (inq-10): live pi and codex sessions load doctrine skills through the
   `.agents/skills` symlinks, with no duplicate warning. Pi loads the extensions
   from the runtime directory.

**Gate:** `doctrine check gate` green, and the REV approved and applied.
