# SL-241 PHASE-04 step 0 — independent enumeration (PRE-CLASSIFICATION)

**This file is the falsifier's first half. It is authored BEFORE any consultation
of CPT-001 and is not to be edited after that consultation** — the classification
and the residue go in a separate section appended below, under its own heading,
so the diff shows the enumeration predates the taxonomy read (EX-3, VA-1).

## Provenance of this pass (EX-9, independence on both sides)

Authored by a fresh context whose only prior reads were `handover.md` (authored
to name no trigger and list no hazard), `plan.toml` PHASE-04, and ASM-007.

- **CPT-001 NOT read.** ASM-007 quotes the five class *labels* (1 explicit
  execution · 2 build-system evaluation · 3 toolchain auto-load, 3g git-level ·
  4 path-shaped data · 5 resource shape) but not their trigger contents. Stated
  as a known, bounded leak rather than claimed clean: knowing five bucket names
  creates some pull toward enumerating only what fits them. Mitigated by method
  (below) — the walk is driven by ecosystem surface, not by the buckets.
- **The light fixture's `interpret:` list NOT read** (`fixtures.md` not opened).
- **`notes.md` F-P01-2 NOT read** (the five hazards withheld from the PHASE-01
  declaration for this reason).
- The seed list in EX-1 / ASM-007's prose (`preinstall`/`postinstall`/`prepare`,
  `.npmrc`, `.nvmrc`, `node_modules/.bin`, husky, `tsconfig extends`,
  config-as-JS, `type`/`exports`, workspace protocol) WAS read — it is the
  criterion text and unavoidable. It is a floor, not a ceiling; items matching it
  are marked `[seed]` so the residue can be assessed against what this pass added
  independently.

## Method

Not "list the ways code runs" — that recalls the seed. Instead: walk the
npm/TypeScript ecosystem **by surface**, and at each surface ask one question.

> If an untrusted party authored the content of this surface, what causes code to
> execute, or a decision to be taken, on the **trusted** side?

Surfaces walked, in order: manifest fields · package-manager config and state ·
runtime auto-load · compiler config · build/test config · editor & agent
integration · git · environment and shell · CI and registry · resource shape ·
data-shaped-as-code. Each entry names its **mechanism** (what does the executing,
and when), because the mechanism is what a class has to describe. No entry is
classified here.

---

## A. `package.json` manifest fields

| id | trigger | mechanism |
|---|---|---|
| A1 | `scripts.preinstall` / `install` / `postinstall` | npm/pnpm/yarn exec on install, **for every dependency in the graph**, not just the root [seed] |
| A2 | `scripts.prepare` / `prepublish` / `prepack` / `postpack` | run on install-from-git-specifier and on pack [seed] |
| A3 | `scripts.pre<x>` / `post<x>` for **arbitrary** `x` | implicit wrappers: adding `pretest` hijacks a *trusted* `npm test` invocation. The trusted command is unchanged; its preamble is not |
| A4 | dependency `bin` field | install writes a shim into `node_modules/.bin`; shadowing is **by name**, so a dep can capture `tsc`, `eslint`, `jest` [seed-adjacent] |
| A5 | `main` / `module` / `browser` / `exports` / `imports` | resolution redirection; `exports` **conditions** (`require`/`import`/`node`/`development`) serve different files to different consumers, so audited-path ≠ executed-path [seed] |
| A6 | `type: "module"` | reinterprets `.js` as ESM, changing which loader/transform executes [seed] |
| A7 | non-registry dependency specifiers: `git+ssh://`, `github:u/r`, `file:../`, `link:`, `https://…tgz`, `npm:alias@ver` | install fetches and executes arbitrary remote content; git specifiers additionally run `prepare`. The **alias** form makes the manifest name lie about the package installed |
| A8 | `overrides` / `resolutions` / `pnpm.overrides` | substitutes a different package for a *transitive* dep — no line in the dep tree names it |
| A9 | `workspaces` **globs** | a newly added directory matching the glob is auto-enrolled; its `scripts` and `bin` join the trusted graph with no manifest edit [seed names workspace links, not glob auto-enrolment] |
| A10 | `packageManager` field | Corepack downloads and executes the named package-manager version — untrusted content chooses the *tool binary* |
| A11 | `engines` + `.nvmrc` / `.node-version` | selects the interpreter [seed: `.nvmrc`] |
| A12 | `files` / `.npmignore` | publish-time inclusion decision — governs exfiltration surface, not execution |
| A13 | `config` block | surfaced to scripts as `npm_package_config_*` env |
| A14 | `pnpm.onlyBuiltDependencies` / `pnpm.patchedDependencies` | the script allowlist itself, and patch files applied to dependency source |

