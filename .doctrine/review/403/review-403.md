# Review RV-403 — reconciliation of SL-271

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

**Surface reviewed.** SL-271 at `slice/SL-271-codex-mcp-registration-during-install`
head `21620b316` in `.worktrees/SL-271` (benign fork, unlanded; the primary tree
holds another agent's untracked work, so `worktree land` would refuse
`tree-unclean` — see `mem.pattern.worktree.solo-land-refuses-unclean-shared-tree`).
This is a solo slice, not a dispatched one: there is no coordination tree and no
`candidate/*` branch, so the audit ran against the fork branch itself, and the
ledger is driven from the primary tree (a fork cannot host its turn verbs).
`review prime` therefore hashed the primary tree's 230 selector paths, not the
fork's — the staleness signal is meaningless for this pass by construction, not a
refusal. Commands that need the slice's runtime state (conformance, verify-vt,
gate) were run **in the fork**; their output is quoted as evidence.

Mode: **conformance** (post-implementation, tied to the slice). Facet
`reconciliation`, self-audit driving both roles.

**Lines of attack, in the order they paid.**

1. **Mechanical conformance first** — `doctrine slice conformance 271` (run in the
   fork; the source-delta registry resolves to the primary tree and the fork holds
   none of its own). Read every cell: 2 undeclared, 1 undelivered, 4 conformant.
   Ask of each what the design intended, not just what changed.
2. **Hold the implementation to the design's disclosure predicate** (sec-5.4,
   labelled "exact"): the three conditions, the message contract in sec-5.2, the
   `dry_run` verbs, the placement of the trust caveat against the activation
   notice. Re-check the probe's premise against `DEC-329` — the code review
   already falsified "the probe answers for the project being installed", so
   carry it here as the audit's own design-wrong finding with a brief entry.
3. **Governance coverage.** Which requirement owns the MCP registration leg? Read
   `SPEC-011` (the boot-install harness-wiring contract) for a requirement covering
   `.mcp.json` or `mcp_servers`. If none exists, the leg is ungoverned and the
   design's citation of a REQ for its failure mode needs checking.
4. **The verification evidence.** `doctrine slice verify-vt 271` for the 11 VT
   mandates; then the three `VA-1` agent checks, which no verb covers — discharge
   them by hand (the `plan_mcp_*` diff for PHASE-01, the README sentence and its
   home for PHASE-02, the three disclosure conditions for PHASE-03) and note where
   their records live, or fail to.
5. **The undelivered selector.** `install/**` is a declared `design-target` that
   matched no edit, while the design's impact table names "`README.md`, `install/`
   docs" as a home for the POL-002 facet 3 `sh` declaration and
   `install/claude-activation.md` is a shipped doc about exactly this activation
   surface. Decide: dropped work, or a stale declaration to drop.
6. **Doc/payload contract** (carried from the code review): `RefreshOutcome`'s
   stated meaning, and the codex Wired/Refreshed single form — both deferred to
   the design pass by `RV-402` `F-2`/`F-4`, so the brief must not lose them.

**Evidence run** (all in the fork unless noted): `slice conformance 271`,
`slice verify-vt 271` (11/11 PASS), `slice status`, `doctrine check gate` (exit 0),
`cargo test --bin doctrine mcp|codex`, `--test e2e_codex_install`,
`--test architecture_layering`, plus live `boot install` runs in dry, real,
idempotent, non-UTF-8, foreign and legacy-refresh states, and the `git diff` of the
`plan_mcp_*` block for PHASE-01's `VA-1`.

## Synthesis

**Overall: acceptable** — the implementation is solid and the governance around it
is not yet true.

**Closure story.** SL-271 lands a second MCP registration arm in `doctrine boot
install`: a `toml_edit` narrow-path write of `[mcp_servers.doctrine]` into a project
`.codex/config.toml`, an ownership predicate that heals only doctrine's own emitted
shapes, a per-harness report seam, and a probe that folds the codex hooks-feature
state into the activation notice. All three phases are complete, the rollup is 3/3,
all 11 `VT` mandates PASS, `doctrine check gate` exits 0, and the delta is scoped:
four `design-target` selectors matched real edits, and the two undeclared paths are
the slice's own plan and status bookkeeping. The mechanical signals are green, and
this audit did not manufacture a finding to justify its cost.

