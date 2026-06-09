# Doctrine

Doctrine is an opinionated but hackable set of tools and conventions for
software engineering with LLM agents.

![HERESIS URITUR; DOCTRINA MANET.](./doctrine.png) 

> Heresy burns; Doctrine remains.

## Design Goals:

1. Correctness 
2. Laziness
3. Hackability
4. Efficiency

- DX for solo developers, teams
- time and token efficiency
- suitability for systems of any size & complexity profile
- useful support for "pre-rational" stages of specification (e.g. product design, backlog)
- quality engineering: robust auditability; formal verification gates
- separation of structured, relational data from prose
- separation of mutable, disposable state from useful artifacts
- thoughfully designed memory retrieval, relevance & decay
- composability; provide "orchestration primitives"
- avoidance of vendor lockin
- single binary distribution
- more with less: focused ambition, not minimalism.

## Non-Goals

- SaaS integration (in core)
- Windows support (for now)
- Integrated TUI (for now)

## Installation

``` zsh
cargo install doctrine

cd my_project || mkdir my_project

doctrine install                  # prompts to confirm; or use --dry-run | --yes
npx skills add davidlee/doctrine  # or `doctrine skills install` for claude code only

doctrine slice new "add killer feature"
```

or install from source (customise templates / skills):

``` zsh
gh repo clone davidlee/doctrine && cd doctrine

# optional: 
# customise install/templates and/or plugins/skills
# they'll get bundled into the binary for installation

cargo install --path .
```

to install skills for other agents:

```zsh
npx skills add davidlee/doctrine # or your fork 
```

## Setup 

```zsh

mkdir my-project && cd my-project
git init 
mkdir .claude 

doctrine install -y 
doctrine skills install # .claude
doctrine skills install --agent universal --yes

doctrine memory sync 

doctrine boot install # .claude/settings.local.json
doctrine boot install --agent codex # works for most 

cat .gitignore # check & adjust to taste

git add -A && git commit -m "chore: doctrine install"
```

## Memory-only use 

Use Doctrine's memory system with your preferred tooling for the rest:

```zsh
cd my-project
doctrine skills install --memory-only -y 

# doctrine memory help
# doctrine memory record --type pattern "red/green/refactor TDD" --glob "src/lib/**/*" --summary "..."
# doctrine memory list
```


## Usage

``` zsh
doctrine slice new "add killer feature"
```

Doctrine ships with self-documenting agent memories. 

The agent should be able to steer while you get used to
the default workflow.

## Hack

templates:

``` zsh
$EDITOR .doctrine/templates
```

skills:

``` zsh
rm .claude/skills/code-review # remove symlink
cp -r .doctrine/skills/code-review/ .claude/skills/

$EDITOR .claude/skills/code-review/SKILL.md
git add -f .claude/skills/code-review/SKILL.md

doctrine install # skips existing non-symlinks
```

or:

``` zsh
gh repo fork davidlee/doctrine --clone
cd doctrine
$EDITOR doctrine/plugins/review
git commit -m "feat: review like a pirate" && git push
cargo install --path .             # build with your edits

# in your projects
doctrine skills install            # from binary, or
npx skills add my-github/doctrine
```

## License

This repository is multi-licensed:

- Rust source code, application code, and compiled binaries are licensed under GPL-3.0-only.
- Files under `plugins/` are licensed under MIT.
- Files under `install/` are licensed under MIT, including templates and `config.toml`.

Where a file contains an SPDX license identifier or a directory contains its own LICENSE file, that more specific notice controls.

## Acknowledgements

The `/worktree` skill's directory-selection and safety-verification patterns are
adapted from [`superpowers:using-git-worktrees`](https://github.com/obra/superpowers)
by Jesse Vincent (MIT).

## Specifications

Product and technical specifications — the durable, agent-readable intent behind
Doctrine's capabilities. Regenerate this list with `just readme-index`.

<!-- BEGIN:readme-index -->
### Product Specifications

- [PRD-001 — Slices](.doctrine/spec/product/001/spec-001.md) — `active`
- [PRD-002 — Specifications](.doctrine/spec/product/002/spec-002.md) — `active`
- [PRD-003 — Skills](.doctrine/spec/product/003/spec-003.md) — `active`
- [PRD-004 — Memory](.doctrine/spec/product/004/spec-004.md) — `active`
- [PRD-005 — Reservation & Leasing](.doctrine/spec/product/005/spec-005.md) — `active`
- [PRD-006 — Install](.doctrine/spec/product/006/spec-006.md) — `active`
- [PRD-007 — Boot & Governance](.doctrine/spec/product/007/spec-007.md) — `active`
- [PRD-008 — ADRs](.doctrine/spec/product/008/spec-008.md) — `active`
- [PRD-009 — Backlog](.doctrine/spec/product/009/spec-009.md) — `active`
- [PRD-010 — Epistemic and Governance Records](.doctrine/spec/product/010/spec-010.md) — `active`
- [PRD-011 — Graph-Derived Backlog Priority](.doctrine/spec/product/011/spec-011.md) — `active`
- [PRD-012 — Technical Specifications](.doctrine/spec/product/012/spec-012.md) — `draft`

### Architecture Decision Records

- [ADR-001 — Module layering: leaf ← engine ← command, no cycles](.doctrine/adr/001/adr-001.md) — `accepted`
- [ADR-002 — Global orientation memory class: repo-empty, unanchored, evergreen](.doctrine/adr/002/adr-002.md) — `accepted`
- [ADR-003 — Canonical change loop: slice-first, observe, reconcile, close](.doctrine/adr/003/adr-003.md) — `accepted`
- [ADR-004 — Relations stored outbound-only; reciprocity is derived](.doctrine/adr/004/adr-004.md) — `accepted`
- [ADR-005 — Shipped knowledge is tiered by access pattern; skills route, reference docs explain](.doctrine/adr/005/adr-005.md) — `accepted`
- [ADR-006 — Worktree posture: policy-agnostic framework, orchestrator-sole-writer dispatch](.doctrine/adr/006/adr-006.md) — `proposed`
- [ADR-007 — Adversarial review as a first-class kind with turn-based ledger coordination](.doctrine/adr/007/adr-007.md) — `proposed`
- [ADR-008 — Project-local jail build isolation and worker confinement for parallel dispatch](.doctrine/adr/008/adr-008.md) — `proposed`
<!-- END:readme-index -->
