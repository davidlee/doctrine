# SPEC-010: Skills distribution

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

Skills distribution is the container that gets Doctrine's *working conventions*
into the agent in front of the code: how the curated `SKILL.md` set is carried,
catalogued, and laid into an agent's own layout. It sits beneath the whole-system
root (SPEC-003) and is the sibling of install & distribution (SPEC-009) — install
*uses* this container (it ensures the skills gitignore negation and shares the
confirm prompt) but does not own the skill tree.

One canonical source feeds two independent channels. The `plugins/` tree is the
single source-of-truth, embedded into the binary at compile time by a *second*
rust-embed folder parallel to `install/`'s. The published marketplace channel
(`.claude-plugin/marketplace.json`) lets a consumer with no Doctrine binary add
the repo as a Claude Code plugin marketplace; the `doctrine skills` channel
serves a consumer who already holds the binary. No skill is duplicated across
channels — both read the same embedded tree.

What this container owns is the *binary-side* mechanism: catalog discovery from
the embed, the dual-path install plan (Claude direct, every other agent
delegated), the canonical-tree-plus-symlink lay-down with its proven-ownership
never-clobber contract, and the `skills list`/`skills install` surface. It rides
the install container (SPEC-009) for the shared project-root walk, the embedded
`install/` gitignore helper, and the confirm prompt, and restates none of that.

## Responsibilities

Mirrors the structured `responsibilities` list: carry the embedded canonical
source; discover and validate the catalog; plan the dual-path per-agent install;
reconcile each Claude link by proven ownership; materialise the canonical tree
and links atomically; self-enforce the derived-tree gitignore; and surface the
`list`/`install` commands.

### One source, two channels

Skills are grouped into domain plugins under `plugins/<domain>/skills/<skill>/`,
each domain a self-contained Claude Code plugin. The whole `plugins/` tree is
embedded by `#[derive(RustEmbed)] #[folder = "plugins/"]` — a second embed
alongside the install container's `install/` one — so the running binary carries
every skill with no network fetch and no sidecar bundle. The same on-disk tree is
what the marketplace channel and `npx skills` discover when pointed at the repo,
so the two channels never fork or maintain the skill set twice. Because the embed
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

### Dual-path routing

Each target agent takes one of two paths. **Claude is installed directly** — the
binary owns Claude's layout and needs no Node — while **every other agent is
delegated** to `npx skills add davidlee/doctrine`, the universal external
installer (vercel-labs/skills) that already understands ~71 agent layouts and
Claude plugin marketplaces. Doctrine does not reimplement per-agent install
logic; it special-cases only the agent it owns and defers the rest. The
delegate argv is assembled purely (subset `--skill` flags, `--global`, always
`--yes` since the plan is already confirmed) and surfaced verbatim in the plan so
a user can run it by hand. Node absent on the delegate path is a hard error with
guidance, never a silent fallback.

### Canonical tree and proven-ownership links

The direct Claude path does not copy skill dirs into `.claude/skills/`. It
materialises a *derived* canonical tree at `.doctrine/skills/<id>` and points a
**relative** agent symlink (`.claude/skills/<id> → ../../.doctrine/skills/<id>`)
at it. The link target is computed from the two directories, never hard-coded, so
it stays correct under a shared `--global` ($HOME) base.

Reconciliation is by *proven ownership*: a managed link is Doctrine's iff its
value equals the relative target we would write — type alone (is-symlink) is
necessary but not sufficient. This yields a trichotomy: **create** a missing
link, **relink** (heal) a symlink already equal to our target — including a
dangling-but-ours link whose canonical has not yet materialised, since ownership
is the link *value*, not its resolvability — and **keep-foreign** anything else
(a symlink pointing elsewhere, or a real directory the user pinned) untouched.
Keep-foreign is both the never-clobber guarantee and the deliberate override
hatch: a user who replaces a link with a real directory keeps it.

### Atomic lay-down

The canonical tree is *derived* (it owns no authored data), so materialise always
overwrites: it stages the embed into a `.tmp-<id>` sibling, then swaps it in by
remove-then-rename (Unix `rename` cannot replace a non-empty directory, and std
has no `RENAME_EXCHANGE`), so a crash leaves at worst a dangling agent link the
next idempotent install heals — a live link never observes a half-staged tree.
Links are written by stage-then-rename likewise. Ownership is re-classified at
*mutation* time, not just at plan time, to close the plan-to-execute TOCTOU
window: a foreign path that appears at the destination after the plan is built
(the confirm prompt, or a concurrent install) is kept, never clobbered.

