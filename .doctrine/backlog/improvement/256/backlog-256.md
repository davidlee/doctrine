# IMP-256: Plan-time selector completeness check

## Problem

Plan VTs name `test_file` paths (e.g. `tests/e2e_prompt_resolve_golden.rs`) that
are not in the slice's `[[selector]]` registry. The gap surfaces late — at
funnel `import --slice` time — as `undeclared-scope`, forcing a mid-drive
selector-add that advances coord HEAD and strands the in-flight worker
(base-drift trap).

Witnessed 4 times:
- SL-193: plan VT-3/VT-4 mandated `tests/e2e_prompt_resolve_golden.rs` — not
  in design-target selectors → `import --slice` refused
- SL-191-P02: new model-band trait key changed the same golden file — not in
  selectors → import refused
- SL-191-P05: same golden file coupling, caught one funnel round late
- SL-194-P01: new `Command::Findings` variant forced `guard.rs` write outside
  selectors → import refused, post-hoc selector-add stranded P01 worker

A new verb's compile-forced touch points (guard.rs write_class, help-tree
assertions, census count) are also systematically undiscoverable from the plan
wording.

## Fix direction

- **`doctrine slice selector doctor <ID>`** (or `check plan <ID>`): reads
  `plan.toml`, collects all `VT-NN.test_file` paths, cross-references against
  the slice's `[[selector]]` registry, and flags undeclared paths.
- **Plan skill integration**: run `selector doctor` after authoring plan.toml,
  before commit. "If any VT test_file path is undeclared, add it to the
  selector set or mark it as aligned-incidental."
- **Stretch**: a checklist of "files a new verb variant touches" (cli enum +
  guard write_class + help-tree/census asserts) surfaced as a planning aid.
- **Stretch**: detect plan VT test_file paths that match known golden files
  (`tests/e2e_*_golden.rs`) and warn that selector scope must include them.

## Related

- RFC-011 case-notes: SL-193, SL-191-P02, SL-191-P05, SL-194-P01
- IMP-162 (lint over-broad selector globs — inverse problem, same surface)
- IMP-257 (base-drift trap — the consequence when selectors are added
  mid-drive)
- IMP-209 (structured VT mandates — the plan-side fix that makes test_file
  parseable)
