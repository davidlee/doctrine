# CHR-048: Release SL-229's consumption hooks to the plugin distribution

Surfaced by the SL-229 audit (RV-306 F-1/F-2). Blocks nothing in the tree —
the authored masters are correct — but until it is done, SL-229's mechanism is
inert everywhere, including in this repo.

## Problem

SL-229 PHASE-03 (`64fcc7ad`) added advisory `/research` hooks to the `slice`,
`design`, `plan` and `phase-plan` skill masters. The commit is not an ancestor
of tag `v0.31.0` (PHASE-01 `73b6c29d` and PHASE-02 `14a9f9f8` are), and is not
on `origin/main`, which is what `.claude-plugin/marketplace.json` sources
(`repo: davidlee/doctrine`, `ref: main`, version `0.31.0`).

Measured in the live cache at
`~/.claude/plugins/cache/doctrine/doctrine/0.31.0/skills/`:

- `research/SKILL.md` — **present** (PHASE-02 shipped inside `v0.31.0`).
- `slice` / `design` / `plan` / `phase-plan` SKILL.md — **zero** matches for
  `slice research` or `/research`.

So the skill is installed and nothing points at it. No agent is told to run the
pre-design round, which is the entire behavioural product of SL-229.

## Consequences

- PHASE-03 EX-2 is unsatisfied under `notes.md` D-a's contract reading (the
  harness-visible copy matches its master).
- SL-229's design VH — "one further real slice driven through the round,
  observations to RFC-011 case-notes" — cannot be gathered by the shipped
  mechanism, only by a human deliberately invoking `/research`. RV-306 F-2.
- R1 ("advisory hooks may under-deliver without enforcement") is untestable
  until this lands, so the RFC-011 eval that was to judge whether harder gating
  is needed cannot run.

## Steps

Must run on the **host** — `just release-check` includes a hermetic nix flake
build and nix is absent in the jail.

1. Promote and push: `git fetch . edge:main`, then push `main` to `origin`.
2. `just release <bump>` — bumps the version, runs the pre-release gate, commits
   and tags. Note `just sync-plugin-versions` keeps `plugin.json` in step; see
   CHR-045 (bump the plugin version when the skill set changes), which this
   release also exercises since PHASE-02 added a new skill.
3. `claude plugin update` locally, then re-verify: the four cached hook files
   should each match their `plugins/doctrine/skills/` master.
4. Record the observation to `.doctrine/rfc/011/case-notes.md` and, once a
   subsequent slice runs the round, close out SL-229's VH there.

## Wider point

Post-SL-227 there is no verb answering "is this authored asset live in a
harness?". Confirming it during the SL-229 audit took five probes across four
surfaces (`git merge-base --is-ancestor` vs the tag, `git branch --contains`,
`marketplace.json`, the cache dir, `library show`). For any slice whose product
*is* shipped prose, that question is the audit. Worth its own item if it recurs.
