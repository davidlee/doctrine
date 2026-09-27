# Notes SL-271: Codex MCP registration during install

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage (explore.triage, 2026-09-27)

Full evidence: `research/research.md` (✓ rows only are load-bearing) and IMP-111's
*Design decisions to settle* (D1–D5).

### Constraining governance

- **ADR-001** — `boot` is command tier; a shared classification core must not
  acquire a caller's domain concept (the D2 precedent). Read the module doc
  comment before siting a new function (`mem.pattern.layering.direction-is-not-cohesion`).
- **ADR-013** — a SPEC-011 change lands through a REV at close, not by a phase.
- **ADR-019** — a client-owned file edit is admissible on an
  integration-justification, minimal-footprint basis (constrains rationale, not
  mechanism).
- **POL-002** — no host abspath in a tracked artefact (facet 1/2); facet 3 forbids
  a **silent** host-tool dependency, which the `sh -c` wrapper is.
- **POL-003** — facets 1–2 keep codex vocabulary at the adapter edge and off
  incidental seams; **facet 3 is the disclosure authority** (opt-in + disclosed,
  no claiming a live entry while trust is pending).
- **STD-001** — every recurring literal (`.codex/config.toml`, `mcp_servers`,
  `env_vars`, `DOCTRINE_BIN`, the wrapper line) gets one named constant.
- **SPEC-011** REQ-185 (plan/apply split), REQ-186 (ownership-predicate merge
  posture), REQ-476 (form scoped to the Claude file — *not* textual cover),
  REQ-477 (reporting), REQ-479 (a codex leg gets its own member). SPEC-009 /
  PRD-006: trackedness comes from the manifest, whose entries add nothing for
  `.codex` ⇒ a client file is tracked ⇒ portable form.

### Shaping decisions (to be settled in `/design`)

| id | decision | current lean |
|---|---|---|
| D1 | report seam | one field carrying its rel path; `MCP_REL` is hardcoded in the `wire()` strings today |
| D2 | policy vs rendering | **separate** pure planner + shell beside the owner-locked merge core, per `mem.pattern.distribution.settings-key-rides-beside-hookspec-merge-core`; keep TOML rendering per-arm |
| D3 | ownership predicate / emitted-form set | fix the emitted set as named constants first; the no-op comparator must track the emitted value (`mem_019f286c92fe77a391635ba1d0743d5f`) |
| D4 | portable vs baked | **portable unconditionally**, mirroring `plan_mcp` — no resolver exists for tracking status, and portable satisfies POL-002 either way |
| D5 | disclosure + file ownership | disclose the trust skip (POL-003 facet 3); decide explicitly whether install also ensures `[features] hooks = true` in the file it now writes |

### Risks

- Comparator thrash: the SL-195 `F-1` / `mem_019f286c92fe77a391635ba1d0743d5f` class.
- Parallel planner: duplicating the JSON decision table (D2).
- Ordering: if both a hook leg and this leg touch `.codex/config.toml`, an ordered
  write needs an explicit guard on the fail-soft outcome, not `?`
  (`mem.pattern.rust.fail-soft-ok-defeats-sequencing`).
- Clobbering a user-owned file: `[features] hooks = true` and comments must
  survive (the edit-preserving requirement is load-bearing here).
- Silent trust-gated skip: install must not imply the entry is live.
- Three parallel agent-detection resolvers (`detect_agents` / `resolve_agents` /
  `resolve_harnesses`) — no detection change is expected here, so do not touch
  one in isolation (`mem_019f139f1dc071829a885949da8203a3`).

### Assumptions (to be recorded as ASM records in the design)

- codex's project layer may override `mcp_servers`, and `.codex/config.toml` is
  the project path (probed at codex 0.155.1).
- `command = "sh"` resolves from PATH, not a project-local `sh` (OQ-6).
- `env_vars` is the supported forwarding seam for `DOCTRINE_BIN`.

### Test-design hazards (from memory)

- Seed ownership-dependent fixtures **literally**, never via a real installer
  (`mem.pattern.boot.test-exec-is-not-doctrine-owned`).
- An absence assertion needs a positive control in the same test
  (`mem.pattern.testing.absence-assertion-over-an-unwritten-file`).

### Open questions

D1–D5 above, plus OQ-6 (project-local `sh` / `cwd` pickup) — OQ-6 resolves with
D4 (portable), where the declaration duty is POL-002 facet 3.

### Review pass (RV-399, 2026-09-27) — what a further pass would probe

Written after the last pass (adversarial review of design.md rev 25/27, fixes
materialised at rev 29; 14 findings, all `fix-now` + verified). A further pass
would focus on what the fixes themselves introduced or left open:

1. **The `is_doctrine_wrapper_line` pattern.** It is now the pivot of ownership — a
   stale-but-ours wrapper refreshes, everything else is foreign. Probe it with
   adversarial line shapes: extra whitespace, a quoted program containing
   `doctrine` in a path component (`/opt/doctrine-tools/bin/other`), an
   env-prefixed line, a trailing comment, and a wrapper whose program is the
   literal `${DOCTRINE_BIN:-doctrine}` re-quoted differently. A false positive
   rewrites a user entry; a false negative strands our own older wording.
2. **The `env_vars` clause.** Check it cannot be satisfied accidentally by a
   sibling key, and that a codex-canonical ordering/spelling variant of the same
   whitelist is not read as stale (repeated refresh thrash).
3. **The probe seam.** Verify the injected-runner seam cannot be satisfied by
   `install::Runner` (stdio-inheriting) by accident, and that the empty-`PATH`
   e2e really pins `Unknown(reason)` rather than a generic failure.
4. **The Rewording obligation.** The report now says "wrote"; check no other line
   (docs, help text, the trust caveat) still claims activation, and that the
   caveat is not printed on `dry_run` or on `PrintedFallback`.
5. **`concat!` composition.** Confirm the constants-agreement test actually fails
   when `PORTABLE_EXEC` changes wording, i.e. it is not a tautology.

No further pass is required to lock: every one of these is a refinement of an
accepted decision (DEC-323/325/328/329/332), not an open question.

### Second pass (RV-399 rounds 2, 2026-09-27) — outcome and what a third would probe

A second fresh pass attacked the fixes rather than the design, and found 17
turther issues — including a genuine compile blocker (`concat!` cannot compose
over `const`s) and seven majors: the disclosure predicate was harness-blind (a
Claude install would have probed codex), the ownership formula contradicted its
own edge table on extra keys/args, `args` indexing could panic on a short entry,
the wrapper pattern was whitespace-fragile and duplicated `is_doctrine_program`,
"a baked abspath is foreign" contradicted the pattern that accepted one, the
shared classifier was never wired into the Claude arm, and `CommandRunner` had no
concrete implementation while the impact table missed five call sites. All are
fixed; the Claude `plan_mcp` refactor is now explicit with the existing suite as
the behaviour-preservation proof.

A third pass would probe: the `normalise` + quoted-span extraction against
pathological lines; whether the strict "extra key ⇒ foreign" rule surprises a
user who adds `enabled = false` (a disclosure question, not a correctness one);
the `plan_mcp` refactor's behaviour preservation against hand-built JSON edge
cases the existing suite may not cover; and the `CaptureRunner` default wiring at
both production call sites.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
