# DEC-279: Design adopt verb derives the crossing

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Diff implementation (inq-5)

Per-section diff via the `similar` crate (user-approved dependency, 2026-09-24). Rejected: shelling out to `git diff --no-index` (impurity, temp files, git coupling in a reporting path); an in-tree line diff (parallel implementation of a solved problem). Value case: an agent adopting an edit it did not make (human edit, cold context) has no other cheap way to see the delta against the snapshot — `git diff` compares against the last commit, not the last materialise.

## Bare adopt (inq-4)

Without `--expect`, the basis is the fingerprint read at entry: bare `adopt` adopts what is on disk at that moment. The pre-write re-check still compares against that admitted fingerprint (`PreWriteBasis::AdmittedAt`), so an edit landing mid-adoption refuses. The residual is that the caller did not see the adopted bytes; the report always prints the adopted fingerprint, and the reviewed path is `adopt --dry-run --diff` then `adopt --expect <fingerprint>`. Requiring `--expect` was rejected: a two-call dance whose first call exists only to learn the value an agent then pastes.

## Seam (inq-6)

`run_apply` splits at the parse boundary: wire parse + unknown-key check + deserialise, then one pipeline taking a typed `ApplyRequest` and a crossing mode (`Ordinary` | `Adopt { expect }`). The wire path passes `Ordinary`; the verb builds an instruction-free request with self-supplied admission inputs and passes `Adopt`. The pure core reads adoption from the mode, not the request, so `adopt_authored` leaves the wire type with no internal-only field. `--dry-run` returns after pass 1 (the candidate computes every change row and writes nothing; adoption mints nothing). The aligned no-op is decided before admission. Journal-before-snapshot, validate-before-rebaseline and the pre-write re-check stay in the one pipeline. Rejected: a dedicated core entry with its own writer (second copy of the ordering-sensitive write path); the verb serialising a payload for `run_apply` (needs a wire key, contradicting DEC-278).

## Locked runs (inq-7)

`adopt` refuses on a locked run, with a remedy naming the regression to `reviewing` for a run still governing execution, and the direct-edit path at reconcile. No other verb's locked-stage behaviour changes (nothing else refuses at `locked` today; captured separately as backlog).

This makes the reconcile workflow canonical: reconcile edits `design.md` directly and does not adopt; the divergence stands. The reconciled bytes cannot be lost to a later `materialise`: `materialise` refuses at entry on a diverged watermark (`commands/design.rs:2073`), and the only doctrine path that moves a locked run's authored tier is regress then `adopt`, which takes the document (the reconciled text) as truth. The residual is DEC-100's tolerated check-to-rename window, which needs a concurrent materialise.