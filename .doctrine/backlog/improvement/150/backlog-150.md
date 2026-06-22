# IMP-150: MCP review tool inline help — document response fields, add workflow + examples

The 10 MCP review tools (`review_new`, `review_list`, `review_show`,
`review_raise`, `review_dispose`, `review_verify`, `review_contest`,
`review_withdraw`, `review_status`, `review_prime`) defined in
`src/mcp_server/tools.rs` L26–195 carry the adversarial review protocol —
structured findings, role-turn coordination, the close-gate, the primed
context cache — but the MCP descriptions omit response shapes, workflow
guidance, examples, and several protocol invariants from `review-ledger.md`
and the consuming skills (`/audit`, `/inquisition`, `/code-review`).

## What works

- **Verbs are all exposed.** All 10 core verbs have MCP tool definitions with
  coherent parameter documentation.
- **Parameter descriptions are clear.** Each field carries a short `"description"`
  string in the JSON Schema — severity vocab, disposition options, facet enum,
  reference format all documented.
- **CLI help parity.** The MCP descriptions roughly match the CLI `--help` text.

## Gap 1: response fields wholly undocumented

APPLIES TO ALL 10 TOOLS. No tool description includes a `Returns: { ... }` block.
An agent sees opaque fields with no semantics:

| Field | Values | Tool(s) | Issue |
|---|---|---|---|
| `findings[].status` | `"open"`, `"answered"`, `"contested"`, `"verified"`, `"withdrawn"` | `review_show` | Finding state machine never explained |
| `findings[].severity` | `"blocker"`, `"major"`, `"minor"`, `"nit"` | `review_show` | Only `blocker` gates close — never mentioned |
| `findings[].disposition` | string or `null` | `review_show` | Present only when responded-to |
| `findings[].response` | string or `null` | `review_show` | Present only when responded-to |
| `findings[].detail` | string | `review_show` | Blanked under `view=summary` — surprise |
| `awaiting` | `"raiser"`, `"responder"`, `"none"` | `review_show`, `review_status` | Turn-indicator: who acts next. Never explained. |
| `finding_count` | integer | `review_show`, `review_status` | Total findings ever raised? Current? Unclear. |
| `body` | string | `review_show` | The `## Brief` + `## Synthesis` markdown. Blanked under `view=summary` |
| `status` | `"active"`, `"done"` | `review_show`, `review_list`, `review_status` | Derived: any open/contested/answered → active |
| `total` | integer or absent | `review_list` | `skip_serializing_if None` — absent when uncapped, not `null` |
| `rounds` | integer | `review_status` | Number of baton handoffs (raise/dispose cycles) — undocumented |
| `cache_primed` | boolean | `review_status` | Whether the prime cache is current — undocumented |
| `stale_paths` | array of strings | `review_status` | Paths whose git-sha has diverged since prime — undocumented |
| `finding_id` | `"F-N"` | `review_raise`, `review_dispose`, `review_verify`, `review_contest`, `review_withdraw` | The finding's scoped id — undocumented |
| `review_id` | integer | `review_raise`, `review_dispose`, `review_verify`, `review_contest`, `review_withdraw` | The parent RV's numeric id — undocumented |
| `canonical` | `"RV-NNN"` | `review_new`, `review_prime`, `review_status` | Canonical ref — undocumented |
| `tracked_paths` | array of strings | `review_prime` | Paths under watch — undocumented |
| `areas_count` / `tracked_count` / `invariants_count` / `risks_count` | integer | `review_prime` | Prime cache stats — undocumented |
| `is_seed` | boolean | `review_prime` (implicit) | `--seed` mode skips persistence and returns only `canonical`, `tracked_paths`, `areas_count` (or zero), and `is_seed: true` — modes produce different shapes, neither documented |

**Fix**: add a `\n\nReturns: { ... }` block to every tool description, modeled on
the memory tool pattern (`memory_find` L206, `memory_retrieve` L221,
`memory_show` L249, `memory_list` L260).

## Gap 2: no turn-protocol / workflow guidance

The memory tools chain explicitly (`find → show → retrieve`). The review tools
have a richer two-role adversarial protocol but NO tool explains it:

```
review_new (open ledger)
  → review_prime (warm context cache)
  → review_raise (raiser raises findings)
  → review_dispose (responder answers)
  → review_verify (raiser accepts — terminal)
  → review_contest (raiser pushes back — hand to responder)
  → review_withdraw (raiser retracts — terminal)
→ review_status (check done)
```

An agent encountering these cold has to reverse-engineer the turn model from
field names and the `awaiting`/`--as` parameters. Key protocol invariants from
`review-ledger.md` that should be inlined or linked:

- **Role-turn model**: raiser raises/verifies/contests/withdraws; responder
  disposes. `--as` switches roles — cooperative, not a security boundary
  (ADR-007, `review-ledger.md` §4).
