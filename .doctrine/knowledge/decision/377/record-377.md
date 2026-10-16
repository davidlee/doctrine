Amends DEC-373 on two points, from the RV-418 adjudication (findings F-3 and
F-11), with user assent on 2026-10-16 ("F-3 and F-11: i'll take your
recommendations").

**Global installs converge without pruning.** `--global` installs refresh canon
copies and relink doctrine links under `$HOME`, but prune nothing. Orphans are
reported instead. `$HOME/.doctrine/*` is shared by every project that installs
globally, possibly with different binary versions, so no single catalog owns it.
An older binary pruning a newer one's members would be a silent regression.
This replaces DEC-373's "prune there uses the same ownership rule". Project
installs keep exact convergence (DEC-374).

**Unregistered harness names keep their npx skills route.** `--agent <name>`
with no `[harness.<name>]` table gets no agent defs and no links, but its skills
are still delegated to `npx skills add --agent <name>`, as today, and install
discloses that the harness is unregistered. DEC-373's "installs nothing" would
have silently dropped cursor and every other npx-supported agent.

**The registry also carries detection and boot routing.** Each
`[harness.<name>]` table gains `detect` (the directory whose presence implies
the harness, replacing `detect_agents`'s hardcoded list) and `boot` (the boot
adapter: claude → claude, pi → codex, codex → codex, universal → none;
replacing `boot_harness_names`).

Rejected: global prune under the shared rule (cross-project and cross-version
hazard); "installs nothing" for unregistered names (regresses working npx
agents).
