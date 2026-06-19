---
seq: 0011
scope: capture
target: discovered — `validate` / spec FK check / memory validate / relation danglers
confidence: med
reversible: yes (proposal only; read-only capture, nothing built)
---
## What
Corpus integrity checking is **fragmented across at least four disjoint command
surfaces**, with no single "is my governance graph healthy?" gate:

- `doctrine validate` — **id-integrity only** (dir basename == toml id, no intra-kind
  duplicate, alias target equality; ADR-006 D3 detect-half). Top-level verb.
- spec FK integrity (`src/main.rs:1471-1475`) — "dangling member / interaction FKs,
  duplicate labels, and (corpus-wide) orphan requirements." A `spec` subcommand.
- memory validation (`src/main.rs:2000`) — "dangling relations, stale verification,
  draft expiry." A `memory` subcommand.
- relation / supersession integrity (`relation_graph::validate_supersession`,
  `src/relation_graph.rs:418`; per-entity danglers via `inspect`).

Each is real and good; the gap is that there is **no aggregating surface**. A team
or CI job that wants "is the corpus sound?" must know and run four+ separate
commands, union their exit codes by hand, and reconcile four output shapes. The
orientation dashboard `doctrine status` does **not** fill this — a grep of
`src/status.rs` for integrity/dangling/orphan/validate is empty; status reports
active/blocked work and boot staleness, not graph integrity.

For the standing focus (indispensable to teams), this is the obvious CI/pre-commit
gate the topology is begging for: one `doctrine doctor` (or `validate --all`) that
runs every integrity check across the whole graph — id, FK, danglers, orphans,
supersession, memory — and returns one go/no-go plus a unified, actionable report.
The graph is the product's source of truth; teams need one command to trust it.

## Options
1. **New `doctrine doctor` verb** that orchestrates the four existing checks and
   prints a unified health report (sectioned by check, one exit code). Tradeoff:
   clearest mental model + best discoverability ("doctor" is the universal name);
   the checks already exist, so it's an aggregator, not new validation logic.
2. **`doctrine validate --all`** — extend the existing id-integrity verb to a
   corpus-wide mode that folds in FK/danglers/memory/supersession. Tradeoff: no new
   verb, reuses the name teams already reach for; risk of overloading `validate`
   (currently a tight, single-purpose ADR-006 verb) and muddying its contract.
3. **Leave fragmented; document the four commands as a "health checklist."**
   Tradeoff: zero build; but a checklist is not a gate — CI can't trust it, and the
   "run these four" knowledge lives outside the tool.

## Recommendation
Option 1 (`doctrine doctor`) as a thin aggregator over the four existing check
functions, with a `--json` shape for CI and a single nonzero-on-any-violation exit.
Rationale: the validation *logic* already exists and is tested; what's missing is
composition + one trustworthy exit code, which is exactly what a CI gate needs.
Keep `validate` tight (its ADR-006 contract is clean); `doctor` calls into it rather
than swallowing it. This is the lowest-effort path to a genuinely
team-indispensable surface — the difference between "doctrine has checks" and
"doctrine tells you, in one command, whether your governance graph is sound."

Decisions deferred to YOU:
- (a) **build or leave fragmented** — is a unified gate wanted, or is per-area
  validation the intended ergonomics?
- (b) **`doctor` verb vs `validate --all`** — new surface vs extend the existing one
  (the contract-purity vs discoverability trade).
- (c) **scope of aggregation** — exactly which checks are in the "health" set
  (the four above + the layering fitness test from 0010? + lazyspec/coverage?), and
  whether any are warn-only vs gate-failing.

## Next doctrine move
```
# confirm the four surfaces + the absence of aggregation (read-only):
doctrine validate --help
doctrine spec check --help          # the FK/orphan check (verb name per --help)
doctrine memory validate --help     # (verb name per --help)
grep -n 'integrity\|dangl\|orphan' src/status.rs   # empty → status is not it

# capture the unified gate (NOT executed — fence forbids backlog transition):
doctrine backlog new improvement "doctrine doctor: unified corpus-health gate \
  aggregating id-integrity + spec FK/orphans + memory + relation/supersession \
  danglers into one go/no-go + --json for CI — pieces exist, composition missing" \
  --tag area:cli --tag area:governance --tag ci
```
(Verbs described, NOT executed.)

## Illustration (optional)
None — a capture. The substance is recognising the four scattered checks as one
absent gate; the build is mechanical composition once the verb/scope is chosen.
