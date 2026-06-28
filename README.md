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

TL'DR:

```zsh
curl -sL https://install.doctrinal.systems | sh
```


**Prebuilt binary (macOS, no Rust toolchain) — recommended:**

``` zsh
# latest release (rolling):
curl -fsSL https://raw.githubusercontent.com/davidlee/doctrine/main/install.sh | sh

# or pin to a release tag for reproducibility:
curl -fsSL https://raw.githubusercontent.com/davidlee/doctrine/v0.8.1/install.sh | sh
```

Installs to `~/.local/bin` (override with `DOCTRINE_BIN_DIR`); choose a version
with `DOCTRINE_VERSION`. The script checksum-verifies what it downloads — read it
before piping to a shell. macOS arm64 + x86_64; Linux is a follow-up.

**Or with [`cargo binstall`](https://github.com/cargo-bins/cargo-binstall) (prebuilt, no compile):**

``` zsh
cargo binstall doctrine
```

**Or `cargo install` (compiles from source; needs a Rust toolchain — may hit the
`-liconv` link error on some macOS toolchains, which the prebuilt paths above
sidestep):**

``` zsh
cargo install doctrine
```

Then bootstrap a project:

``` zsh
cd my_project || mkdir my_project

doctrine install                  # prompts to confirm; or use --dry-run | --yes
npx skills add davidlee/doctrine  # or `doctrine install --agent claude` for claude code only

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

or use it as a nix flake:

```nix
inputs.doctrine.url = "github:davidlee/doctrine";
# ...
doctrine = inputs.doctrine.packages.${system}.doctrine;
```

to install skills for other agents:

```zsh
npx skills add davidlee/doctrine # or your fork 
```

## Post-Install Setup 

```zsh

mkdir my-project && cd my-project
git init 
mkdir .claude 

# these are also the 'I updated doctrine' routine:
doctrine install -y 
npx skills add davidlee/doctrine --agent universal -y

cat .gitignore # check & adjust to taste

git add -A && git commit -m "chore: doctrine install"
```

## Memory-only use 

Use Doctrine's memory system with your preferred tooling for the rest:

```zsh
cd my-project
doctrine install --agent claude --only-memory -y 

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
doctrine install --agent claude --yes # from binary, or
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
- [PRD-011 — Graph-Derived Priority and Actionability](.doctrine/spec/product/011/spec-011.md) — `active`
- [PRD-012 — Technical Specifications](.doctrine/spec/product/012/spec-012.md) — `active`
- [PRD-013 — Requirement Reconciliation](.doctrine/spec/product/013/spec-013.md) — `active`
- [PRD-014 — Estimation & Value](.doctrine/spec/product/014/spec-014.md) — `active`
- [PRD-015 — Dispatch & worktree](.doctrine/spec/product/015/spec-015.md) — `active`

### Technical Specifications

- [SPEC-001 — Graph-Derived Priority Engine](.doctrine/spec/tech/001/spec-001.md) — `active`
- [SPEC-002 — Requirement Reconciliation Engine](.doctrine/spec/tech/002/spec-002.md) — `active`
- [SPEC-003 — Doctrine](.doctrine/spec/tech/003/spec-003.md) — `active`
- [SPEC-004 — Entity engine](.doctrine/spec/tech/004/spec-004.md) — `active`
- [SPEC-005 — ADR entity surface](.doctrine/spec/tech/005/spec-005.md) — `active`
- [SPEC-006 — Spec composition machinery](.doctrine/spec/tech/006/spec-006.md) — `active`
- [SPEC-007 — Memory engine](.doctrine/spec/tech/007/spec-007.md) — `active`
- [SPEC-008 — Id lifecycle](.doctrine/spec/tech/008/spec-008.md) — `active`
- [SPEC-009 — Install & distribution](.doctrine/spec/tech/009/spec-009.md) — `active`
- [SPEC-010 — Skills distribution](.doctrine/spec/tech/010/spec-010.md) — `active`
- [SPEC-011 — Boot snapshot](.doctrine/spec/tech/011/spec-011.md) — `active`
- [SPEC-012 — Dispatch & worktree](.doctrine/spec/tech/012/spec-012.md) — `active`
- [SPEC-013 — CLI surface](.doctrine/spec/tech/013/spec-013.md) — `active`
- [SPEC-014 — Slice surface](.doctrine/spec/tech/014/spec-014.md) — `active`
- [SPEC-015 — Backlog entity surface](.doctrine/spec/tech/015/spec-015.md) — `active`
- [SPEC-016 — Governance kinds (POL/STD)](.doctrine/spec/tech/016/spec-016.md) — `active`
- [SPEC-017 — Tech-spec spine](.doctrine/spec/tech/017/spec-017.md) — `active`
- [SPEC-018 — Cross-corpus relation contract](.doctrine/spec/tech/018/spec-018.md) — `active`
- [SPEC-019 — Knowledge-record entity surface](.doctrine/spec/tech/019/spec-019.md) — `active`
- [SPEC-020 — Estimation facet](.doctrine/spec/tech/020/spec-020.md) — `active`
- [SPEC-021 — Dispatch orchestrator process](.doctrine/spec/tech/021/spec-021.md) — `active`
- [SPEC-022 — Git interaction model](.doctrine/spec/tech/022/spec-022.md) — `active`

### Architecture Decision Records

- [ADR-001 — Module layering: leaf ← engine ← command, no cycles](.doctrine/adr/001/adr-001.md) — `accepted`
- [ADR-002 — Global orientation memory class: repo-empty, unanchored, evergreen](.doctrine/adr/002/adr-002.md) — `accepted`
- [ADR-003 — Canonical change loop: slice-first, observe, reconcile, close](.doctrine/adr/003/adr-003.md) — `accepted`
- [ADR-004 — Relations stored outbound-only; reciprocity is derived](.doctrine/adr/004/adr-004.md) — `superseded`
- [ADR-005 — Shipped knowledge is tiered by access pattern; skills route, reference docs explain](.doctrine/adr/005/adr-005.md) — `accepted`
- [ADR-006 — Worktree posture: policy-agnostic framework, orchestrator-sole-writer dispatch](.doctrine/adr/006/adr-006.md) — `accepted`
- [ADR-007 — Adversarial review as a first-class kind with turn-based ledger coordination](.doctrine/adr/007/adr-007.md) — `accepted`
- [ADR-008 — Project-local jail build isolation and worker confinement for parallel dispatch](.doctrine/adr/008/adr-008.md) — `accepted`
- [ADR-009 — Slice lifecycle state machine and conduct axis](.doctrine/adr/009/adr-009.md) — `accepted`
- [ADR-010 — Relation modelling: unify the contract and write seam, keep storage bespoke](.doctrine/adr/010/adr-010.md) — `accepted`
- [ADR-011 — Harness-agnostic orchestrator spawn interface and per-harness capability altitude](.doctrine/adr/011/adr-011.md) — `accepted`
- [ADR-012 — Dispatch integration topology: isolated coordination worktree, class-routed projection, preserved code branches](.doctrine/adr/012/adr-012.md) — `accepted`
- [ADR-013 — Revision as a first-class change-axis kind; governance dependency routes through a Revision](.doctrine/adr/013/adr-013.md) — `accepted`
- [ADR-014 — RFC: governance-neutral first-class kind, precursor to Revision](.doctrine/adr/014/adr-014.md) — `accepted`
- [ADR-015 — Multi-dimensional priority scoring](.doctrine/adr/015/adr-015.md) — `accepted`
- [ADR-016 — Relation intent as a closed role dimension](.doctrine/adr/016/adr-016.md) — `accepted`
- [ADR-017 — Actionability gating via inbound needs on unsettled records](.doctrine/adr/017/adr-017.md) — `accepted`
<!-- END:readme-index -->
