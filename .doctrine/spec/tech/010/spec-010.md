# SPEC-010: Skills distribution

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

Skills distribution is the container that gets Doctrine's *working conventions*
into the agent in front of the code: how the curated `SKILL.md` set is carried,
catalogued, and laid into an agent's own layout. It sits beneath the whole-system
root (SPEC-003) and is the sibling of install & distribution (SPEC-009): install
owns the skill canon tree and its links for harnesses with a `skills` route, and
this container specifies that mechanism.

One canonical source feeds two independent channels. The `plugins/` tree is the
single source-of-truth, embedded into the binary at compile time by a *second*
rust-embed folder parallel to `install/`'s. The published marketplace channel
(`.claude-plugin/marketplace.json`) lets a consumer with no Doctrine binary add
the repo as a Claude Code plugin marketplace; the `doctrine claude` channel
serves a consumer who already holds the binary. No skill is duplicated across
channels — both read the same embedded tree.

What this container owns is the *binary-side* mechanism: catalog discovery from
the embed, the per-harness install plan (every registered harness with a
`skills` route links the embedded catalog — Claude at `.claude/skills`, codex,
pi and universal at `.agents/skills`), the canonical-tree-plus-symlink lay-down with its proven-ownership never-clobber
contract, and the `claude install` surface. It rides
the install container (SPEC-009) for the shared project-root walk, the embedded
`install/` gitignore helper, and the confirm prompt, and restates none of that.

## Responsibilities

Mirrors the structured `responsibilities` list: carry the embedded canonical
source; discover and validate the catalog; plan the per-harness install from
the embed; reconcile each registered harness link by proven ownership; materialise the canonical tree
and links atomically; cover the derived tree with the project's managed
gitignore block; and surface the `install` command.

### One source, two channels