## B. Package-manager config and state

| id | trigger | mechanism |
|---|---|---|
| B1 | `.npmrc` (project + user) | `registry=` / `@scope:registry=` redirect *where code comes from*; `//host/:_authToken=` and `_auth` are credentials; `script-shell=` picks the interpreter for every script; `ignore-scripts=false` re-arms A1 [seed] |
| B2 | `.yarnrc.yml` `yarnPath` | **executes a package-manager bundle checked into the repo** — the tool is repo content |
| B3 | `.yarnrc.yml` `plugins:` | arbitrary JS loaded *into* the package manager process |
| B4 | `.pnpmfile.cjs` / `pnpmfile.js` | **arbitrary JS executed by pnpm during resolution** — `readPackage` / `afterAllResolved` hooks. Runs even under `--ignore-scripts` |
| B5 | lockfile (`package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`) | pins resolved **URLs** + integrity; a tampered lockfile redirects fetch to an attacker host while the manifest reads clean |
| B6 | `.pnp.cjs` (Yarn PnP) | a require-hook loaded at Node startup that rewrites **all** module resolution |
| B7 | `.yarn/releases/*.cjs`, `.yarn/plugins/*.cjs` | the artifacts B2/B3 point at |
| B8 | `node_modules/.bin` on `PATH` | shim directory prepended by the package manager for script execution [seed] |
| B9 | Corepack env (`COREPACK_*`, `COREPACK_ENABLE_*`) | governs whether A10 is armed |

## C. Node runtime auto-load

| id | trigger | mechanism |
|---|---|---|
| C1 | `NODE_OPTIONS` carrying `--require` / `--import` / `--experimental-loader` | injected from `.env`, direnv, CI config, or a `scripts` entry; applies to **every** subsequent `node` in the session |
| C2 | registered loader hooks (`register()`, `--experimental-loader`) | a package interposes on all `import` resolution and source text |
| C3 | `NODE_PATH` | resolution root injection |
| C4 | `node --env-file`, Node's built-in `.env` support, `-r dotenv/config` | file content becomes process env, feeding C1 |
| C5 | `node --run` (Node 22+) | runs `package.json` scripts natively, outside the package manager's allowlists |
| C6 | **top-level side effects on `import`/`require`** | merely importing executes module top-level code. The base case that makes every resolution-redirect trigger (A5, B6, C3) load-bearing |
| C7 | `binding.gyp` / `gypfile` | **a build script** — `node-gyp` compiles and links at install |
| C8 | `node-pre-gyp` / `prebuild-install` | downloads and executes a prebuilt binary at install |
| C9 | `.node` native addon | `dlopen` of an untrusted binary, outside any JS sandbox |
| C10 | `NODE_V8_COVERAGE`, `NODE_REPL_EXTERNAL_MODULE` | write-path redirection and REPL preload |

## D. TypeScript / compiler

| id | trigger | mechanism |
|---|---|---|
| D1 | `tsconfig.json` `extends` | resolves **through `node_modules`**, so a dependency supplies compiler config [seed] |
| D2 | `compilerOptions.plugins` | **language-service plugins are JS executed by tsserver** — i.e. by the editor, on file open, with no build invoked |
| D3 | `types` / `typeRoots` | auto-includes `@types/*` declarations by **directory presence**, with no import statement anywhere |
| D4 | `/// <reference types="…" />` in a `.d.ts` | transitively pulls further files into the program |
| D5 | `paths` / `baseUrl` | resolution redirection at compile time |
| D6 | `ts-node` / `swc` / `babel` config blocks (incl. `ts-node.require`) | arbitrary JS in the transform pipeline |
| D7 | `.d.ts` content itself | not executed, but **trusted for typechecking decisions** — influences what compiles and what a downstream generator emits. Agency without execution |

## E. Build / test tooling — config-as-code

