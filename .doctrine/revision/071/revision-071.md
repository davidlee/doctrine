# REV REV-071 — Install converges framework-owned artefacts

Revision — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-281 gives `doctrine install` an ownership axis (ADR-019's *owned* property):
framework-owned artefacts — the canon roots `.doctrine/{agents,skills,workflows}`
and the symlinks pointing harness dirs into them — are installed in full,
refreshed, pruned on project installs and gitignored through one managed block;
user-owned files keep write-if-absent. DEC-376 routes the spec change through
one Revision, drafted at `/plan` against the text current on 2026-10-16 and
applied at `/reconcile` once the code proves the behaviour. Decision basis:
DEC-372 (explicit `--agent` is integration), DEC-373 (manifest registry),
DEC-374 (convergence and recognition), DEC-375 (managed block), DEC-377
(global installs report, never prune).

SL-281's design §Revision lists SPEC-009, REQ-170 and REQ-176. Plan-time
re-reading found more live text the change contradicts — the same class as
RV-418 F-7 — so this REV also carries REQ-167, REQ-172, REQ-180 and SPEC-010's
prose. Rows are all `modify`; excerpts below.

REQ-170, REQ-176, REQ-167, REQ-172 and REQ-180 are title-only (empty
statement bodies); their rows retitle them and fill the statement.

Not revised: REQ-178 (re-prove at mutation time) holds as written; REQ-179
(atomic staged swaps) holds — unique exclusive staging names refine it, they
don't contradict it; DEC-010's published set is unchanged.

## SPEC-009 (primary)

### D2

**Before.** "D2 — install is idempotent and never overwrites. Files are written
only when absent, directories are create-if-missing, and gitignore lines are
deduplicated; re-install is a no-op and local edits survive. The cost — install
never updates an existing installed file — is accepted."

**After.** "D2 — install acts by ownership. User-owned files (the base
backings, hymn starters, authored entities) are written only when absent and
never overwritten or pruned. Framework-owned artefacts — the `[framework]`
canon roots and doctrine links into them — converge: every shipped member is
written and refreshed, and on a project install anything else inside a canon
root is removed and every removal reported; a global install writes and
relinks but only reports orphans. In a shared harness link dir a symlink whose
target lies inside a canon root is doctrine's (relinked or removed); anything
else is foreign and kept. Directories are create-if-missing. Framework
gitignore lines live in one managed block rewritten each project install."

### D4

**Before.** "…The manifest declares the minimal base fileset (D7), the
materialize-on-demand surfaces (D8), the dirs, gitignore negations, and root
markers…"

**After.** append: "It also declares ownership: `[framework] roots` (the canon
roots install converges) and the `[harness.*]` registry — per harness its
detect dir, boot adapter, agent link dir and one skills route (`skills` link
dir or `skills_via` npx target). The harness vocabulary is the registry; an
unregistered name gets no defs or links and is disclosed."

### D5 and the Overview boundary paragraph

**Before (Overview).** "Skills distribution is a sibling container (PRD-003 /
SPEC-010): install does not own the skill symlink tree — it only ensures the
skills gitignore negation and shares its confirm prompt; the skill provisioning
mechanism lives next door, and install *uses* it as a peer."

**Before (D5).** "The skill symlink tree is a sibling container's mechanism
(install only ensures its gitignore negation and shares the confirm prompt)…"

**After (both).** "Install owns the skill canon tree and its links for
harnesses with a `skills` route (true since SL-250); SPEC-010 specifies that
mechanism. For harnesses routed `skills_via`, install delegates to `npx skills
add`, a peer it uses but does not own." The memory-sync half of D5 is
unchanged.

### D7

**After.** append: "An explicit `--agent` harness leg is integration, not
default projection: it installs that harness's full agent-def set and links,
which the base projection never does (DEC-372)."

### Manifest section — `[gitignore].entries`

**Before.** "lines appended to the project `.gitignore`…"

**After.** "the runtime-state part of the managed gitignore block. The block is
rendered from these entries, a `<root>/*` line per framework root and one
anchored line per doctrine link present after the run, and spliced between
fixed markers, replacing the previous block. Lines outside the markers are
never read or changed; a malformed marker pair is refused and the file left
untouched."

### "Plan then execute, idempotently"

**Before.** "…The plan is a list of typed steps — create-dir, install, skip,
gitignore… files are written only when the destination is absent and never
overwritten, and `.gitignore` lines are appended only when not already present…
Re-running install is a safe no-op…"

