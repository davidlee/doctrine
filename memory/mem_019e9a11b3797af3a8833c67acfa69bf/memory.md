# Doctrine overview

Doctrine governs intentional change in a repo. Four pillars over a thin Rust
shell (the `doctrine` CLI):

- **Slice lifecycle** — every change is a *slice*: scope → design → plan →
  phased execution → audit → close. See [[signpost.doctrine.lifecycle-start]].
- **Governance** — the rules of the road live in `.doctrine/state/boot.md` (the
  boot snapshot, `@`-imported into `CLAUDE.md`). It carries the routing table,
  the core process, and the guardrails.
- **Memory** — durable, scoped knowledge you query instead of rediscover; this
  corpus is itself shipped memory. See [[concept.doctrine.memory-model]].
- **Entity engine** — slices, ADRs, specs, and memories are authored entities
  over one engine. See [[concept.doctrine.entity-engine]].

**Start here, every substantive task:** run `/route`. It picks the governing
skill *before* you read files, run commands, or write code — don't improvise
past it. The boot snapshot (`.doctrine/state/boot.md`) is the authority: read
it, don't trust a remembered version.

Orient further:
- [[signpost.doctrine.file-map]] — where everything lives.
- [[signpost.doctrine.skill-map]] — which skill governs which situation.
- [[signpost.doctrine.cli-command-map]] — the CLI verb surface.
- [[concept.doctrine.storage-model]] — authored vs runtime vs derived.

The CLI is the source of truth for command shapes — `doctrine --help`, never
guess. See [[fact.doctrine.cli-source-of-truth]].
