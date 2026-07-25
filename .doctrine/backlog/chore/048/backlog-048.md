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

## Outcome (closed 2026-07-25 · done)

Released as **v0.31.1**. Verified in the live cache: all five skills under
`~/.claude/plugins/cache/doctrine/doctrine/0.31.1/skills/` are byte-identical to
their `plugins/doctrine/skills/` masters — `slice`/`plan`/`phase-plan` one hook
each, `design` two, `research` two. `main == edge`, tag `v0.31.1` contains
PHASE-03 `64fcc7ad6`. RV-306 F-1/F-2 discharged; PHASE-03 EX-2 satisfied under
`notes.md` D-a's contract reading. Recorded to `.doctrine/rfc/011/case-notes.md`
(`ace4637cb`).

Two corrections to this card, for anyone reading it as precedent:

- The stated host-only blocker was wrong. `just nix-build` **soft-skips** when
  nix is off PATH (`justfile:141-146` — stderr note, exit 0), so `just release`
  would have *succeeded* in the jail with the hermetic build silently elided. The
  real hard blocker is the jail's disabled git SSH shim, which makes `git push`
  impossible. A soft gate mistaken for a hard gate is the shape that ships a
  hollow binary (AGENTS.md, crane embed-strip).
- Local `main` already contained PHASE-03; only `origin/main` lagged. And
  PHASE-02's new `research` skill had already shipped inside v0.31.0, so this
  release was prose-only — hence `patch`.

The **cache key is the version directory** and the marketplace sources
`ref: main`, so a bump is load-bearing for invalidation; pushing main without one
leaves agents on the stale dir. The cache tracks main's *tip*, not the tag.

### Decisions

- **No liveness verb.** The "Wider point" above is declined (2026-07-25): a
  `doctrine`-side verb answering "is this asset live in a harness?" would be
  modelling one harness's cache layout, which is not doctrine's business. Do not
  re-raise as a backlog item.
- **CHR-045 narrowed and tagged `yagni`.** `sync-plugin-versions` structurally
  fixed the manifest drift; the residual (nothing forces a bump when the skill
  set changes) is deliberately not built. See CHR-045.

### Residual, not tracked here

SL-229's design VH — one further real slice driven through the round, with
observations to RFC-011 case-notes — is now *possible* but not *done*, and R1
("advisory hooks may under-deliver without enforcement") becomes testable at the
next `/slice`. SL-229 is closed, so the case-notes entry is that obligation's
only live home. This card is closed on its own scope: the hooks are shipped.