Where it stands: one blocker and three smaller corrections, none of them a coding
defect. **F-1** is the substantive one, and it is carried from the code review
(`RV-402` F-1), because the code review and this audit reached it by different
routes — one by probing the artifact, one by holding the design's own probe premise
to account. `codex features list` answers with the *user* codex layer until a
project is trusted, and install runs before trust; the design's claim that the
probe "answers for the project being installed" is therefore false, and the `step 1`
instruction can be suppressed for a project that disables hooks. **F-2** is the
governance hole that let this happen quietly: no requirement in `SPEC-011` covers
MCP registration at all — not the codex leg, and not the Claude `.mcp.json` leg that
shipped earlier under `CHR-013` — so the behaviour had no acceptance criterion
except this slice's own artefacts, and design sec-5.4 reached for `REQ-186`, which
governs the Claude hook-set merge. **F-3** is a design self-contradiction between
sec-5.4's "exact disclosure predicate" and sec-5.2's message contract (the code
follows sec-5.2). **F-4** is the undelivered `install/**` selector meeting the
design's own naming of `install/` docs as a home for the `sh` declaration: a
decision to make, not a defect.

Standing risks, all conscious:

- The design correction for F-1 may imply code (the probe mechanism). Reconcile
  writes design and governance, not code, so the brief names that as a further
  deliverable rather than smuggling a silent edit into a reconcile pass. This is
  the one thing that can move the slice's lifecycle position backwards.
- The Codex arm reports a stale-entry refresh identically to a fresh wire
  (`RV-402` F-4). Accepted for this slice because design sec-5.2 specifies one form;
  the question is carried to the design pass rather than decided unilaterally.
- The probe spawns `codex` on the installing machine, with a side effect on the
  user's global codex home (`~/.codex/tmp/arg0` rewritten during a scratch install).
  Declared only when it fails (`RV-402` F-5); if F-1 resolves by reading the project
  file, the subprocess and this risk both disappear.
- `STRICT ownership` heals doctrine's output and nothing else: a user's own
  `env_vars` value, an extra key, a third argument or a different program is
  `Foreign` and never touched, and a `Malformed` file yields the same one-line
  fallback as a foreign entry (one wording, two causes, by decision). Deliberate,
  and the matrix test is the proof.
- The `install/**` selector either goes, or the shipped corpus gains a sentence a
  client can read. Left as an explicit decision rather than drift.

Credit where the audit looked hardest and found nothing: the ownership formula's
index guards, the edit-preserving write (comment, `[features]` and siblings survive;
second run byte-stable; non-UTF-8 untouched), the inline-parent spelling, and the
Claude payload refactor's behaviour preservation (payload derived from
`desired_mcp_entry`, printed once, `plan_mcp_*` suite unchanged). The `VA-1`
substance all three phases claimed checks out; only their recording is thin.

**Haiku** — *the file says hooks off; / codex, untrusted, says on / the notice says nothing.*

## Reconciliation Brief

From `RV-403` F-1..F-4 (F-5 and F-6 are `aligned`). Grouped by write surface.

### Per-slice (direct edit)

- **design.md sec-5.2** (the probe paragraph: "`cwd` is the install root, so the
  probe answers for the project being installed") **and sec-5.4** (the probe's
  disclosure bullet) — `RV-403` F-1. Correct the claim: a pre-trust
  `codex features list` answers for the user layer, so the probe cannot speak for
  the project file the notice names. Record which remedy was chosen (the three
  options and my recommendation are in `RV-402` F-1's response — read the project
  file's `[features] hooks` key directly; or keep the probe and always print step 1
  with the probe result as a rider; or keep the mechanism and document the hole).
  The `DEC-329` record this descends from needs the same correction.
