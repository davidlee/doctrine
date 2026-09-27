A phase that retires symbols an earlier phase's plan VT mandates name
(`test_file` + `keywords`) leaves those mandates mechanically FAILing in
`doctrine slice verify-vt` — and the criteria are immutable-append, so the
keyword set cannot be re-pointed.

This is expected supersession, not drift, but it must be dispositioned: the audit
raises it, and reconcile records it in `design.md` sec-9 "Verification alignment"
(the new phase's VTs are the live ones; the old phase's were satisfied at its
execution and are superseded).

Do NOT edit the `plan.toml` criterion, and do not keep retired symbols alive just
to satisfy the gate. The `verify-vt` FAIL stays visible on the current tree by
design, with the design record as its disposition.

Seen on SL-271: PHASE-04 retired the codex hooks probe (`parse_codex_features`,
`codex_hooks_state`, `HooksState`), so PHASE-03 VT-1..VT-4 read FAIL; RV-404 F-1
routed the record to reconcile.