| id | trigger | mechanism |
|---|---|---|
| E1 | `jest.config.js`, `vite.config.ts`, `webpack.config.js`, `rollup.config.mjs`, `eslint.config.js`, `.babelrc.js`, `postcss.config.js`, `tailwind.config.js`, `playwright.config.ts` | **executed as JS at tool startup** [seed: config-as-JS] |
| E2 | `.eslintrc` `extends` / `plugins` / `parser` / `processor` | resolves and **requires** JS from `node_modules` — so *linting* untrusted content executes the config's plugins |
| E3 | Jest `setupFiles` / `globalSetup` / `transform` / `testEnvironment` / `reporters` / `moduleNameMapper` | each names a JS module the runner executes |
| E4 | Vite/Vitest `plugins`, and `vite.config` being bundled by esbuild before evaluation | build tool executes config to learn how to build |
| E5 | `.mocharc.yml` `require:`, `nyc` / `c8` config | preload modules |
| E6 | husky (`.husky/*` + `prepare: husky install`) | sets `core.hooksPath` → **git hooks come from repo content** [seed: husky] |
| E7 | `lint-staged` config | maps globs to **shell commands**, run from a hook |
| E8 | `simple-git-hooks` (`.simple-git-hooks.json`), `commitlint` | same shape as E6/E7 |
| E9 | `Makefile` / `justfile` / `Taskfile.yml` present in-tree | invoked by convention by a human or CI; recipe content is repo content |
| E10 | `patch-package` / `pnpm patch` patch files | rewrite dependency **source** at install, after any audit of that dependency |
| E11 | **test files discovered by glob** | adding a file that matches the runner's pattern adds executed code with no config edit |
| E12 | Jest snapshot files (`__snapshots__/*.snap`) | **JS modules `require`d by the runner** — a data-looking artifact that executes |
| E13 | framework config (`next.config.js`, `nuxt.config.ts`, `astro.config.mjs`, `svelte.config.js`, `gatsby-node.js`, `.storybook/main.ts`, `instrumentation.ts`) | executed by the framework's CLI on dev/build |

## F. Editor, IDE, and agent integration — the trusted side is a workstation

| id | trigger | mechanism |
|---|---|---|
| F1 | `.vscode/settings.json` | `typescript.tsdk` **loads a TypeScript compiler from the repo**; `eslint.nodePath`; `terminal.integrated.env.*` injects env into every terminal |
| F2 | `.vscode/tasks.json` with `runOptions.runOn: folderOpen` | **executes a command on folder open** — no build, no test, no install |
| F3 | `.vscode/launch.json` `preLaunchTask` | executes on debug start |
| F4 | `.devcontainer/devcontainer.json` | `initializeCommand` runs **on the host**, `postCreateCommand`/`postAttachCommand` in-container; `features` and `image` fetch remote content |
| F5 | tsserver auto-loading `typescript` from `node_modules` | **the editor executes the repo's own compiler build** merely by opening a `.ts` file |
| F6 | `jsconfig.json`, `.idea/` run configurations | same shape as D1/F3 for other editors |
| F7 | `AGENTS.md` / `CLAUDE.md` / `.cursorrules` / `.github/copilot-instructions.md` | **prompt injection** — untrusted repo content acquires agency over an LLM agent that has tool access on the trusted side. Instruction-shaped data, executed by a model |
| F8 | `.mcp.json` / editor MCP config | launches long-lived server **processes** on workspace open |
| F9 | `.editorconfig` | decision-taking only, no execution |

## G. Git-level

| id | trigger | mechanism |
|---|---|---|
| G1 | `.gitmodules` + gitlink entries (mode `160000`) | recursive fetch/checkout of a remote the tree names |
| G2 | `.gitattributes` `filter=` (clean/smudge), `diff=`, `merge=` driver | **runs a command on checkout/diff/merge**; the driver body comes from config, but the *attachment* is tree content |
| G3 | `.gitattributes filter=lfs` | network fetch on checkout |
| G4 | `core.hooksPath` / `.git/hooks/*` | executed by ordinary git verbs |
| G5 | an imported tree's `.git/config` | `alias.* = !sh -c …`, `core.fsmonitor`, `core.editor`, `core.pager`, `core.sshCommand`, `credential.helper`, `url.*.insteadOf` (silently rewrites remotes) |
| G6 | symlinks escaping the worktree; `..` traversal in archive entries (zip-slip) | write outside the intended root during extraction/checkout |
| G7 | `.git` shadowing via case-insensitive FS (`.GIT`), unicode normalisation, NTFS 8.3 (`git~1`), alternate data streams | classic checkout-writes-into-`.git` bypass family |
| G8 | ref names / `packed-refs` shape | a ref name that is a path traversal |
| G9 | `.gitignore` / `.mailmap` | decision-taking, negligible agency |