- **Severity + close-gate**: only `blocker` gates the target's close transitions
  (`review-ledger.md` §3, §6).
- **`--note` is ephemeral**: on `verify`/`contest`, the `note` parameter is baton
  chatter for the log, NOT durable rationale. Durable justification belongs in
  a finding's `response` or a new finding (`review-ledger.md` §4).
- **Parent-tree caveat**: review verbs refuse a worktree/fork-resolved root;
  drive from the main tree (`review-ledger.md` §6).
- **Self-review posture**: when one agent is both raiser and responder, use
  `--as raiser` / `--as responder` to switch roles. The per-review lock and
  per-finding `can()` gate keep it correct.
- **Prime before review**: `review_prime --seed` → curate → `review_prime`
  warms the context cache. `status` reports `cache_primed` as optimization
  signal, never a gate (`review-ledger.md` §2).
- **Finding lifecycle**: `open → answered → (verified | contested → answered
  → ... | withdrawn)`. Open/contested can also be directly withdrawn.

**Fix**: add a short "Protocol" sentence to the top-level tool descriptions
that carry role semantics (`raise`, `dispose`, `verify`, `contest`, `withdraw`),
and a longer workflow block on `review_new`.

## Gap 3: no examples

None of the 10 tool descriptions include example invocations or response shapes.
Even a single `review_show` response example with the finding skeleton labelled
would eliminate most guesswork about `awaiting`, `findings[]`, and the
`view=summary` interaction.

**Fix**: one example JSON response block for each output shape class:

```json
// review_show RV-007 (full):
{
  "Showed": {
    "id": 7,
    "canonical": "RV-007",
    "title": "Reconciliation review of SL-024",
    "status": "active",
    "awaiting": "raiser",
    "facet": "reconciliation",
    "target": "SL-024",
    "finding_count": 3,
    "findings": [
      {
        "id": "F-1",
        "status": "answered",
        "severity": "blocker",
        "title": "Expected vs observed: ...",
        "detail": "Evidence at src/foo.rs L42: ...",
        "disposition": "fix-now",
        "response": "Fixed in abc1234"
      }
    ],
    "body": "## Brief\n\n..."
  }
}
```

## Gap 4: disposition vocab incomplete / inconsistent

The `review_dispose` MCP schema says `"The disposition: fixed | design-wrong |
tolerated"` — only three values. But `review-ledger.md` §4 lists five:
`aligned | fix-now | design-wrong | follow-up | tolerated`.

- The CLI `--help` says `"(free-text; e.g. fixed / design-wrong / tolerated)"`
  — which is accurate (free-text) but misleading by example.
- The MCP description reads like a closed enum. An agent may restrict itself
  to those three values.
- `aligned` and `follow-up` are essential dispositions used by the audit and
  inquisition skills.

**Fix**: update the MCP description to match the full vocab:
`"The disposition: aligned | fix-now | design-wrong | follow-up | tolerated"`.
Keep it as `type: "string"` (free-text in practice, but these are the sanctioned
values).

## Gap 5: `view=summary` interaction undocumented

`review_show` accepts `view: "summary"` which blanks `body` (the markdown prose)
and per-finding `detail` + `response`. But:
- The tool description says "summary drops the brief body + per-finding prose,
  keeping the finding skeleton" — "per-finding prose" is ambiguous (does it
  mean the `detail` field? `response`? both?)
- The response still includes the fields (as empty string / null), which could
  confuse an agent that doesn't know they were blanked.

**Fix**: clarify that `view=summary` blanks `body` (markdown prose) and sets
each finding's `detail` to `""` and `response` to `null`, while preserving
`id`, `status`, `severity`, `title`, and `disposition`.

## Gap 6: `review_prime --seed` mode shape divergence

`review_prime` has two modes: normal prime (persists the `domain_map`) and
`--seed` (emits git-changed candidate paths, writes nothing). The response
shape differs — `is_seed` is set `true`, only `canonical` and `tracked_paths`
are populated, the count fields are zero. Neither mode's response is documented.

**Fix**: note the mode split in the description and document both response
shapes.

## References

- Tool definitions: `src/mcp_server/tools.rs` L26–195
- Handler dispatch: `src/mcp_server/tools.rs` L364–480
- Review output enum: `src/review.rs` L767 (`ReviewOutput`)
- Review CLI: `doctrine review --help` (all subcommands)
- Ledger protocol: `install/review-ledger.md` (shipped to `.doctrine/review-ledger.md`)
- Consuming skills: `plugins/doctrine/skills/audit/SKILL.md`, `plugins/doctrine/skills/inquisition/SKILL.md`
- IMP-148 (prior art for memory tools): `.doctrine/backlog/improvement/148/backlog-148.md`
