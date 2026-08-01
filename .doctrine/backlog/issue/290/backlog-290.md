# ISS-290: Payload term ValueKind declared but never checked at construction

Every `ChangeEvent` declares the shape of its payload —
`ChangeEvent::payload_terms()` in `src/design_run/change_log.rs` returns
`&[(PayloadKey, ValueKind)]`. **Nothing enforces that declaration against the
terms actually constructed.** `ChangeEvent::ordered` reads `payload_terms()` for
**key order only** and never inspects the `ValueKind`; the only other reader is
`rendered_payload_fits_its_cap_for_every_event_kind` (EX-14(c)/VA-7), which
*saturates* from the declared kinds rather than comparing them to anything real.

So a term built with the wrong `ValueKind` is caught by nothing. The kinds carry
different admission bounds (`Token` → `DESIGN_ID_BYTES` 32, `Label` →
`DESIGN_STAGE_LABEL_BYTES` 16, `Digest`/`Prose` unbounded), so the drift is not
cosmetic: it silently changes what values are admissible, and the containment
proof then proves a budget for a shape that is not the one being built.

## How it surfaced

SL-233 PHASE-08, `F-P08.2`. `StepDischarged` declares
`(PayloadKey::Step, ValueKind::Token)` but `run.rs:1441` constructed a `label`.
A runbook admitted a 64-byte step id while a discharge could only record 16, so
a 17-byte step id parsed, blocked its edge, and could never be discharged — an
unclearable edge, silent until a run tried it. **Latent since PHASE-16**; the
shipped five escaped only because `explore.research` is exactly 16 bytes.

Fixed at the instance in `3516a8b3` (the call site now matches the declared
shape, and `RUNBOOK_STEP_ID_BYTES` is derived from `DESIGN_ID_BYTES` rather than
independently chosen). **The class is still unguarded.**

## Still-live instance: `Outcome`

One line down from the fix, `StepDischarged` declares
`(PayloadKey::Outcome, ValueKind::Token)` while `run.rs:1442` builds a `label`.

Harmless today — `outcome_label` returns `attested` / `skipped`, well inside
both bounds — and **deliberately not repaired**, because unlike the step id it is
genuinely ambiguous *which side is wrong*. `Outcome` really is a closed
vocabulary, so `Label` is arguably the more precise kind and the *declaration*
the error; compare `CheckpointDisposed`, which declares its `Disposition` as
`Label`. Fixing either side without a ruling would entrench a guess. That
ruling belongs with this issue.

## What a fix looks like

The declaration is already the single source of truth for key order; making it
the source for kind too would let `PayloadTerm` construction take its kind
*from* the event rather than from the call site's choice — turning a silent
mismatch into an unrepresentable state rather than a check that has to be
remembered.

Note the wire route matters as much as the constructors: `PayloadTerm`
deserialises through `try_from = "PayloadTermWire"` precisely so re-entry is an
admission point and not a bypass (RV-321 F-1), and a hand-edited snapshot can
currently claim any kind for any key. Whatever binds kind to key should bind it
on that path too, or the guarantee is only as good as the constructors.

Shared machinery, so the **behaviour-preservation gate** applies: the existing
suites are the proof and must stay green unchanged.

Raised from SL-233 PHASE-08 `F-P08.4`. Related: IMP-375 (the design-prompt store
is not project-overridable, which is what makes the PHASE-08 bound tightening
safe today and would stop being true if that changed).
