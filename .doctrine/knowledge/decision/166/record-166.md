# DEC-166: Extract link reconcile, keep the target loop local

## Decision

The proven-ownership link trichotomy is extracted to **one helper**, and the
rebuilt skills channel (`OQ-2b`) gets its multi-target capability as a **local
loop** over link dirs. No generic multi-target driver.

## The duplication already exists

The link-reconcile block is **byte-identical** in the two live legs:

- agents — `src/install.rs:2179-2191`
- workflows — `src/install.rs:2279-2291`

Same `classify_link` match, same three arms, same `write_link` calls, same
`writeln!` strings. So the fourth-copy risk `T3` raised is not something the
skills leg would introduce: two copies are here today, and skills would be the
third. Extracting removes duplication that **exists**, not duplication forecast
for tomorrow.

## Why not a generic driver

A driver taking canonical dir, link dirs, items and a materialise callback is
maximally DRY on paper, but it would have to absorb three genuinely unlike
materialise phases:

| leg | materialise |
|---|---|
| agents | single file, **with a hymns bake** (`src/install.rs:2164-2175`) |
| workflows | single file, plain |
| skills | a whole directory tree |

plus the agents leg's per-agent-name link-dir branching
(`"claude" => claude_agents_dir, _ => pi_agents_dir`). `T3` itself observes that
the materialise step is exactly where the legs differ — so a driver unifying
them would be absorbing the one part that genuinely varies, for three callers.

## Keeping the extraction sane

The boundary is deliberate. The helper takes the inputs the existing block
already has (`file_name`, `dest`, `target`, the writer) and does the trichotomy.
It does **not** grow a materialise callback, does **not** become a trait, and
does **not** absorb the agent-name branching. Small enough that the two existing
call sites shrink and nothing else moves.

## Parameterisation is a rename, not construction

The recovered code is already shaped for this — `claude_links(skills, agent_dir,
canon_dir)` in `git show 347197e8^:src/skills.rs` already takes the link dir as a
parameter and its body is agent-agnostic. `OQ-9` describes the work correctly:
rename `claude_links` → `agent_links`, hoist `agent_dir` out of
`install_for_claude` into a parameter, loop the link phase over targets while
materialise still runs once.

## Scope

The loop takes a **list**, but SL-250 drives it with **one entry**
(`.claude/skills`). Shipping `.agents/skills` as the second target, and deciding
which harnesses stop delegating to `npx`, is IMP-406's — per `OQ-9` this slice
ships only the parameterisation.

Recorded from design run `dr-019fd692` checkpoint `cp-6` disposing `inq-6`.