## H. Environment and shell

| id | trigger | mechanism |
|---|---|---|
| H1 | `.envrc` (direnv) | **arbitrary shell on `cd`** |
| H2 | `.env` / `.env.local` | consumed by Vite/Next/Node; `VITE_*` values are **inlined into the bundle**, so data becomes shipped code |
| H3 | `flake.nix` / `shell.nix` / `default.nix` | evaluation fetches remote inputs; IFD executes builders |
| H4 | `.tool-versions` (asdf), `.mise.toml` | mise `[tasks]` and `[env]` `_.source` execute arbitrary shell on directory entry |
| H5 | `Dockerfile` / `docker-compose.yml` | build executes `RUN`; compose mounts and entrypoints |
| H6 | repo content writing to `$HOME` dotfiles | persistence beyond the worktree — the next shell is the executor |
| H7 | `PATH` containing `.` or a repo-relative directory | name shadowing of ordinary commands |
| H8 | **terminal escape sequences** in file content or command output | OSC 52 clipboard write; sequences that some terminals honour as input injection. The *output channel* itself is a trigger |

## I. CI and supply chain

| id | trigger | mechanism |
|---|---|---|
| I1 | `.github/workflows/*.yml` `pull_request_target` + checkout of PR head | untrusted code executed **with secrets** |
| I2 | `${{ … }}` interpolation into a `run:` block | script injection via attacker-controlled branch name / PR title / commit message |
| I3 | `.github/actions/*/action.yml`, `Jenkinsfile` (Groovy, executed), `.gitlab-ci.yml`, `azure-pipelines.yml` | pipeline definition is executable content |
| I4 | Renovate `postUpgradeTasks` / `allowedPostUpgradeCommands` | bot executes repo-declared commands |
| I5 | scope→registry mapping order (B1) | dependency confusion / typosquat resolution |

## J. Resource shape

| id | trigger | mechanism |
|---|---|---|
| J1 | archive bombs, deeply nested `node_modules`, symlink cycles | unbounded expansion during install/extract |
| J2 | pathological regex in config a trusted tool consumes | ReDoS in the trusted process |
| J3 | very large lockfile / very many workspace packages | resolver time/memory blowup |
| J4 | path depth or filename length exceeding a limit mid-write | partial state, and divergent behaviour per platform |
| J5 | output volume — a script emitting GBs to stdout | fills the **trusted** side's log/disk, not the sandbox's |

## K. Data-shaped-as-code

| id | trigger | mechanism |
|---|---|---|
| K1 | JSON containing `__proto__` | **prototype pollution** in a trusted config parser — later behaviour of unrelated trusted code changes. Agency with no exec surface anywhere |
| K2 | YAML tags (`!!js/function`, `!!python/object`) | unsafe loader instantiates/executes on parse |
| K3 | `//# sourceMappingURL=` | debugger/tool fetches a remote or `data:` URL, or reads an attacker-chosen path |
| K4 | HTML/SVG fixtures rendered by a browser-backed test harness | real browser context executes them |

---

## Count

96 enumerated triggers across 11 surfaces. Of these, **11 match the EX-1 /
ASM-007 seed** (A1, A2, A4, A5, A6, A9-partial, A11, B1, B8, D1, E1, E6); the
remaining ~85 are this pass's own, though many are near neighbours of a seed
entry rather than independent discoveries.

## NOT YET DONE — do not read this file as complete

Classification against CPT-001, the residue, and the ASM-007 amendment are
**deliberately absent**. They are appended below this line in a later commit,
after CPT-001 is read (EX-1, EX-2). The gap between the two commits is the
evidence that this pass was independent (VA-1).

<!-- CLASSIFICATION APPENDED BELOW THIS LINE IN A LATER COMMIT -->
