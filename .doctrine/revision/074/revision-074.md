# REV REV-074 — Embed-sourced skills and the pi boot adapter

Revision — a pending revise-intent against authored governance/spec truth. The
structured `[[change]]` payload lives in the sister `revision-074.toml`; this
prose companion carries the rationale and the free-text before/after excerpts
the structured payload can only label.

Raised by SL-284 during execution (design sec-6), applied at reconcile.

## Rationale

SL-284 deleted an installation mechanism and replaced it with a different one,
so the specs that describe installation now describe something that no longer
exists. The landed behaviour (commits `0fed5b55f`.., PHASE-01..04):

- every registered harness with a `skills` route receives the embedded catalog
  directly — canon at `.doctrine/skills/<id>`, a relative link at its declared
  site (`.claude/skills` for Claude; `.agents/skills` for codex, pi and
  universal) — and an explicit `--agent` absent from the registry is refused
  before any write, instead of being passed to an external runner;
- the `npx skills add …` delegation, its `skills_via` registry key and the
  `[install] repo` setting are gone;
- pi has its own boot adapter: it imports `AGENTS.md`, auto-detects on `.pi/`,
  writes its generated extensions into the disposable runtime tier
  (`.doctrine/state/pi/extensions`) and registers that directory through
  `.pi/settings.json`; codex's arm no longer writes pi files.

The typed payload below is the narrowest the engine admits: `modify` rows carry
the requirement the revision will re-write, and the replacement statement for
each is recorded here because a `modify` row takes no text. Decision rows
(`D2`, `D4`, `D5`), the concerns/hypotheses blocks, overview and responsibility
prose are doc-local — they have no peer FK — so they are enumerated below as the
manual payload for reconcile, not as rows.

`REQ-480`'s status row is conditional: apply it only once reconcile has verified
the landed pi adapter against the requirement. Do not infer other SPEC-011
pending requirements active by association.

## Replacement statements (the `modify` rows)

**REQ-175** (SPEC-010 FR-003) — Route each registered harness with a `skills`
route directly from the embedded catalog: materialise `.doctrine/skills/<id>`
and reconcile relative links at its declared link site (`.claude/skills` for
Claude; `.agents/skills` for codex, pi and universal). Refuse an explicit
`--agent` that is absent from the registry before writing, suggesting
`universal` for another harness that reads `.agents/skills`. No npx delegation.

**REQ-172** (SPEC-009 NF-003) — Install owns the embedded skill canon and the
links for every registered harness with a `skills` route (SPEC-010); it does not
absorb global-memory materialisation, which remains the standalone
`doctrine memory sync` verb that install only hints at.

**REQ-185** (SPEC-011 FR-005) — `boot install` prepends
`@.doctrine/state/boot.md` once into each target file: Claude `CLAUDE.md`, codex
and pi `AGENTS.md`, deduplicating same-inode targets. Explicit `--agent pi`
selects pi; `.pi/` auto-detects pi, while `AGENTS.md` alone continues to imply
codex, not pi.

**REQ-480** (SPEC-011 FR-011) — The pi boot adapter plans and installs owned
`index.ts`, `mcp.ts` and `surface.ts` in `.doctrine/state/pi/extensions` through
the shared ownership-aware descriptor core (generate absent, regenerate changed
owned files, skip foreign files), reporting each file separately; it ensures
`.pi/settings.json` lists `../.doctrine/state/pi/extensions` while preserving
foreign entries. A malformed settings file, or an `extensions` key that is not
an array, is left untouched with the line to add printed; that outcome is
reported, a settings write counts as touched, and the fallback leaves the boot
leg incomplete. A write prints a project-trust/`/reload` activation notice and
never claims activation. `surface.ts` retains the existing neutral-envelope
bounded fail-open memory-surface adapter contract. Codex writes no pi files.

## Prose payload (manual, at reconcile — no peer FK, so no rows)

**SPEC-010 — whole-spec Claude-only residue is in scope, not just `D2`/`REQ-175`.**
- Overview and Responsibilities, and structured responsibility 3
  (`spec-010.toml`): replace the dual-path claim with direct embedded
  canon+links for every registered skills route; keep the marketplace channel
  as an external channel statement, not doctrine's installation path. In the
  responsibilities prose, "reconcile each Claude link" → each registered
  harness link.
- `D2` and the surrounding "Dual-path routing / canonical links / command
  surface" prose: `D2` becomes "registered `skills` routes link the embedded
  catalog"; no runner, delegated argv or Node fallback; explicit unknown agent
  refused with the `universal` suggestion; `--only-memory` selects the memory
  pair. `D3`'s "Claude gets" example is no longer exhaustive.
- Concerns/hypotheses: delete **Delegate version skew (accepted)**; delete
  **Multi-agent status fidelity** (every registered skills link site is now a
  directory doctrine reconciles; foreign or shadowed entries are reported, not
  claimed owned). Replace **Delegate the long tail, special-case only what we
  own** with **Register the supported harnesses, share the embedded canon**: a
  new harness is a registry entry with a declared link site; unknown names are
  refused. Add **`universal --global` destination differs from npx (accepted)**:
  it links under `$HOME/.agents/skills`, not npx's `~/.config/agents/skills`,
  and doctrine models no per-scope route override.

**SPEC-009** — Overview/D4/D5 (`spec-009.md`): delete the npx peer/interaction
promise; `D4` declares exactly one `skills` route per registered harness, no
`skills_via`, and pre-write refusal of an unknown name rather than disclosure
after the fact; `D5` reads "memory sync remains separate; install owns the
registered skills links", keeping the memory-sync hint. Leave its forward-intent
warning (D6–D9) intact.

**SPEC-011** — responsibilities and boot-install prose: add pi's `AGENTS.md`
import, `.pi/` detection, and the pi-only runtime extension/settings arm; the
codex arm writes only hooks, MCP and the `AGENTS.md` import. Describe the
settings outcome separately (write vs malformed manual fallback), the
trust/`/reload` notice on write and silence on no-op, and the shared import
dedup. The existing foreign-skip concern gains pi-settings merge safety.

## Search dispositions (design sec-6 requires none left unaddressed)

`doctrine search npx -k all` — remaining hits are historical revision/spec text
and the marketplace channel, both covered above: the marketplace mention may
stay only as a third-party external channel; the rest is addressed by the
SPEC-009/010 payload. No live installation promise survives.

`doctrine search skills_via -k all` — hits in the corpus are historical slice
and revision text plus `src/install.rs`'s `stale_skills_via_field_is_refused`
test, which deliberately proves a stale manifest is refused. No authored
specification retains a `skills_via` route.

## Out of scope

Backlog items that reference the old npx path keep their own identity; they are
not edited by this revision. `mem.fact.doctrine.agents-skill-mirror-is-published-source`
is retired in the slice close work, not here.