**After.** "The base plan is typed create-dir / install / skip steps;
framework-owned paths get an ownership plan of write / link / relink / unlink /
prune / report actions computed purely from the declaration, the embedded set
and a disk snapshot. `--dry-run` prints both, including would-prune actions,
and changes nothing. Re-install leaves user-owned files untouched and converges
framework-owned ones; the executor re-proves each entry before mutating it.
Removals never pass through a symlinked ancestor."

### Concerns

- "Never-overwrite is the safety contract" → narrowed to user-owned files;
  framework canon roots are declared framework-owned and gitignored, so a user
  file placed inside one is removed and reported (accepted).
- "entries are deliberately narrow and additive" → "the managed block is
  narrow and rewritten; user `!` rules placed after it survive re-install".

### Hypotheses

- "Idempotent never-overwrite is the right default" → "…for user-owned files;
  framework-owned files converge so a re-install pushes updates and retires
  removed instructions."
- "One manifest drives the whole lay-down" → extended: "…including the
  ownership declaration and harness registry."

## REQ-167 (SPEC-009 FR-004)

**Before (title).** "Build an inspectable plan of create-dir, install, skip, and
gitignore steps before executing"

**After (title).** "Build an inspectable plan — base steps plus framework-owned
write, link, relink, unlink and prune actions — before executing"

## REQ-170 (SPEC-009 NF-001)

**Before (title).** "Install is idempotent: files written only when absent and
never overwritten, dirs created-if-missing, gitignore lines deduplicated"

**After (title).** "Install is idempotent by ownership: user-owned files
write-if-absent, framework-owned artefacts converge and are reported, dirs
created-if-missing, the gitignore block rewritten"

**After (statement).** "Two consecutive project installs with the same binary
leave the tree byte-identical after the second; user-owned files are never
overwritten or removed; every framework-owned write, relink, unlink and prune
is reported."

## REQ-172 (SPEC-009 NF-003)

**Before (title).** "Install owns neither skills distribution nor memory sync,
treating both as a sibling verb it hints at or defers to"

**After (title).** "Install does not absorb memory sync, and delegates skills
only for harnesses routed through npx"

## SPEC-010

- **Responsibility 4 and §Canonical tree and proven-ownership links.**
  **Before.** "a managed link is Doctrine's iff its value equals the relative
  target we would write…" **After.** "a link is Doctrine's iff it is a symlink
  whose target lies inside a framework canon root — a relative value joined to
  its link dir and normalised lexically, an absolute value compared
  component-wise under the base. Ours and mapped to a shipped member: relinked
  to the current layout (heals stale layouts). Ours and unmapped: unlinked on a
  project install, reported on a global one. Anything else: kept foreign."
- **Responsibility 5 and §Atomic lay-down.** "a `.tmp-<id>` staging sibling" →
  "a unique `.tmp-<name>.<pid>.<n>` staging entry made by exclusive create;
  staging entries are never pruned".
- **Responsibility 6 (self-enforced gitignore).** Replaced: the project's
  managed block carries `.doctrine/skills/*`; a global install writes no
  ignore.
- **Concerns "Never-clobber"** — "ownership-by-target-value" → "ownership by
  target-inside-a-framework-root".
- **D3** — as responsibility 4. **D4** — "self-enforces the
  `.doctrine/skills/*` ignore" → "is covered by the managed gitignore block".
- **Overview.** "every other agent delegated" → "every harness without a
  doctrine skills route delegated (pi and codex via the `universal` target)".

## REQ-176 (SPEC-010 FR-004)

**Before (title).** "Install Claude as a derived canonical skills tree with
relative agent symlinks reconciled by proven target-value ownership"

**After (title).** "Install Claude as a derived canonical skills tree with
relative symlinks owned when their target lies inside a framework root"

**After (statement).** "A symlink in a registered link dir whose target lies
inside a canon root is doctrine's and is relinked to the current layout or
removed; a stale-layout link is healed; a non-symlink or a symlink elsewhere
is kept untouched."

## REQ-180 (SPEC-010 NF-003)

**Before (title).** "Self-enforce the derived .doctrine/skills gitignore
anchored at the tree's base so it follows the tree under global"

**After (title).** "The derived .doctrine/skills tree is covered by the
project's managed gitignore block; a global install writes no ignore"

Note: the "follows the tree under global" half has had no implementation since
IMP-226 removed `skills.rs`; `ensure_gitignored`'s only caller today is the
base plan's `Step::Gitignore`. This row records truth, not a behaviour change.
