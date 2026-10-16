# REV REV-072 — SL-281 spec residuals

Revision — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-281 landed REV-071 (install converges framework-owned artefacts) at
`/reconcile`. Landing it left text in SPEC-009 and SPEC-010 that REV-071 did not
list but that now contradicts what it landed: SPEC-009 D2, D5 and D7, and the
retitled REQ-167, REQ-170, REQ-172, REQ-176 and REQ-180 (RV-420 F-17). This
revision removes those contradictions and nothing else. Every row is `modify`.
"Structured" sections live in the spec's `.toml` `responsibilities` list; the
rest are prose in the `.md`.

The only registered harness with a doctrine `skills` route is `claude`
(`install/manifest.toml` `[harness.*]`); codex, pi and universal are routed
`skills_via = "universal"`. So "Claude direct" stays true; what changes is the
wording of *why* (route, not agent name) and who owns the tree (install).

## SPEC-009 (primary)

### Responsibility 3 (structured, `spec-009.toml`)

**Before.** "…plus the target directory (default `.doctrine`), the directories
to create even when empty, the `.gitignore` lines to append, and the
project-root markers…"

**After.** "…plus the target directory (default `.doctrine`), the directories
to create even when empty, the runtime-state entries of the managed
`.gitignore` block, the `[framework]` ownership declaration and `[harness.*]`
registry, and the project-root markers…"

### Responsibility 5 (structured, `spec-009.toml`)

**Before.** "Build an inspectable plan (create-dir / install / skip / gitignore
steps) and run it idempotently: files are written only when absent and never
overwritten, directories are created-if-missing, and `.gitignore` lines are
appended only when not already present — so re-install is a safe no-op."

**After.** "Build an inspectable plan — base create-dir / install / skip steps
plus framework-owned write / link / relink / unlink / prune / report actions —
and run it by ownership: user-owned files are written only when absent and never
overwritten, framework-owned artefacts converge and every change is reported,
directories are created-if-missing, and the managed `.gitignore` block is
rewritten — so a second install with the same binary changes nothing."

### D5 heading

**Before.** "**D5 — skills distribution and memory sync are not absorbed.**"

**After.** "**D5 — memory sync is not absorbed; skills are delegated only for
`skills_via` harnesses.**" (Body unchanged; it already says this.)

## SPEC-010

### Overview, first paragraph

**Before.** "…It sits beneath the whole-system root (SPEC-003) and is the
sibling of install & distribution (SPEC-009) — install *uses* this container (it
ensures the skills gitignore negation and shares the confirm prompt) but does
not own the skill tree."

**After.** "…It sits beneath the whole-system root (SPEC-003) and is the
sibling of install & distribution (SPEC-009): install owns the skill canon tree
and its links for harnesses with a `skills` route, and this container specifies
that mechanism."

### Dual-path routing

**Before.** "Each target agent takes one of two paths. **Claude is installed
directly** — the binary owns Claude's layout and needs no Node — while **every
other agent is delegated** to `npx skills add davidlee/doctrine`…"

**After.** "Each target harness takes one of two paths. A harness with a
doctrine `skills` route — today only **Claude** — **is installed directly**: the
binary owns its layout and needs no Node. **Every harness routed `skills_via` is
delegated** (pi and codex via the `universal` target) to `npx skills add
davidlee/doctrine`…" (Rest of the section unchanged.)

### D2

**Before.** "**D2 — Claude direct, all others delegated.** The net routing rule
is target == Claude ⇒ direct lay-down; any other agent ⇒ `npx skills add
davidlee/doctrine`. Doctrine owns only Claude's layout and defers the rest to
the universal installer."

**After.** "**D2 — `skills` route direct, `skills_via` delegated.** The net
routing rule is a harness with a doctrine `skills` route (today only Claude) ⇒
direct lay-down; a harness routed `skills_via` ⇒ `npx skills add
davidlee/doctrine`. Doctrine owns only the `skills`-routed layouts and defers the
rest to the universal installer."

### Responsibility 3 (structured, `spec-010.toml`)

**Before.** "…every other agent delegates to `npx skills add
davidlee/doctrine`, the universal external installer doctrine does not
reimplement."

**After.** "…every harness routed `skills_via` delegates to `npx skills add
davidlee/doctrine`, the universal external installer doctrine does not
reimplement."

### D4 heading

**Before.** "**D4 — the canonical tree is derived and self-ignored.**"

**After.** "**D4 — the canonical tree is derived and covered by the managed
gitignore block.**" (Body already says this since REV-071.)

### Hypotheses

- **"A canonical tree with symlinks beats copies."**
  **Before.** "…so a refresh updates one tree and ownership is provable by link
  value — at the cost of a Unix symlink-swap dance."
  **After.** "…so a refresh updates one tree and ownership is provable by link
  target (inside a framework canon root) — at the cost of a Unix symlink-swap
  dance."
- **"Idempotent, additive, never-overwrite."**
  **Before.** "**Idempotent, additive, never-overwrite.** Treating an existing
  managed link as a heal and a foreign path as keep is preferred so re-running
  changes nothing already in place and a user's local override is never lost —
  install is additive in this capability (remove/update are out of scope)."
  **After.** "**Idempotent, convergent, never-clobber.** Treating an existing
  doctrine link as a relink and a foreign path as keep is preferred so re-running
  changes nothing already in place and a user's local override is never lost; on
  a project install a doctrine link no longer mapped to a shipped member is
  removed and reported (REQ-176)."