### Gitignore self-enforcement

The canonical `.doctrine/skills/*` tree is derived and must be ignored. `skills
install` enforces that ignore itself — independent of whether `doctrine install`
ran first — anchored at the same base the canonical tree is written to, so under
`--global` the entry follows the tree to `$HOME` rather than polluting a project
with an ignore for a tree that is not there.

### The command surface

`skills list` enumerates the catalog grouped by domain with per-agent install
status, read from symlink presence under `.claude/skills/` (a dangling-but-managed
link counts as installed — status uses `symlink_metadata`, never `exists`, which
would follow the link and hide it). `skills install` selects a subset by
`--skill`/`--domain` or the derived `--only-memory` subset, targets one or more
`--agent`s (default: detect `.claude/`, else error — Doctrine does not guess
non-Claude agents), scopes to project or `--global`, and gates on `--dry-run` /
the shared confirm prompt / `--yes`. The pure planner builds an inspectable plan;
the thin shell prints it, prompts, and executes.

## Concerns

- **Never-clobber is the safety contract.** The direct path must never destroy a
  user's pinned skill (a real dir or a foreign symlink); ownership-by-target-value
  re-proven at mutation time is what upholds it across the confirm/concurrency
  window.
- **Delegate version skew (accepted).** Delegated installs pull the repo at
  `HEAD`, not the embedded snapshot in the running binary, so a non-Claude agent
  can receive newer or older skills than the binary carries. v1 tracks `HEAD`;
  pinning to a build ref is backlogged.
- **Embed staleness across recompiles.** Like the install container, the embed is
  fixed at compile time — an edited `SKILL.md` is invisible until the embedding
  crate recompiles; a refresh discipline, not a runtime fault.
- **Multi-agent status fidelity.** Install status is authoritative only for
  Claude (a directory Doctrine owns); for delegated agents it is best-effort and
  may read as not-tracked.

## Hypotheses

- **Delegate the long tail, special-case only what we own.** Reimplementing ~71
  agent layouts is not worth it; delegating every non-Claude agent to the
  universal installer and direct-installing only Claude (whose layout and embed
  Doctrine already owns) is preferred, keeping the default agent working offline
  with no Node.
- **A canonical tree with symlinks beats copies.** Materialising one derived
  `.doctrine/skills/<id>` tree and pointing relative agent links at it is
  preferred over copying skill dirs per agent, so a refresh updates one tree and
  ownership is provable by link value — at the cost of a Unix symlink-swap dance.
- **Idempotent, additive, never-overwrite.** Treating an existing managed link as
  a heal and a foreign path as keep is preferred so re-running changes nothing
  already in place and a user's local override is never lost — install is
  additive in this capability (remove/update are out of scope).

## Decisions

- **D1 — one embedded source, two channels.** `plugins/` is the single
  source-of-truth, embedded by a second rust-embed folder parallel to `install/`;
  the marketplace and the binary both read it, and no skill is duplicated.
- **D2 — Claude direct, all others delegated.** The net routing rule is target ==
  Claude ⇒ direct lay-down; any other agent ⇒ `npx skills add davidlee/doctrine`.
  Doctrine owns only Claude's layout and defers the rest to the universal
  installer.
- **D3 — direct install is a canonical tree plus relative symlinks, reconciled by
  proven ownership.** Claude gets a derived `.doctrine/skills/<id>` tree and a
  relative link into it; a link is ours iff its value equals our computed target,
  yielding create / relink-heal / keep-foreign, re-proven at mutation time.
- **D4 — the canonical tree is derived and self-ignored.** It owns no authored
  data, so materialise always overwrites via a staged atomic swap, and `skills
  install` self-enforces the `.doctrine/skills/*` ignore anchored at the
  tree's base.
- **D5 — marketplace-only subset domains are excluded from the CLI catalog.**
  `doctrine-memory`/`doctrine-partner` are symlink subsets of the canonical
  `doctrine` domain for marketplace publishing; discovery drops them so they
  never collide on a duplicate skill id, while `--only-memory` derives its subset
  from one of them.
