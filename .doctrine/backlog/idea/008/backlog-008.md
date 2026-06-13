# IDE-008: Executable phase gates: run phase VT criteria via the SL-057 verify contract at completion-flip, earlier than audit

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Idea

SL-057 ships an executable verification contract: a VT coverage check resolves to
a runnable `[command, *extra_args]` (alias / literal / project-default), runs it,
and derives observed status (exit 0 AND optional matcher ⇒ Verified). That same
contract could make **phase gates executable**.

Today a phase's `VT` exit/verification criteria are proven at **audit** — late,
by close-reading. If a phase plan entry's `VT-NNN` criterion carried (or resolved
to) a runnable check, the `completed` flip could **run** it and fail the gate
in-line, surfacing a broken gate at execution time instead of deferring proof to
reconciliation.

## Shape (rough — for a future slice/design)

- Reuse the SL-057 `verify::resolve` + `coverage_verify` run/dedup seam; do **not**
  fork a second runner (no parallel impl).
- Wire into the `/execute` / phase-completion path (the `in_progress → completed`
  flip), not a new top-level verb.
- Open question: does a phase `VT` criterion own its check inline, or cite a
  coverage entry key that already carries one? The latter reuses the SL-057 store
  directly and keeps one source of truth for "what runs".
- Keep advisory-vs-gating deliberate (cf. conduct autonomy): a failing executable
  gate should *block the flip* only where the project opts in.

Relates: SL-057 (the verify contract this builds on), ADR-009 (phase/lifecycle
FSM, conduct axis), SPEC-002 (VT continuous re-derivation).
