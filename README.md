# Doctrine

Doctrine is an opinionated but hackable set of tools and conventions for
software engineering with LLM agents.

![HERESIS URITUR; DOCTRINA MANET.](./doctrine.png) 

## Design Goals:

1. Correctness 
2. Autonomy
3. Adaptability
3. Efficiency
4. Simplicity

## License

This repository is multi-licensed:

- Rust source code, application code, and compiled binaries are licensed under GPL-3.0-only.
- Files under `plugins/` are licensed under MIT.
- Files under `install/` are licensed under MIT, including templates and `config.toml`.

Where a file contains an SPDX license identifier or a directory contains its own LICENSE file, that more specific notice controls.

## Installation

``` zsh
cargo install doctrine

cd my_project || mkdir my_project

doctrine install                  # prompts to confirm; or use --dry-run | --yes
npx skills add davidlee/doctrine  # or `doctrine skills install` for claude code only

doctrine slice new "add killer feature"
```

or from source:
``` zsh
gh repo clone davidlee/doctrine && cd doctrine
cargo install --path .
```

## Usage

``` zsh
doctrine slice new "add killer feature"
```

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
