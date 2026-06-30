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


**Prebuilt binary (macOS + Linux, no Rust toolchain) — recommended:**

``` zsh
# latest release (rolling):
curl -fsSL https://raw.githubusercontent.com/davidlee/doctrine/main/install.sh | sh

# or pin to a release tag for reproducibility:
curl -fsSL https://raw.githubusercontent.com/davidlee/doctrine/v0.8.1/install.sh | sh
```

Installs to `~/.local/bin` (override with `DOCTRINE_BIN_DIR`); choose a version
with `DOCTRINE_VERSION`. The script checksum-verifies what it downloads — read it
before piping to a shell. macOS arm64 + x86_64; Linux x86_64 + aarch64 (static
musl — runs on any distro regardless of glibc version).

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

# also run this after a new doctrine version:
doctrine install -y

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


