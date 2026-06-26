Here's the memory schema — taken from the authoritative `memory-spec.md` and CLI surfaces.

## Memory types (6)

| Type | Purpose | Lifespan |
|---|---|---|
| `concept` | Stable mental model, terminology, taxonomy | Long |
| `fact` | Atomic checkable truth (invariant, default, limit) | Long |
| `pattern` | Repeatable recipe / command sequence / workflow | Medium |
| `signpost` | Navigation pointer set — "start here" for a domain | Long |
| `system` | Subsystem map + architecture pointers | Medium |
| `thread` | Short-lived working set for one task/slice | Days–weeks |

Rule: use the narrowest type that fits. A durable thread gets **promoted** to `fact`/`pattern`/`system`, not left to linger.

## Lifecycle statuses (6)

`active | draft | superseded | retracted | archived | quarantined`

| Status | Default retrieval |
|---|---|
| `active` | included |
| `draft` | excluded (unless `--include-draft`) |
| `superseded` | excluded; visible when tracing |
| `retracted` | excluded; audit-visible |
| `archived` | excluded; historical |
| `quarantined` | excluded from agent context; review-only (security) |

**Status is separate from review state** — a memory can be `active` + `stale` + `unverified` simultaneously.

## Verification states

`unverified | verified | stale | disputed` — a separate axis from lifecycle (`[review].verification_state`).

## Trust levels

`low | medium | high` — combined with severity for the **trust holdback**: low-trust ∧ high-severity memories are suppressed from `retrieve` (but visible via `find`/`show`).

## Severity

`critical | high | medium | low | none`

## Lifespan classification

`semantic | episodic | procedural | working | identity`

- `identity` returns everything
- `semantic` filters out `episodic`/`working`
- `procedural` excludes `working`

Used as a filter on `find`/`retrieve` to suppress transient noise.

## TOML schema (`memory.toml`)

```toml
memory_uid = "mem_018f3a..."         # client-minted UUID; stable forever
memory_key = "mem.pattern.cli.skinny" # optional human slug
schema_version = 1
memory_type = "pattern"              # concept|fact|pattern|signpost|system|thread
status = "active"                    # active|draft|superseded|retracted|archived|quarantined
title = "…"
summary = "…"                        # one line
created = "2026-06-04"
updated = "2026-06-04"

[scope]                              # the retrieval key
paths    = ["src/main.rs"]           # exact/prefix — strongest match (weight 3)
globs    = ["src/**/*.rs"]           # glob match (weight 2)
commands = ["doctrine slice"]        # command token-prefix (weight 1)
tags     = ["cli", "architecture"]   # set intersection (weight 0)
workspace = "default"
repo = "github.com/davidlee/doctrine"
repo_id_kind = "remote"              # explicit|remote|local_root
repo_id_confidence = "high"          # high|medium|low

[git]                                # anchor; doctrine builds it
anchor_kind = "commit"               # commit|checkout_state|none
commit = "…"                         # set iff anchor_kind = commit
tree = "…"                           # HEAD^{tree}
checkout_state_id = ""               # set iff anchor_kind = checkout_state (dirty)
base_commit = "…"
ref_name = "refs/heads/main"
verified_sha = "…"
normalizer = "forget.checkout.v1"    # frozen frame-algorithm tag

[review]
verification_state = "unverified"    # unverified|verified|stale|disputed
reviewed = ""                        # date of last verification
review_by = ""                       # scheduled review horizon

[trust]
trust_level = "medium"               # low|medium|high
confidence = "medium"                # low|medium|high
actor_type = "agent"                 # user|service|agent|system|external
actor_id = "…"
producer_id = "doctrine-cli"
capture_method = "manual"

[ranking]
severity = "high"                    # critical|high|medium|low|none
weight = 8                           # integer; higher = earlier

[[relation]]                         # lifecycle edges (designed, inert in v1)
rel = "supersedes"
to  = "mem_018e…"

[[source]]                           # provenance
kind = "code"                        # code|adr|spec|commit|doc
ref  = "src/main.rs"
note = ""
```

## Staleness modes

| Mode | Condition | Metric |
|---|---|---|
| scoped + attested | has scope + `verified_sha` | commits touching scoped paths since SHA |
| scoped, unattested | has scope, no `verified_sha` | days since `reviewed` |
| unscoped | no path/git scope | days since `reviewed` |
| global / evergreen | `repo = ""` + `anchor_kind = none` | **none** — non-decaying `reference` |