Skills are grouped into domain plugins under `plugins/<domain>/skills/<skill>/`,
each domain a self-contained Claude Code plugin. The whole `plugins/` tree is
embedded by `#[derive(RustEmbed)] #[folder = "plugins/"]` — a second embed
alongside the install container's `install/` one — so the running binary carries
every skill with no network fetch and no sidecar bundle. The same on-disk tree is
what the marketplace channel discovers when pointed at the repo (third-party
installers can read it too; doctrine's install path uses none of them), so the
two channels never fork or maintain the skill set twice. Because the embed
is fixed at compile time, an edited `SKILL.md` is invisible until the embedding
crate recompiles — the refresh discipline the install container documents applies
here unchanged.

### Catalog discovery

Discovery groups embedded asset paths by the `<domain>/skills/<skill>/` shape into
catalog entries, reading only `name` and `description` from each `SKILL.md` YAML
frontmatter — everything else is opaque payload. Two integrity rules hold at
discovery: a skill's directory name must equal its frontmatter `name`, and skill
ids are globally unique across domains (a duplicate id is a hard error, since
ids flatten into one agent skills directory). The *marketplace-only subset
domains* — `doctrine-memory` and `doctrine-partner` — are excluded from the CLI
catalog: their skills are symlinks back into the canonical `doctrine` domain, so
the embed carries them as path duplicates that would collide on id. They exist
only to publish a standalone subset on the marketplace channel.

### Registry routing

Every registered harness with a `skills` route (the `[harness.*]` registry,
SPEC-009 `D4`) is installed directly from the embedded catalog: the binary owns
the layout and needs no Node, network or external runner. The route names the
harness's link site — `.claude/skills` for Claude; `.agents/skills` for codex,
pi and universal, which share that directory, so selecting several of them
yields one link per skill. An explicit `--agent` absent from the registry is
refused before any write, naming the registered harnesses and suggesting
`universal` for another harness that reads `.agents/skills`. `--only-memory`
subsets every selected harness to the memory pair.

### Canonical tree and proven-ownership links

Install does not copy skill dirs into a harness's link site. It materialises a
*derived* canonical tree at `.doctrine/skills/<id>` and points a **relative**
symlink at it from each selected link site
(`.claude/skills/<id> → ../../.doctrine/skills/<id>`, and likewise
`.agents/skills/<id>`). The link target is computed from the two directories, never hard-coded, so
it stays correct under a shared `--global` ($HOME) base.

Reconciliation is by *proven ownership*: a link is Doctrine's iff it is a
symlink whose target lies inside a framework canon root — a relative value joined
to its link dir and normalised lexically, an absolute value compared
component-wise under the base. Ours and mapped to a shipped member: relinked to
the current layout (heals stale layouts). Ours and unmapped: unlinked on a
project install, reported on a global one. Anything else: kept foreign.
Keep-foreign is both the never-clobber guarantee and the deliberate override
hatch: a user who replaces a link with a real directory keeps it.

### Atomic lay-down

The canonical tree is *derived* (it owns no authored data), so materialise always
overwrites: it stages the embed into a unique `.tmp-<name>.<pid>.<n>` staging
entry made by exclusive create (staging entries are never pruned), then swaps it
in by remove-then-rename (Unix `rename` cannot replace a non-empty directory, and
std has no `RENAME_EXCHANGE`), so a crash leaves at worst a dangling agent link the
next idempotent install heals — a live link never observes a half-staged tree.
Links are written by stage-then-rename likewise. Ownership is re-classified at
*mutation* time, not just at plan time, to close the plan-to-execute TOCTOU
window: a foreign path that appears at the destination after the plan is built
(the confirm prompt, or a concurrent install) is kept, never clobbered.

### Gitignore coverage

The canonical `.doctrine/skills/*` tree is derived and must be ignored. The
project's managed gitignore block carries `.doctrine/skills/*`; a global install
writes no ignore.

### The command surface

Catalog enumeration is internal — there is no public list verb. `claude install`
selects a subset by `--skill`/`--domain` or the derived `--only-memory` subset,
targets one or more registered `--agent`s (default: each registered harness
whose `detect` directory is present), scopes to project or `--global`, and gates on
`--dry-run` / the shared confirm prompt / `--yes`. The pure planner builds an
inspectable plan; the thin shell prints it, prompts, and executes.

## Concerns

- **Never-clobber is the safety contract.** The direct path must never destroy a
  user's pinned skill (a real dir or a foreign symlink); ownership by
  target-inside-a-framework-root re-proven at mutation time is what upholds it
  across the confirm/concurrency window.
- **Embed staleness across recompiles.** Like the install container, the embed is
  fixed at compile time — an edited `SKILL.md` is invisible until the embedding
  crate recompiles; a refresh discipline, not a runtime fault.
- **`universal --global` destination differs from npx (accepted).** A global
  universal install links under `$HOME/.agents/skills`, not npx's
  `~/.config/agents/skills`; doctrine models no per-scope route override.

## Hypotheses

- **Register the supported harnesses, share the embedded canon.** Most
  non-Claude harnesses read `.agents/skills`, so a registry entry with a declared
  link site covers them from the embed the binary already carries — no external
  installer, no version skew against the running binary, offline, no Node. A new
  harness is a registry entry; an unknown name is refused.
- **A canonical tree with symlinks beats copies.** Materialising one derived
  `.doctrine/skills/<id>` tree and pointing relative agent links at it is
  preferred over copying skill dirs per agent, so a refresh updates one tree and
  ownership is provable by link target (inside a framework canon root) — at the
  cost of a Unix symlink-swap dance.
- **Idempotent, convergent, never-clobber.** Treating an existing doctrine link as
  a relink and a foreign path as keep is preferred so re-running changes nothing
  already in place and a user's local override is never lost; on a project install
  a doctrine link no longer mapped to a shipped member is removed and reported
  (REQ-176).

## Decisions

- **D1 — one embedded source, two channels.** `plugins/` is the single
  source-of-truth, embedded by a second rust-embed folder parallel to `install/`;
  the marketplace and the binary both read it, and no skill is duplicated.
- **D2 — registered `skills` routes link the embedded catalog.** Every
  registered harness with a `skills` route gets the canonical tree and links at
  its declared site; there is no runner, delegated argv or Node fallback. An
  explicit unknown `--agent` is refused before any write with the `universal`
  suggestion, and `--only-memory` selects the memory pair.
- **D3 — direct install is a canonical tree plus relative symlinks, reconciled by
  proven ownership.** The selected harnesses share one derived
  `.doctrine/skills/<id>` tree, each with a relative link into it from its link
  site; a link is ours iff it is a symlink whose target lies inside a framework
  canon root — ours and mapped to a shipped member is relinked
  to the current layout, ours and unmapped is unlinked on a project install and
  reported on a global one, anything else is kept foreign — re-proven at mutation
  time.
- **D4 — the canonical tree is derived and covered by the managed gitignore
  block.** It owns no authored
  data, so materialise always overwrites via a staged atomic swap, and it is
  covered by the managed gitignore block.
- **D5 — marketplace-only subset domains are excluded from the CLI catalog.**
  `doctrine-memory`/`doctrine-partner` are symlink subsets of the canonical
  `doctrine` domain for marketplace publishing; discovery drops them so they
  never collide on a duplicate skill id, while `--only-memory` derives its subset
  from one of them.