- **design.md sec-5.4** ("the leg's failure mode is disclosure, not error
  (SPEC-011 REQ-186)") — `RV-403` F-2. `REQ-186` governs the Claude hook-set
  merge; re-point the citation at the requirement the REV below will create.
- **design.md sec-5.4** (the "exact disclosure predicate", first bullet) —
  `RV-403` F-3. Restate harness-scoped: the Claude arm prints its
  registered/refreshed line, the Codex arm its single wrote/would-write form. This
  is also where `RV-402` F-4's deferred question (should Codex distinguish a
  refresh from a fresh wire?) is decided.
- **slice-271.toml** `[[selector]]` — `RV-403` F-4. Either drop the `install/**`
  `design-target` (README is the declaration's home, as `EX-9`/`VA-1` pinned) or
  keep it and owe the shipped-corpus sentence below.

### Governance/spec (REV)

- **SPEC-011** — `RV-403` F-2. One Revision, **two members** (`doctrine revision`;
  `SPEC-011` is the boot-install harness-wiring contract):
  1. a Claude `.mcp.json` MCP-server-registration requirement, retro-covering the
     behaviour shipped under `CHR-013` and never governed;
  2. a codex `[mcp_servers]` registration requirement covering this slice's leg —
     the strict ownership predicate, edit-preserving narrow-path write, the
     one-wording fallback, and the disclosure (registration states what was
     written, never activation).
  Extend the spec's scope enumeration to include the leg, which it currently omits.
  No phase cites either member's id, so no criterion is disturbed.

### Further deliverables (reconcile cannot write these)

- If `RV-403` F-1 is remedied by changing the probe's mechanism, and if F-4 keeps
  the `install/**` selector, then a code change (the probe) and a docs change (the
  shipped corpus) are owed. Both are outside reconcile's two write surfaces: name
  them at reconcile as a phase or a follow-up backlog item, and say which in the
  reconciliation outcome. Do not let a reconcile pass absorb a silent code edit.

### Carried from RV-402 (not re-raised)

- `RV-402` F-5 — the README host-dependency sentence. Its disposition was `fix-now`
  *coupled to F-1*: written once, after the mechanism is chosen. Record where it
  landed, or that F-1 obsoleted it.
- `RV-402` F-4 — the Codex refresh-verb question, resolved by F-3 above.

### Decision — user, recorded 2026-09-27

**F-1's remedy is option A: disclose from the project file, drop the probe.** The
user ratified this after the audit's synthesis, choosing:

- `install_codex_mcp` (or the Codex arm) reads `[features] hooks` out of the
  project `.codex/config.toml` — the file doctrine already reads and writes —
  instead of shelling `codex features list`;
- the probe machinery retires: `HooksState`, `parse_codex_features`,
  `codex_hooks_state`, `write_codex_activation`'s `Option<HooksState>`, `wire()`'s
  `runner` parameter, and `install.rs`'s `Capture` / `CommandRunner` /
  `CaptureRunner`;
- `POL-002` facet 3 improves as a by-product: no `codex` subprocess on the install
  path, so **`RV-402` F-5 is obsoleted** (README's Host-dependencies section keeps
  only the `sh` statement, which is already there and correct) and the
  `~/.codex/tmp/arg0` side effect goes with it.

Consequences the reconciler must carry, not re-decide:

1. **This owes code, and reconcile does not write code.** The slice's design
   back-edge (`reconcile -> design`, `doctrine slice status 271 design`) is the
   route: amend `design.md` sec-5.2/sec-5.4 and the `DEC-329` record, extend the
   plan with the phase that retires the probe and reads the file, then execute,
   re-audit and reconcile again. `PHASE-03`'s `EX-4`/`EX-5` stand as history
   (criteria are immutable-append); the new phase owns the replacement behaviour.
2. **The acceptance test from F-1 is still owed in that phase**: a project
   `.codex/config.toml` with `[features] hooks = false` must still deliver the
   step-1 instruction. Under option A the natural shape inverts — the filesystem
   fixture replaces the injected runner, so the fakes and the `PATH`-emptied e2e
   case retire with the seam.
3. **The two-member `SPEC-011` Revision is unaffected** and still owed as
   briefed above; it describes the registration leg, not the probe.
4. The design-back-edge move is refused while a live `dispatch/<slice>`
   coordination worktree exists and the slice is only movable out of a terminal
   status — neither applies (`SL-271` is at `audit`, no coordination tree).
